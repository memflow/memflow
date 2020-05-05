#[macro_use]
mod masks;
use masks::*;

use crate::error::{Error, Result};
use byteorder::{ByteOrder, LittleEndian};

use crate::address::{Address, Length, Page, PageType, PhysicalAddress};
use crate::arch;
use crate::mem::AccessPhysicalMemory;

pub fn bits() -> u8 {
    64
}

pub fn byte_order() -> arch::ByteOrder {
    arch::ByteOrder::LittleEndian
}

pub fn page_size() -> Length {
    Length::from_kb(4)
}

pub fn len_addr() -> Length {
    Length::from(8)
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

#[allow(clippy::nonminimal_bool)]
pub fn virt_to_phys<T: AccessPhysicalMemory>(
    mem: &mut T,
    dtb: Address,
    addr: Address,
) -> Result<PhysicalAddress> {
    let pml4e = read_pt_address(
        mem,
        Address::from((dtb.as_u64() & make_bit_mask(12, 51)) | pml4_index_bits!(addr.as_u64())),
    )?;
    if !check_entry!(pml4e.as_u64()) {
        return Err(Error::new("unable to read pml4e"));
    }

    let pdpte = read_pt_address(
        mem,
        Address::from((pml4e.as_u64() & make_bit_mask(12, 51)) | pdpte_index_bits!(addr.as_u64())),
    )?;
    if !check_entry!(pdpte.as_u64()) {
        return Err(Error::new("unable to read pdpte"));
    }

    if is_large_page!(pdpte.as_u64()) {
        //trace!("found 1gb page");
        let phys_addr = Address::from(
            (pdpte.as_u64() & make_bit_mask(30, 51)) | (addr.as_u64() & make_bit_mask(0, 29)),
        );
        let page_size = Length::from_gb(1);
        return Ok(PhysicalAddress {
            address: phys_addr,
            page: Some(Page {
                page_type: PageType::from_writeable_bit(is_writeable_page!(pdpte.as_u64())),
                page_base: phys_addr.as_page_aligned(page_size),
                page_size,
            }),
        });
    }

    let pgd = read_pt_address(
        mem,
        Address::from((pdpte.as_u64() & make_bit_mask(12, 51)) | pd_index_bits!(addr.as_u64())),
    )?;
    if !check_entry!(pgd.as_u64()) {
        return Err(Error::new("unable to read pgd"));
    }

    if is_large_page!(pgd.as_u64()) {
        //trace!("found 2mb page");
        let phys_addr = Address::from(
            (pgd.as_u64() & make_bit_mask(21, 51)) | (addr.as_u64() & make_bit_mask(0, 20)),
        );
        let page_size = Length::from_mb(2);
        return Ok(PhysicalAddress {
            address: phys_addr,
            page: Some(Page {
                page_type: PageType::from_writeable_bit(is_writeable_page!(pgd.as_u64())),
                page_base: phys_addr.as_page_aligned(page_size),
                page_size,
            }),
        });
    }

    let pte = read_pt_address(
        mem,
        Address::from((pgd.as_u64() & make_bit_mask(12, 51)) | pt_index_bits!(addr.as_u64())),
    )?;
    if !check_entry!(pte.as_u64()) {
        return Err(Error::new("unable to read pte"));
    }

    //trace!("found 4kb page");
    let phys_addr = Address::from(
        (pte.as_u64() & make_bit_mask(12, 51)) | (addr.as_u64() & make_bit_mask(0, 11)),
    );
    let page_size = Length::from_kb(4);
    Ok(PhysicalAddress {
        address: phys_addr,
        page: Some(Page {
            page_type: PageType::from_writeable_bit(is_writeable_page!(pte.as_u64())),
            page_base: phys_addr.as_page_aligned(page_size),
            page_size,
        }),
    })
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
