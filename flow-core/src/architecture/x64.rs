#[macro_use]
mod masks;
use masks::*;

use crate::error::{Error, Result};
use byteorder::{ByteOrder, LittleEndian};

use crate::architecture;
use crate::mem::AccessPhysicalMemory;
use crate::types::{Address, Length, Page, PageType, PhysicalAddress};

pub fn bits() -> u8 {
    64
}

pub fn byte_order() -> architecture::ByteOrder {
    architecture::ByteOrder::LittleEndian
}

pub fn page_size() -> Length {
    Length::from_kb(4)
}

pub fn len_addr() -> Length {
    Length::from(8)
}

fn pml_index_bits(a: u64, level: u32) -> u64 {
    (a & make_bit_mask(3 + 9 * level, 11 + 9 * level)) >> (9 * level)
}

// assume a 4kb page-table page for pt reads
fn read_pt_address<T: AccessPhysicalMemory>(mem: &mut T, addr: Address) -> Result<Address> {
    let mut buf = vec![0; len_addr().as_usize()];
    let page_size = Length::from_kb(4);
    mem.phys_read_raw_into(
        PhysicalAddress {
            address: addr,
            page: Some(Page {
                page_type: PageType::PAGE_TABLE,
                page_base: addr.as_page_aligned(page_size),
                page_size,
            }),
        },
        &mut buf,
    )?;
    Ok(Address::from(LittleEndian::read_u64(&buf)))
}

fn read_pt_address_iter<T: AccessPhysicalMemory>(
    mem: &mut T,
    addrs: &mut Vec<(Address, Address, [u8; 8])>,
) {
    let page_size = Length::from_kb(4);
    let _ = mem.phys_read_raw_iter(addrs.iter_mut().map(|(_, tmp_addr, arr)| {
        arr.iter_mut().for_each(|x| *x = 0);
        (
            PhysicalAddress {
                address: *tmp_addr,
                page: Some(Page {
                    page_type: PageType::PAGE_TABLE,
                    page_base: tmp_addr.as_page_aligned(page_size),
                    page_size,
                }),
            },
            &mut arr[..],
        )
    }));
    addrs
        .iter_mut()
        .for_each(|(_, tmp_addr, buf)| *tmp_addr = Address::from(LittleEndian::read_u64(buf)));
}

pub fn virt_to_phys_iter<T: AccessPhysicalMemory, VI: Iterator<Item = Address>>(
    mem: &mut T,
    dtb: Address,
    addrs: VI,
    out: &mut Vec<Result<PhysicalAddress>>,
) -> () {
    //TODO: Optimize this to not use allocs
    let mut data = addrs
        .map(|addr| {
            (
                addr,
                Address::from(
                    (dtb.as_u64() & make_bit_mask(12, 51)) | pml_index_bits(addr.as_u64(), 4),
                ),
                [0; 8],
            )
        })
        .collect::<Vec<_>>();

    for (i, error_str) in [
        "unable to read pml4e",
        "unable to read pdpte",
        "unable to read pgd",
        "unable to read pte",
    ]
    .iter()
    .enumerate()
    {
        read_pt_address_iter(mem, &mut data);
        let pt_level = 4 - i as u32;

        let mut i = 0;
        while let Some((addr, tmp_addr, _)) = data.get_mut(i) {
            if !check_entry!(tmp_addr.as_u64()) {
                data.swap_remove(i);
                out.push(Err(Error::new(*error_str)));
            } else if (pt_level != 4 && is_large_page!(tmp_addr.as_u64())) || pt_level == 1 {
                let (addr, tmp_addr, _) = data.swap_remove(i);

                let phys_addr = Address::from(
                    (tmp_addr.as_u64() & make_bit_mask(3 + 9 * pt_level, 51))
                        | (addr.as_u64() & make_bit_mask(0, 2 + 9 * pt_level)),
                );
                let page_size = Length::from_b(8 << (9 * pt_level));
                out.push(Ok(PhysicalAddress {
                    address: phys_addr,
                    page: Some(Page {
                        page_type: PageType::from_writeable_bit(is_writeable_page!(
                            tmp_addr.as_u64()
                        )),
                        page_base: phys_addr.as_page_aligned(page_size),
                        page_size,
                    }),
                }));
            } else {
                i += 1;
                *tmp_addr = Address::from(
                    (tmp_addr.as_u64() & make_bit_mask(12, 51))
                        | pml_index_bits(addr.as_u64(), pt_level - 1),
                );
            }
        }

        if data.is_empty() {
            break;
        }
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[bench]
    fn bench_add_two(b: &mut Bencher) {
        b.iter(|| virt_to_phys());
    }
}
*/
