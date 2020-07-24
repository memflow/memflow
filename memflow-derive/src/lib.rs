use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn};

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
        #[doc(hidden)]
        pub static CONNECTOR_NAME: &str = #connector_name;

        #[doc(hidden)]
        #[no_mangle]
        pub static MEMFLOW_CONNECTOR: memflow_core::connector::ConnectorDescriptor = memflow_core::connector::ConnectorDescriptor {
            connector_version: memflow_core::connector::MEMFLOW_CONNECTOR_VERSION,
            name: CONNECTOR_NAME,
            factory: connector_factory,
        };

        pub extern "C" fn connector_factory(args: &memflow_core::connector::ConnectorArgs) -> memflow_core::error::Result<Box<dyn memflow_core::mem::PhysicalMemory>> {
            let connector = #func_name(args)?;
            Ok(Box::new(connector))
        }

        #func
    };
    gen.into()
}
