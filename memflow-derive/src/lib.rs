use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, Data, DeriveInput, Fields, ItemFn};

#[derive(Debug, FromMeta)]
struct ConnectorFactoryArgs {
    name: String,
    #[darling(default)]
    version: Option<String>,
    #[darling(default)]
    description: Option<String>,
}

#[derive(Debug, FromMeta)]
struct OSFactoryArgs {
    name: String,
    #[darling(default)]
    version: Option<String>,
    #[darling(default)]
    description: Option<String>,
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

    let version_gen = args
        .version
        .map_or_else(|| quote! { env!("CARGO_PKG_VERSION") }, |v| quote! { #v });

    let description_gen = args.description.map_or_else(
        || quote! { env!("CARGO_PKG_DESCRIPTION") },
        |d| quote! { #d },
    );

    let connector_descriptor: proc_macro2::TokenStream =
        ["MEMFLOW_CONNECTOR_", &(&connector_name).to_uppercase()]
            .concat()
            .parse()
            .unwrap();

    let func = parse_macro_input!(input as ItemFn);
    let func_name = &func.sig.ident;

    let create_gen = if func.sig.inputs.len() > 1 {
        quote! {
            #[doc(hidden)]
            extern "C" fn mf_create(
                args: &::memflow::types::ReprCStr,
                _: Option<&mut ::std::os::raw::c_void>,
                log_level: i32,
                out: &mut ::memflow::plugins::connector::MUConnectorInstance
            ) -> i32 {
                ::memflow::plugins::connector::create_with_logging(args, log_level, out, #func_name)
            }
        }
    } else {
        quote! {
            #[doc(hidden)]
            extern "C" fn mf_create(
                args: &::memflow::types::ReprCStr,
                _: Option<&mut ::std::os::raw::c_void>,
                _: i32,
                out: &mut ::memflow::plugins::connector::MUConnectorInstance
            ) -> i32 {
                ::memflow::plugins::connector::create_without_logging(args, out, #func_name)
            }
        }
    };

    let gen = quote! {
        #[doc(hidden)]
        #[no_mangle]
        pub static #connector_descriptor: ::memflow::plugins::ConnectorDescriptor = ::memflow::plugins::ConnectorDescriptor {
            plugin_version: ::memflow::plugins::MEMFLOW_PLUGIN_VERSION,
            name: #connector_name,
            version: #version_gen,
            description: #description_gen,
            create: mf_create,
        };

        #create_gen

        #func
    };

    gen.into()
}

#[proc_macro_attribute]
pub fn os_layer(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let args = match OSFactoryArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let os_name = args.name;

    let version_gen = args
        .version
        .map_or_else(|| quote! { env!("CARGO_PKG_VERSION") }, |v| quote! { #v });

    let description_gen = args.description.map_or_else(
        || quote! { env!("CARGO_PKG_DESCRIPTION") },
        |d| quote! { #d },
    );

    let os_descriptor: proc_macro2::TokenStream = ["MEMFLOW_OS_", &(&os_name).to_uppercase()]
        .concat()
        .parse()
        .unwrap();

    let func = parse_macro_input!(input as ItemFn);
    let func_name = &func.sig.ident;

    let create_gen = if func.sig.inputs.len() > 2 {
        quote! {
            #[doc(hidden)]
            extern "C" fn mf_create(
                args: &::memflow::types::ReprCStr,
                mem: ::memflow::plugins::COption<::memflow::plugins::ConnectorInstance>,
                log_level: i32,
                out: &mut ::memflow::plugins::os::MUOSInstance
            ) -> i32 {
                ::memflow::plugins::os::create_with_logging(args, mem.into(), log_level, out, #func_name)
            }
        }
    } else {
        quote! {
            #[doc(hidden)]
            extern "C" fn mf_create(
                args: &::memflow::types::ReprCStr,
                mem: ::memflow::plugins::COption<::memflow::plugins::ConnectorInstance>,
                _: i32,
                out: &mut ::memflow::plugins::os::MUOSInstance
            ) -> i32 {
                ::memflow::plugins::os::create_without_logging(args, mem.into(), out, #func_name)
            }
        }
    };

    let gen = quote! {
        #[doc(hidden)]
        #[no_mangle]
        pub static #os_descriptor: ::memflow::plugins::OSLayerDescriptor = ::memflow::plugins::OSLayerDescriptor {
            os_version: ::memflow::plugins::MEMFLOW_PLUGIN_VERSION,
            name: #os_name,
            version: #version_gen,
            description: #description_gen,
            create: mf_create,
        };

        #create_gen

        #func
    };

    gen.into()
}

#[proc_macro_attribute]
pub fn os_layer_bare(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let args = match OSFactoryArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let os_name = args.name;

    let version_gen = args
        .version
        .map_or_else(|| quote! { env!("CARGO_PKG_VERSION") }, |v| quote! { #v });

    let description_gen = args.description.map_or_else(
        || quote! { env!("CARGO_PKG_DESCRIPTION") },
        |d| quote! { #d },
    );

    let os_descriptor: proc_macro2::TokenStream = ["MEMFLOW_OS_", &(&os_name).to_uppercase()]
        .concat()
        .parse()
        .unwrap();

    let func = parse_macro_input!(input as ItemFn);
    let func_name = &func.sig.ident;
    let create_gen = quote! {
        #[doc(hidden)]
        extern "C" fn mf_create(
            args: &::memflow::types::ReprCStr,
            mem: ::memflow::plugins::COption<::memflow::plugins::ConnectorInstance>,
            log_level: i32,
            out: &mut ::memflow::plugins::os::MUOSInstance
        ) -> i32 {
            ::memflow::plugins::create_bare(args, mem.into(), log_level, out, #func_name)
        }
    };

    let gen = quote! {
        #[doc(hidden)]
        #[no_mangle]
        pub static #os_descriptor: ::memflow::plugins::os::OSDescriptor = ::memflow::plugins::os::OSDescriptor {
            plugin_version: ::memflow::plugins::MEMFLOW_PLUGIN_VERSION,
            name: #os_name,
            version: #version_gen,
            description: #description_gen,
            create: mf_create,
        };

        #create_gen

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
