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

#[proc_macro_attribute]
pub fn connector(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let args = match ConnectorFactoryArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let connector_name = args.name;

    let input = parse_macro_input!(input as ItemFn);
    let func_name = &input.sig.ident;

    let gen = quote! {
        #[doc(hidden)]
        pub static CONNECTOR_NAME: &str = #connector_name;

        #[doc(hidden)]
        #[no_mangle]
        pub static CONNECTOR_DECLARATION: memflow_core::ConnectorPlugin = memflow_core::ConnectorPlugin {
            memflow_plugin_version: memflow_core::MEMFLOW_PLUGIN_VERSION,
            name: CONNECTOR_NAME,
            factory: connector_factory,
        };

        pub extern "C" fn connector_factory(args: &str) -> memflow_core::Result<Box<dyn memflow_core::PhysicalMemory>> {
            let connector = #func_name(args)?;
            Ok(Box::new(connector))
        }

        #input
    };
    gen.into()
}
