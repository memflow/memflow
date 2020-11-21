use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, Data, DeriveInput, Fields, ItemFn};

#[derive(Debug, FromMeta)]
struct ConnectorFactoryArgs {
    name: String,
    ty: syn::Ident,
    #[darling(default)]
    version: Option<String>,
}

// We should add conditional compilation for the crate-type here
// so our rust libraries who use a connector wont export those functions
// again by themselves (e.g. the ffi).
//
// This would also lead to possible duplicated symbols if
// multiple connectors are imported.
//
// See https://github.com/rust-lang/rust/issues/20267 for the tracking issue.
//
// #[cfg(crate_type = "cdylib")]
#[proc_macro_attribute]
pub fn connector(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let args = match ConnectorFactoryArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let connector_name = args.name;
    let connector_type = args.ty;

    let func = parse_macro_input!(input as ItemFn);
    let func_name = &func.sig.ident;

    let gen = quote! {
        #[cfg(feature = "inventory")]
        #[doc(hidden)]
        pub static CONNECTOR_NAME: &str = #connector_name;

        #[cfg(feature = "inventory")]
        #[doc(hidden)]
        #[no_mangle]
        pub static MEMFLOW_CONNECTOR: ::memflow::connector::ConnectorDescriptor = ::memflow::connector::ConnectorDescriptor {
            connector_version: ::memflow::connector::MEMFLOW_CONNECTOR_VERSION,
            name: CONNECTOR_NAME,
            vtable: ::memflow::connector::ConnectorFunctionTable {
                create: mf_create,

                phys_read_raw_list: mf_phys_read_raw_list,
                phys_write_raw_list: mf_phys_write_raw_list,
                metadata: mf_metadata,

                clone: mf_clone,

                drop: mf_drop,
            },
        };

        #[cfg(feature = "inventory")]
        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn mf_create(
            log_level: i32,
            args: *const ::std::os::raw::c_char,
        ) -> std::option::Option<&'static mut c_void> {
            let argsstr = unsafe { ::std::ffi::CStr::from_ptr(args) }.to_str().ok()?;
            let conn_args = ::memflow::connector::ConnectorArgs::parse(argsstr).ok()?;

            let conn = Box::new(#func_name(log_level, &conn_args).ok()?);
            Some(unsafe { ::std::mem::transmute(Box::into_raw(conn)) })
        }

        #[cfg(feature = "inventory")]
        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn mf_phys_read_raw_list(
            phys_mem: std::option::Option<&mut c_void>,
            read_data: *mut ::memflow::mem::PhysicalReadData,
            read_data_count: usize,
        ) -> i32 {
            if let Some(conn_box) = phys_mem {
                let mut conn: Box<#connector_type> = unsafe { Box::from_raw(::std::mem::transmute(conn_box)) };
                let read_data_slice = unsafe { std::slice::from_raw_parts_mut(read_data, read_data_count) };
                let result = match conn.as_mut().phys_read_raw_list(read_data_slice) {
                    Ok(_) => 0,
                    Err(_) => -1,
                };
                std::mem::forget(conn);
                result
            } else {
                -1
            }
        }

        #[cfg(feature = "inventory")]
        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn mf_phys_write_raw_list(
            phys_mem: std::option::Option<&mut c_void>,
            write_data: *const ::memflow::mem::PhysicalWriteData,
            write_data_count: usize,
        ) -> i32 {
            if let Some(conn_box) = phys_mem {
                let mut conn: Box<#connector_type> = unsafe { Box::from_raw(::std::mem::transmute(conn_box)) };
                let write_data_slice =
                    unsafe { std::slice::from_raw_parts(write_data, write_data_count) };
                let result = match conn.as_mut().phys_write_raw_list(write_data_slice) {
                    Ok(_) => 0,
                    Err(_) => -1,
                };
                std::mem::forget(conn);
                result
            } else {
                -1
            }
        }

        #[cfg(feature = "inventory")]
        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn mf_metadata(phys_mem: std::option::Option<&c_void>) -> PhysicalMemoryMetadata {
            if let Some(conn_box) = phys_mem {
                let conn: Box<#connector_type> = unsafe { Box::from_raw(::std::mem::transmute(conn_box)) };
                let metadata = conn.as_ref().metadata();
                std::mem::forget(conn);
                metadata
            } else {
                PhysicalMemoryMetadata {
                    size: 0,
                    readonly: true,
                }
            }
        }

        #[cfg(feature = "inventory")]
        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn mf_clone(
            phys_mem: std::option::Option<&c_void>,
        ) -> std::option::Option<&'static mut c_void> {
            if let Some(conn_box) = phys_mem {
                let conn: Box<#connector_type> = unsafe { Box::from_raw(::std::mem::transmute(conn_box)) };
                let cloned_conn = Box::new(conn.as_ref().clone());
                std::mem::forget(conn);
                Some(unsafe { ::std::mem::transmute(Box::into_raw(cloned_conn)) })
            } else {
                None
            }
        }

        #[cfg(feature = "inventory")]
        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn mf_drop(phys_mem: std::option::Option<&mut c_void>) {
            if let Some(conn_box) = phys_mem {
                let _: Box<#connector_type> = unsafe { Box::from_raw(::std::mem::transmute(conn_box)) };
                // drop box
            }
        }

        #func
    };
    gen.into()
}

#[proc_macro_derive(ByteSwap)]
pub fn byteswap_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut gen_inner = quote!();
    match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(named) => {
                for field in named.named.iter() {
                    let name = field.ident.as_ref().unwrap();
                    gen_inner.extend(quote!(
                        self.#name.byte_swap();
                    ));
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let gen = quote!(
        impl #impl_generics ::memflow::types::byte_swap::ByteSwap for #name #ty_generics #where_clause {
            fn byte_swap(&mut self) {
                #gen_inner
            }
        }
    );

    gen.into()
}
