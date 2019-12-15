use log::trace;

#[macro_use]
mod masks;
use masks::*;

use byteorder::{ByteOrder, LittleEndian};
use crate::error::{Error, Result};

use crate::address::Address;
use crate::arch::x64;
use crate::mem::PhysicalRead;

// TODO: can we put this in a trait?
fn read_address<T: PhysicalRead>(mem: &mut T, addr: Address) -> Result<Address> {
    let a = mem.phys_read(addr, x64::len_addr())?;
    Ok(Address::from(LittleEndian::read_u64(&a)))
}

pub fn vtop<T: PhysicalRead>(mem: &mut T, dtb: Address, addr: Address) -> Result<Address> {
    let pml4e = read_address(
        mem,
        Address::from((dtb.as_u64() & make_bit_mask(12, 51)) | pml4_index_bits!(addr.as_u64())),
    )?;
    if !check_entry!(pml4e.as_u64()) {
        return Err(Error::new("unable to read pml4e"));
    }

    let pdpte = read_address(
        mem,
        Address::from((pml4e.as_u64() & make_bit_mask(12, 51)) | pdpte_index_bits!(addr.as_u64())),
    )?;
    if !check_entry!(pdpte.as_u64()) {
        return Err(Error::new("unable to read pdpte"));
    }

    if is_large_page!(pdpte.as_u64()) {
        trace!("found 1gb page");
        return Ok(Address::from(
            (pdpte.as_u64() & make_bit_mask(30, 51)) | (addr.as_u64() & make_bit_mask(0, 29)),
        ));
    }

    let pgd = read_address(
        mem,
        Address::from((pdpte.as_u64() & make_bit_mask(12, 51)) | pd_index_bits!(addr.as_u64())),
    )?;
    if !check_entry!(pgd.as_u64()) {
        return Err(Error::new("unable to read pgd"));
    }

    if is_large_page!(pgd.as_u64()) {
        trace!("found 2mb page");
        return Ok(Address::from(
            (pgd.as_u64() & make_bit_mask(21, 51)) | (addr.as_u64() & make_bit_mask(0, 20)),
        ));
    }

    let pte = read_address(
        mem,
        Address::from((pgd.as_u64() & make_bit_mask(12, 51)) | pt_index_bits!(addr.as_u64())),
    )?;
    if !check_entry!(pte.as_u64()) {
        return Err(Error::new("unable to read pte"));
    }

    trace!("found 4kb page");
    Ok(Address::from(
        (pte.as_u64() & make_bit_mask(12, 51)) | (addr.as_u64() & make_bit_mask(0, 11)),
    ))
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[bench]
    fn bench_add_two(b: &mut Bencher) {
        b.iter(|| vtop());
    }
}
*/
