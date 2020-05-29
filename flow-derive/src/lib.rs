extern crate proc_macro;

#[macro_use]
extern crate syn;

extern crate quote;

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(VirtualAddressTranslator)]
pub fn virtual_translator_trait_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let (impl_generics, type_generics, _) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics crate::mem::VirtualAddressTranslator for #name #type_generics {
            fn virt_to_phys_iter<B, VI: Iterator<Item = (Address, B)>, OV: Extend<(Result<PhysicalAddress>, Address, B)>> (&mut self, arch: Architecture, dtb: Address, addrs: VI, out: &mut OV) {
                arch.virt_to_phys_iter(self, dtb, addrs, out)
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(AccessVirtualMemory)]
pub fn virtual_memory_trait_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let (impl_generics, type_generics, _) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics crate::mem::AccessVirtualMemory for #name #type_generics {
            fn virt_read_raw_iter<'vtl, VI: crate::mem::virt::VirtualReadIterator<'vtl>>(
                &mut self,
                arch: Architecture,
                dtb: Address,
                iter: VI) -> Result<()> {
                vat::virt_read_raw_iter(self, arch, dtb, iter)
            }

            fn virt_write_raw(
                &mut self,
                arch: Architecture,
                dtb: Address,
                addr: Address,
                data: &[u8],
            ) -> Result<()> {
                vat::virt_write_raw(self, arch, dtb, addr, data)
            }

            fn virt_page_info(
                &mut self,
                arch: Architecture,
                dtb: Address,
                addr: Address
            ) -> Result<Page> {
                vat::virt_page_info(self, arch, dtb, addr)
            }
        }
    };

    TokenStream::from(expanded)
}
