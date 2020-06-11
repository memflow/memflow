#[macro_use]
mod masks;
use masks::*;

use crate::error::{Error, Result};
use byteorder::{ByteOrder, LittleEndian};

use super::SplitAtIndex;
use crate::architecture::Endianess;
use crate::mem::PhysicalMemory;
use crate::types::{Address, Length, PageType, PhysicalAddress};
use bumpalo::{collections::Vec as BumpVec, Bump};

pub fn bits() -> u8 {
    64
}

pub fn endianess() -> Endianess {
    Endianess::LittleEndian
}

pub fn len_addr() -> Length {
    Length::from(8)
}

fn pml_index_bits(a: u64, level: u32) -> u64 {
    (a & make_bit_mask(
        3 + pt_entries_log2() * level,
        11 + pt_entries_log2() * level,
    )) >> (9 * level)
}

// assume a level 1 (4kb) page for pt reads
fn read_pt_address_iter<T: PhysicalMemory + ?Sized, B>(
    mem: &mut T,
    addrs: &mut BumpVec<(Address, B, Address, [u8; 8])>,
) {
    let page_size = page_size_level(1);

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

pub fn page_size() -> Length {
    page_size_level(1)
}

pub fn page_size_level(pt_level: u32) -> Length {
    //Each PT level up has 512 more entries than the lower level. 512 = 4096 / 8
    Length::from_b(len_addr().as_u64() << (pt_entries_log2() * pt_level))
}

fn get_phys_page(pt_level: u32, pt_addr: Address, virt_addr: Address) -> PhysicalAddress {
    let phys_addr = Address::from(
        (pt_addr.as_u64() & make_bit_mask(3 + pt_entries_log2() * pt_level, 51))
            | (virt_addr.as_u64() & make_bit_mask(0, 2 + pt_entries_log2() * pt_level)),
    );

    PhysicalAddress::with_page(
        phys_addr,
        PageType::from_writeable_bit(is_writeable_page!(pt_addr.as_u64())),
        page_size_level(pt_level),
    )
}

#[allow(clippy::nonminimal_bool)]
pub fn virt_to_phys_iter<T, B, VI, OV>(
    mem: &mut T,
    dtb: Address,
    addrs: VI,
    out: &mut OV,
    arena: &Bump,
) where
    T: PhysicalMemory + ?Sized,
    B: SplitAtIndex,
    VI: Iterator<Item = (Address, B)>,
    OV: Extend<(Result<PhysicalAddress>, Address, B)>,
{
    //TODO: build a tree to eliminate duplicate phys reads with multiple elements
    let mut data_to_translate = BumpVec::new_in(arena);

    data_to_translate.extend(addrs.map(|(addr, buf)| {
        (
            addr,
            buf,
            Address::from(
                (dtb.as_u64() & make_bit_mask(12, 51)) | pml_index_bits(addr.as_u64(), 4),
            ),
            [0; 8],
        )
    }));

    //There are 4 almost identical stages in x64 vtop
    //We just have different error messages
    for (pt_cnt, error_str) in [
        "unable to read pml4e",
        "unable to read pdpte",
        "unable to read pgd",
        "unable to read pte",
    ]
    .iter()
    .enumerate()
    {
        read_pt_address_iter(mem, &mut data_to_translate);

        let pt_level = 4 - pt_cnt as u32;
        let next_page_size = page_size_level(pt_level - 1);

        //Loop through the data in reverse order to allow the data buffer grow on the back when
        //memory regions are split
        for i in (0..data_to_translate.len()).rev() {
            let (addr, buf, pt_addr, tmp_arr) = data_to_translate.swap_remove(i);

            if !check_entry!(pt_addr.as_u64()) {
                //There has been an error in translation, push it to output with the associated buf
                out.extend(Some((Err(Error::new(*error_str)), addr, buf)).into_iter());
            } else if is_final_mapping(pt_level, pt_addr) {
                //We reached an actual page. The translation was successful
                out.extend(
                    Some((Ok(get_phys_page(pt_level, pt_addr, addr)), addr, buf)).into_iter(),
                );
            } else {
                //We still need to continue the page walk

                //As an optimization, divide and conquer the input memory regions.
                //Potential speedups of 4x for up to 2M sequential regions, and 2x for up to 1G sequential regions,
                //assuming all pages are 4kb sized.

                let mut tbuf = Some(buf);
                let mut addr = addr;

                while let Some(buf) = tbuf {
                    let next_addr = (addr + next_page_size).as_page_aligned(next_page_size);

                    let pt_addr = Address::from(
                        (pt_addr.as_u64() & make_bit_mask(12, 51))
                            | pml_index_bits(addr.as_u64(), pt_level - 1),
                    );

                    let (left, next_buf) = buf.split_at(next_addr - addr);

                    data_to_translate.push((addr, left, pt_addr, tmp_arr));

                    addr = next_addr;
                    tbuf = next_buf;
                }
            }
        }

        if data_to_translate.is_empty() {
            break;
        }
    }

    debug_assert!(data_to_translate.is_empty());
}
