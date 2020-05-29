#[macro_use]
mod masks;
use masks::*;

use crate::error::{Error, Result};
use byteorder::{ByteOrder, LittleEndian};

use crate::architecture;
use crate::mem::AccessPhysicalMemory;
use crate::types::{Address, Length, PageType, PhysicalAddress};

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
fn read_pt_address_iter<T: AccessPhysicalMemory, B>(
    mem: &mut T,
    addrs: &mut Vec<(Address, B, Address, [u8; 8])>,
) {
    let page_size = Length::from_kb(4);
    let _ = mem.phys_read_iter(addrs.iter_mut().map(|(_, _, pt_addr, arr)| {
        arr.iter_mut().for_each(|x| *x = 0);
        (
            PhysicalAddress::with_page(*pt_addr, PageType::PAGE_TABLE, page_size),
            &mut arr[..],
        )
    }));
    addrs
        .iter_mut()
        .for_each(|(_, _, pt_addr, buf)| *pt_addr = Address::from(LittleEndian::read_u64(buf)));
}

fn is_final_mapping(pt_level: u32, pt_addr: Address) -> bool {
    pt_level == 1 || (is_large_page!(pt_addr.as_u64()) && pt_level != 4)
}

const fn pt_entries_log2() -> u32 {
    9
}

fn get_phys_page(pt_level: u32, pt_addr: Address, virt_addr: Address) -> PhysicalAddress {
    let phys_addr = Address::from(
        (pt_addr.as_u64() & make_bit_mask(3 + pt_entries_log2() * pt_level, 51))
            | (virt_addr.as_u64() & make_bit_mask(0, 2 + pt_entries_log2() * pt_level)),
    );
    let page_size = Length::from_b(len_addr().as_u64() << (pt_entries_log2() * pt_level));

    PhysicalAddress::with_page(
        phys_addr,
        PageType::from_writeable_bit(is_writeable_page!(pt_addr.as_u64())),
        page_size,
    )
}

pub fn virt_to_phys_iter<
    T: AccessPhysicalMemory,
    B,
    VI: Iterator<Item = (Address, B)>,
    OV: Extend<(Result<PhysicalAddress>, Address, B)>,
>(
    mem: &mut T,
    dtb: Address,
    addrs: VI,
    out: &mut OV,
) {
    //TODO: Optimize this to not use allocs
    let mut data = addrs
        .map(|(addr, buf)| {
            (
                addr,
                buf,
                Address::from(
                    (dtb.as_u64() & make_bit_mask(12, 51)) | pml_index_bits(addr.as_u64(), 4),
                ),
                [0; 8],
            )
        })
        .collect::<Vec<_>>();

    for (pt_cnt, error_str) in [
        "unable to read pml4e",
        "unable to read pdpte",
        "unable to read pgd",
        "unable to read pte",
    ]
    .iter()
    .enumerate()
    {
        read_pt_address_iter(mem, &mut data);
        let pt_level = 4 - pt_cnt as u32;

        let mut i = 0;
        //Possibly make this iterator based? Call in out.extend with some sort of filtering
        while let Some((addr, _, pt_addr, _)) = data.get_mut(i) {
            if !check_entry!(pt_addr.as_u64()) {
                let (addr, buf, _, _) = data.swap_remove(i);
                out.extend(Some((Err(Error::new(*error_str)), addr, buf)).into_iter());
            } else if is_final_mapping(pt_level, *pt_addr) {
                let (addr, buf, pt_addr, _) = data.swap_remove(i);

                out.extend(
                    Some((Ok(get_phys_page(pt_level, pt_addr, addr)), addr, buf)).into_iter(),
                );
            } else {
                i += 1;
                *pt_addr = Address::from(
                    (pt_addr.as_u64() & make_bit_mask(12, 51))
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
