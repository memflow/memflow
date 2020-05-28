use crate::error::{Error, Result};

use crate::architecture::ByteOrder;
use crate::types::{Address, Length, PhysicalAddress};

use crate::mem::AccessPhysicalMemory;

use log::warn;

pub fn bits() -> u8 {
    32
}

pub fn byte_order() -> ByteOrder {
    ByteOrder::LittleEndian
}

pub fn page_size() -> Length {
    Length::from_kb(4)
}

pub fn len_addr() -> Length {
    Length::from(4)
}

pub fn virt_to_phys_iter<T: AccessPhysicalMemory, VI: Iterator<Item = Address>>(
    _mem: &mut T,
    _dtb: Address,
    addrs: VI,
    out: &mut Vec<Result<PhysicalAddress>>,
) -> () {
    warn!("x86::virt_to_phys_iter() not implemented yet");
    addrs.for_each(|_| {
        out.push(Err(Error::new(
            "x86::virt_to_phys_iter() not implemented yet",
        )))
    })
}
