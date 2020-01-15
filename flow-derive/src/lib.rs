extern crate proc_macro;

#[macro_use]
extern crate syn;

extern crate quote;

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(VirtualMemoryTrait)]
pub fn virtual_memory_trait_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let expanded = quote! {
        impl crate::mem::VirtualMemoryTrait for #name {
            fn virt_read(
                &mut self,
                arch: Architecture,
                dtb: Address,
                addr: Address,
                out: &mut [u8],
            ) -> Result<()> {
                VatImpl::new(self).virt_read(arch, dtb, addr, out)
            }

            fn virt_write(
                &mut self,
                arch: Architecture,
                dtb: Address,
                addr: Address,
                data: &[u8],
            ) -> Result<()> {
                VatImpl::new(self).virt_write(arch, dtb, addr, data)
            }
        }
    };

    TokenStream::from(expanded)
}
