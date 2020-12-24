use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, Data, DeriveInput, Fields, ItemFn};

#[derive(Debug, FromMeta)]
struct ConnectorFactoryArgs {
    name: String,
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
            factory: connector_factory,
        };

        #[cfg(feature = "inventory")]
        pub extern "C" fn connector_factory(args: &::memflow::connector::ConnectorArgs) -> ::memflow::error::Result<::memflow::connector::ConnectorType> {
            let connector = #func_name(args)?;
            Ok(Box::new(connector))
        }

        pub fn static_connector_factory(args: &::memflow::connector::ConnectorArgs) -> ::memflow::error::Result<impl ::memflow::mem::PhysicalMemory> {
            #func_name(args)
        }

        #func
    };
    gen.into()
}

/// Auto derive the `Pod` trait for structs.
///
/// The type is checked for requirements of the `Pod` trait:
///
/// * Be annotated with `repr(C)` or `repr(transparent)`.
///
/// * Have every field's type implement `Pod` itself.
///
/// * Not have any padding between its fields.
///
/// # Compile errors
///
/// Error reporting is not very ergonomic due to how errors are detected:
///
/// * `error[E0277]: the trait bound $TYPE: Pod is not satisfied`
///
///   The struct contains a field whose type does not implement `Pod`.
///
/// * `error[E0512]: cannot transmute between types of different sizes, or dependently-sized types`
///
///   This error means your struct has padding as its size is not equal to a byte array of length equal to the sum of the size of its fields.
///
/// * `error: no rules expected the token <`
///
///   The struct contains generic parameters which are not supported. It may still be possible to manually implement `Pod` but extra care should be taken to ensure its invariants are upheld.
///
/// # Remarks:
/// This custom derive macro is required because the dataview proc macro searches for ::dataview::derive_pod!().
/// See https://github.com/CasualX/dataview/blob/master/derive_pod/lib.rs for the original implementation.
#[proc_macro_derive(Pod)]
pub fn pod_derive(input: TokenStream) -> TokenStream {
	return format!("::memflow::dataview::derive_pod!{{ {} }}", input).parse().unwrap()
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
