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

pub fn virt_to_phys_iter<
    T: AccessPhysicalMemory,
    B,
    VI: Iterator<Item = (Address, B)>,
    OV: Extend<(Result<PhysicalAddress>, Address, B)>,
>(
    _mem: &mut T,
    _dtb: Address,
    addrs: VI,
    out: &mut OV,
) -> () {
    warn!("x86::virt_to_phys_iter() not implemented yet");
    out.extend(addrs.map(|(addr, buf)| {
        (
            Err(Error::new("x86::virt_to_phys_iter() not implemented yet")),
            addr,
            buf,
        )
    }));
}
