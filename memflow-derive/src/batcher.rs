use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, DeriveInput, Expr, Lit, Meta,
};

struct MemflowBatcherAttribute {
    offset: u32,
}

impl Parse for MemflowBatcherAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let nested_meta: Meta = input.parse()?;

        if let Meta::NameValue(name_value) = nested_meta {
            if name_value.path.is_ident("offset") {
                if let Expr::Lit(lit) = name_value.value {
                    if let Lit::Int(int) = lit.lit {
                        return Ok(MemflowBatcherAttribute {
                            offset: int.base10_parse().unwrap(),
                        });
                    }
                }
            }
        }

        panic!("No offset found in #[memflow(...)] attribute");
    }
}

pub fn batcher_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_ident = input.ident;
    let fields = match input.data {
        syn::Data::Struct(ref data_struct) => &data_struct.fields,
        _ => panic!("Batcher can only be used on structs"),
    };

    let mut batch_fields = Vec::new();
    for field in fields.iter() {
        let field_name = &field.ident;

        for attr in &field.attrs {
            if let Meta::List(meta_list) = &attr.meta {
                if let Ok(res) = meta_list.parse_args::<MemflowBatcherAttribute>() {
                    let offset = res.offset;
                    batch_fields.push(quote! {
                        batcher.read_into(
                          address + #offset,
                          &mut self.#field_name,
                        );
                    });
                }
            }
        }
    }

    TokenStream::from(quote! {
        impl #struct_ident {
            fn read_all_batched(&mut self, mut view: impl memflow::prelude::MemoryView, address: memflow::prelude::Address) {
                let mut batcher = view.batcher();
                #(#batch_fields)*
            }
        }
    })
}
