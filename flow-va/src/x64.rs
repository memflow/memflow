#[macro_use]
mod masks;
use masks::*;

use std::io::{Error, ErrorKind, Result};
use byteorder::{ByteOrder, LittleEndian};

use arch::x64;
use address::Address;
use mem::PhysicalRead;

// TODO: can we put this in a trait?
fn read_address<T: PhysicalRead>(mem: &mut T, addr: Address) -> Result<Address> {
	let a = mem.phys_read(addr, x64::len_addr())?;
	Ok(Address::from(LittleEndian::read_u64(&a)))
}

pub fn vtop<T: PhysicalRead>(mem: &mut T, dtb: Address, addr: Address) -> Result<Address> {
	let pml4e = read_address(mem,
		Address::from((dtb.addr & make_bit_mask(12, 51)) | pml4_index_bits!(addr.addr)))?;
	if !check_entry!(pml4e.addr) {
		return Err(Error::new(ErrorKind::Other, "unable to read pml4e"));
	}

	let pdpte = read_address(mem, 
		Address::from((pml4e.addr & make_bit_mask(12, 51)) | pdpte_index_bits!(addr.addr)))?;
	if !check_entry!(pdpte.addr) {
		return Err(Error::new(ErrorKind::Other, "unable to read pdpte"));
	}

	if is_large_page!(pdpte.addr) {
		println!("found 1gb page");
		return Ok(Address::from((pdpte.addr & make_bit_mask(30, 51)) | (addr.addr & make_bit_mask(0, 29))));
	}

	let pgd = read_address(mem, 
		Address::from((pdpte.addr & make_bit_mask(12, 51)) | pd_index_bits!(addr.addr)))?;
	if !check_entry!(pgd.addr) {
		return Err(Error::new(ErrorKind::Other, "unable to read pgd"));
	}

	if is_large_page!(pgd.addr) {
		println!("found 2mb page");
		return Ok(Address::from((pgd.addr & make_bit_mask(21, 51)) | (addr.addr & make_bit_mask(0, 20))));
	}

	let pte = read_address(mem,
		Address::from((pgd.addr & make_bit_mask(12, 51)) | pt_index_bits!(addr.addr)))?;
	if !check_entry!(pte.addr) {
		return Err(Error::new(ErrorKind::Other, "unable to read pte"));
	}

	println!("found 4kb page");
	return Ok(Address::from((pte.addr & make_bit_mask(12, 51)) | (addr.addr & make_bit_mask(0, 11))));
}
