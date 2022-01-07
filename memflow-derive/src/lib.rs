use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro_crate::*;
use quote::{format_ident, quote};
use syn::{parse_macro_input, AttributeArgs, Data, DeriveInput, Fields, ItemFn};

#[derive(Debug, FromMeta)]
struct ConnectorFactoryArgs {
    name: String,
    #[darling(default)]
    version: Option<String>,
    #[darling(default)]
    description: Option<String>,
    #[darling(default)]
    help_fn: Option<String>,
    #[darling(default)]
    target_list_fn: Option<String>,
}

#[derive(Debug, FromMeta)]
struct OsFactoryArgs {
    name: String,
    #[darling(default)]
    version: Option<String>,
    #[darling(default)]
    description: Option<String>,
    #[darling(default)]
    help_fn: Option<String>,
}

fn validate_plugin_name(name: &str) {
    if !name.chars().all(char::is_alphanumeric) {
        panic!("plugin name must only contain alphanumeric characters");
    }
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
    let crate_path = crate_path();

    let attr_args = parse_macro_input!(args as AttributeArgs);
    let args = match ConnectorFactoryArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let connector_name = args.name;
    validate_plugin_name(&connector_name);

    let version_gen = args
        .version
        .map_or_else(|| quote! { env!("CARGO_PKG_VERSION") }, |v| quote! { #v });

    let description_gen = args.description.map_or_else(
        || quote! { env!("CARGO_PKG_DESCRIPTION") },
        |d| quote! { #d },
    );

    let help_gen = if args.help_fn.is_some() {
        quote! { Some(mf_help_callback) }
    } else {
        quote! { None }
    };

    let target_list_gen = if args.target_list_fn.is_some() {
        quote! { Some(mf_target_list_callback) }
    } else {
        quote! { None }
    };

    let connector_descriptor: proc_macro2::TokenStream =
        ["MEMFLOW_CONNECTOR_", &(&connector_name).to_uppercase()]
            .concat()
            .parse()
            .unwrap();

    let func = parse_macro_input!(input as ItemFn);
    let func_name = &func.sig.ident;

    let create_fn_gen = quote! {
            #[doc(hidden)]
            extern "C" fn mf_create(
                args: Option<&#crate_path::plugins::connector::ConnectorArgs>,
                _: cglue::option::COption<#crate_path::plugins::os::OsInstanceArcBox>,
                lib: #crate_path::plugins::LibArc,
                logger: Option<&'static #crate_path::plugins::PluginLogger>,
                out: &mut #crate_path::plugins::connector::MuConnectorInstanceArcBox<'static>
            ) -> i32 {
                #crate_path::plugins::connector::create(args, lib, logger, out, #func_name)
            }
    };

    let help_fn_gen = args.help_fn.map(|v| v.parse().unwrap()).map_or_else(
        proc_macro2::TokenStream::new,
        |func_name: proc_macro2::TokenStream| {
            quote! {
                #[doc(hidden)]
                extern "C" fn mf_help_callback(
                    mut callback: #crate_path::plugins::HelpCallback,
                ) {
                    let helpstr = #func_name();
                    let _ = callback.call(helpstr.into());
                }
            }
        },
    );

    let target_list_fn_gen = args.target_list_fn.map(|v| v.parse().unwrap()).map_or_else(
        proc_macro2::TokenStream::new,
        |func_name: proc_macro2::TokenStream| {
            quote! {
                #[doc(hidden)]
                extern "C" fn mf_target_list_callback(
                    mut callback: #crate_path::plugins::TargetCallback,
                ) -> i32 {
                    #func_name()
                        .map(|mut targets| {
                            targets
                                .into_iter()
                                .take_while(|t| callback.call(t.clone()))
                                .for_each(|_| ());
                        })
                        .into_int_result()
                }
            }
        },
    );

    let gen = quote! {
        #[doc(hidden)]
        #[no_mangle]
        pub static #connector_descriptor: #crate_path::plugins::ConnectorDescriptor = #crate_path::plugins::ConnectorDescriptor {
            plugin_version: #crate_path::plugins::MEMFLOW_PLUGIN_VERSION,
            input_layout: <<#crate_path::plugins::LoadableConnector as #crate_path::plugins::Loadable>::CInputArg as #crate_path::abi_stable::StableAbi>::LAYOUT,
            output_layout: <<#crate_path::plugins::LoadableConnector as #crate_path::plugins::Loadable>::Instance as #crate_path::abi_stable::StableAbi>::LAYOUT,
            name: #crate_path::cglue::CSliceRef::from_str(#connector_name),
            version: #crate_path::cglue::CSliceRef::from_str(#version_gen),
            description: #crate_path::cglue::CSliceRef::from_str(#description_gen),
            help_callback: #help_gen,
            target_list_callback: #target_list_gen,
            create: mf_create,
        };

        #create_fn_gen

        #help_fn_gen

        #target_list_fn_gen

        #func
    };

    gen.into()
}

#[proc_macro_attribute]
pub fn connector_bare(args: TokenStream, input: TokenStream) -> TokenStream {
    let crate_path = crate_path();

    let attr_args = parse_macro_input!(args as AttributeArgs);
    let args = match ConnectorFactoryArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let connector_name = args.name;
    validate_plugin_name(&connector_name);

    let version_gen = args
        .version
        .map_or_else(|| quote! { env!("CARGO_PKG_VERSION") }, |v| quote! { #v });

    let description_gen = args.description.map_or_else(
        || quote! { env!("CARGO_PKG_DESCRIPTION") },
        |d| quote! { #d },
    );

    let help_gen = if args.help_fn.is_some() {
        quote! { Some(mf_help_callback) }
    } else {
        quote! { None }
    };

    let target_list_gen = if args.target_list_fn.is_some() {
        quote! { Some(mf_target_list_callback) }
    } else {
        quote! { None }
    };

    let connector_descriptor: proc_macro2::TokenStream =
        ["MEMFLOW_CONNECTOR_", &(&connector_name).to_uppercase()]
            .concat()
            .parse()
            .unwrap();

    let func = parse_macro_input!(input as ItemFn);
    let func_name = &func.sig.ident;

    let create_fn_gen = quote! {
            #[doc(hidden)]
            extern "C" fn mf_create(
                args: Option<&#crate_path::plugins::connector::ConnectorArgs>,
                os: cglue::option::COption<#crate_path::plugins::os::OsInstanceArcBox<'static>>,
                lib: #crate_path::plugins::LibArc,
                logger: Option<&'static #crate_path::plugins::PluginLogger>,
                out: &mut #crate_path::plugins::connector::MuConnectorInstanceArcBox<'static>
            ) -> i32 {
                #crate_path::plugins::create_bare(args, os.into(), lib, logger, out, #func_name)
            }
    };

    let help_fn_gen = args.help_fn.map(|v| v.parse().unwrap()).map_or_else(
        proc_macro2::TokenStream::new,
        |func_name: proc_macro2::TokenStream| {
            quote! {
                #[doc(hidden)]
                extern "C" fn mf_help_callback(
                    mut callback: #crate_path::plugins::HelpCallback,
                ) {
                    let helpstr = #func_name();
                    let _ = callback.call(helpstr.into());
                }
            }
        },
    );

    let target_list_fn_gen = args.target_list_fn.map(|v| v.parse().unwrap()).map_or_else(
        proc_macro2::TokenStream::new,
        |func_name: proc_macro2::TokenStream| {
            quote! {
                #[doc(hidden)]
                extern "C" fn mf_target_list_callback(
                    mut callback: #crate_path::plugins::TargetCallback,
                ) -> i32 {
                    #func_name()
                        .map(|mut targets| {
                            targets
                                .into_iter()
                                .take_while(|t| callback.call(t.clone()))
                                .for_each(|_| ());
                        })
                        .into_int_result()
                }
            }
        },
    );

    let gen = quote! {
        #[doc(hidden)]
        #[no_mangle]
        pub static #connector_descriptor: #crate_path::plugins::ConnectorDescriptor = #crate_path::plugins::ConnectorDescriptor {
            plugin_version: #crate_path::plugins::MEMFLOW_PLUGIN_VERSION,
            input_layout: <<#crate_path::plugins::LoadableConnector as #crate_path::plugins::Loadable>::CInputArg as #crate_path::abi_stable::StableAbi>::LAYOUT,
            output_layout: <<#crate_path::plugins::LoadableConnector as #crate_path::plugins::Loadable>::Instance as #crate_path::abi_stable::StableAbi>::LAYOUT,
            name: #crate_path::cglue::CSliceRef::from_str(#connector_name),
            version: #crate_path::cglue::CSliceRef::from_str(#version_gen),
            description: #crate_path::cglue::CSliceRef::from_str(#description_gen),
            help_callback: #help_gen,
            target_list_callback: #target_list_gen,
            create: mf_create,
        };

        #create_fn_gen

        #help_fn_gen

        #target_list_fn_gen

        #func
    };

    gen.into()
}

#[proc_macro_attribute]
pub fn os_layer(args: TokenStream, input: TokenStream) -> TokenStream {
    let crate_path = crate_path();

    let attr_args = parse_macro_input!(args as AttributeArgs);
    let args = match OsFactoryArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let os_name = args.name;
    validate_plugin_name(&os_name);

    let version_gen = args
        .version
        .map_or_else(|| quote! { env!("CARGO_PKG_VERSION") }, |v| quote! { #v });

    let description_gen = args.description.map_or_else(
        || quote! { env!("CARGO_PKG_DESCRIPTION") },
        |d| quote! { #d },
    );

    let help_gen = if args.help_fn.is_some() {
        quote! { Some(mf_help_callback) }
    } else {
        quote! { None }
    };

    let os_descriptor: proc_macro2::TokenStream = ["MEMFLOW_OS_", &(&os_name).to_uppercase()]
        .concat()
        .parse()
        .unwrap();

    let func = parse_macro_input!(input as ItemFn);
    let func_name = &func.sig.ident;

    let create_fn_gen = quote! {
            #[doc(hidden)]
            extern "C" fn mf_create(
                args: Option<&#crate_path::plugins::os::OsArgs>,
                mem: #crate_path::cglue::COption<#crate_path::plugins::connector::ConnectorInstanceArcBox<'static>>,
                lib: #crate_path::plugins::LibArc,
                logger: Option<&'static #crate_path::plugins::PluginLogger>,
                out: &mut #crate_path::plugins::os::MuOsInstanceArcBox<'static>
            ) -> i32 {
                #crate_path::plugins::os::create(args, mem.into(), lib, logger, out, #func_name)
            }
    };

    let help_fn_gen = args.help_fn.map(|v| v.parse().unwrap()).map_or_else(
        proc_macro2::TokenStream::new,
        |func_name: proc_macro2::TokenStream| {
            quote! {
                #[doc(hidden)]
                extern "C" fn mf_help_callback(
                    mut callback: #crate_path::plugins::HelpCallback,
                ) {
                    let helpstr = #func_name();
                    let _ = callback.call(helpstr.into());
                }
            }
        },
    );

    let gen = quote! {
        #[doc(hidden)]
        #[no_mangle]
        pub static #os_descriptor: #crate_path::plugins::OsLayerDescriptor = #crate_path::plugins::OsLayerDescriptor {
            os_version: #crate_path::plugins::MEMFLOW_PLUGIN_VERSION,
            name: #os_name,
            version: #version_gen,
            description: #description_gen,
            help_callback: #help_gen,
            target_list_callback: None, // non existent on Os Plugins
            create: mf_create,
        };

        #create_fn_gen

        #help_fn_gen

        #func
    };

    gen.into()
}

#[proc_macro_attribute]
pub fn os_layer_bare(args: TokenStream, input: TokenStream) -> TokenStream {
    let crate_path = crate_path();

    let attr_args = parse_macro_input!(args as AttributeArgs);
    let args = match OsFactoryArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let os_name = args.name;
    validate_plugin_name(&os_name);

    let version_gen = args
        .version
        .map_or_else(|| quote! { env!("CARGO_PKG_VERSION") }, |v| quote! { #v });

    let description_gen = args.description.map_or_else(
        || quote! { env!("CARGO_PKG_DESCRIPTION") },
        |d| quote! { #d },
    );

    let help_gen = if args.help_fn.is_some() {
        quote! { Some(mf_help_callback) }
    } else {
        quote! { None }
    };

    let os_descriptor: proc_macro2::TokenStream = ["MEMFLOW_OS_", &(&os_name).to_uppercase()]
        .concat()
        .parse()
        .unwrap();

    let func = parse_macro_input!(input as ItemFn);
    let func_name = &func.sig.ident;

    let create_fn_gen = quote! {
        #[doc(hidden)]
        extern "C" fn mf_create(
            args: Option<&#crate_path::plugins::os::OsArgs>,
            mem: #crate_path::cglue::COption<#crate_path::plugins::connector::ConnectorInstanceArcBox<'static>>,
            lib: #crate_path::plugins::LibArc,
            logger: Option<&'static #crate_path::plugins::PluginLogger>,
            out: &mut #crate_path::plugins::os::MuOsInstanceArcBox<'static>
        ) -> i32 {
            #crate_path::plugins::create_bare(args, mem.into(), lib, logger, out, #func_name)
        }
    };

    let help_fn_gen = args.help_fn.map(|v| v.parse().unwrap()).map_or_else(
        proc_macro2::TokenStream::new,
        |func_name: proc_macro2::TokenStream| {
            quote! {
                #[doc(hidden)]
                extern "C" fn mf_help_callback(
                    mut callback: #crate_path::plugins::HelpCallback,
                ) {
                    let helpstr = #func_name();
                    let _ = callback.call(helpstr.into());
                }
            }
        },
    );

    let gen = quote! {
        #[doc(hidden)]
        #[no_mangle]
        pub static #os_descriptor: #crate_path::plugins::os::OsDescriptor = #crate_path::plugins::os::OsDescriptor {
            plugin_version: #crate_path::plugins::MEMFLOW_PLUGIN_VERSION,
            input_layout: <<#crate_path::plugins::os::LoadableOs as #crate_path::plugins::Loadable>::CInputArg as #crate_path::abi_stable::StableAbi>::LAYOUT,
            output_layout: <<#crate_path::plugins::os::LoadableOs as #crate_path::plugins::Loadable>::Instance as #crate_path::abi_stable::StableAbi>::LAYOUT,
            name: #crate_path::cglue::CSliceRef::from_str(#os_name),
            version: #crate_path::cglue::CSliceRef::from_str(#version_gen),
            description: #crate_path::cglue::CSliceRef::from_str(#description_gen),
            help_callback: #help_gen,
            target_list_callback: None, // non existent on Os Plugins
            create: mf_create,
        };

        #create_fn_gen

        #help_fn_gen

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
    let crate_path = crate_path();

    format!("{}::dataview::derive_pod!{{ {} }}", crate_path, input)
        .parse()
        .unwrap()
}

#[proc_macro_derive(ByteSwap)]
pub fn byteswap_derive(input: TokenStream) -> TokenStream {
    let crate_path = crate_path();

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
        impl #impl_generics #crate_path::types::byte_swap::ByteSwap for #name #ty_generics #where_clause {
            fn byte_swap(&mut self) {
                #gen_inner
            }
        }
    );

    gen.into()
}

fn crate_path() -> proc_macro2::TokenStream {
    let (col, ident) = crate_path_ident();
    quote!(#col #ident)
}

fn crate_path_ident() -> (Option<syn::token::Colon2>, proc_macro2::Ident) {
    match crate_path_fixed() {
        FoundCrate::Itself => (None, format_ident!("crate")),
        FoundCrate::Name(name) => (Some(Default::default()), format_ident!("{}", name)),
    }
}

fn crate_path_fixed() -> FoundCrate {
    let found_crate = crate_name("memflow").expect("memflow found in `Cargo.toml`");

    match found_crate {
        FoundCrate::Itself => {
            let has_doc_env = std::env::vars().any(|(k, _)| {
                k == "UNSTABLE_RUSTDOC_TEST_LINE" || k == "UNSTABLE_RUSTDOC_TEST_PATH"
            });

            if has_doc_env {
                FoundCrate::Name("memflow".to_string())
            } else {
                FoundCrate::Itself
            }
        }
        x => x,
    }
}
