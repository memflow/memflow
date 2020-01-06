extern crate proc_macro;

#[macro_use]
extern crate syn;

extern crate quote;

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(VirtualRead)]
pub fn virtual_read_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let expanded = quote! {
        impl crate::mem::read::VirtualRead for #name {
            fn virt_read(
                &mut self,
                arch: Architecture,
                dtb: Address,
                addr: Address,
                len: Length,
            ) -> Result<Vec<u8>> {
                VatImpl::new(self).virt_read(arch, dtb, addr, len)
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(VirtualWrite)]
pub fn virtual_write_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let expanded = quote! {
        impl crate::mem::write::VirtualWrite for #name {
            fn virt_write(
                &mut self,
                arch: Architecture,
                dtb: Address,
                addr: Address,
                data: &[u8],
            ) -> Result<Length> {
                VatImpl::new(self).virt_write(arch, dtb, addr, data)
            }
        }
    };

    TokenStream::from(expanded)
}
