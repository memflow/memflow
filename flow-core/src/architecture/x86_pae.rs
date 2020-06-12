use crate::error::{Error, Result};

use crate::architecture::Endianess;
use crate::types::{Address, Length, PhysicalAddress};

use crate::mem::PhysicalMemory;

use log::warn;

pub fn bits() -> u8 {
    32
}

pub fn endianess() -> Endianess {
    Endianess::LittleEndian
}

pub fn page_size() -> Length {
    Length::from_kb(4)
}

pub fn page_size_level(pt_level: u32) -> Length {
    match pt_level {
        1 => Length::from_kb(4),
        2 => Length::from_mb(2),
        _ => panic!(
            "non existent page table level '{}' for architecture x86 (pae mode)",
            pt_level
        ),
    }
}

pub fn len_addr() -> Length {
    Length::from(4)
}

// https://github.com/libvmi/libvmi/blob/master/libvmi/arch/intel.c#L327
pub fn virt_to_phys_iter<T, B, VI, OV>(_mem: &mut T, _dtb: Address, addrs: VI, out: &mut OV)
where
    T: PhysicalMemory + ?Sized,
    VI: Iterator<Item = (Address, B)>,
    OV: Extend<(Result<PhysicalAddress>, Address, B)>,
{
    warn!("x86_pae::virt_to_phys_iter() not implemented yet");
    out.extend(addrs.map(|(addr, buf)| {
        (
            // get pdpi
            // get pgd -> check 2mb page
            // get pte -> check 4kb page
            Err(Error::new(
                "x86_pae::virt_to_phys_iter() not implemented yet",
            )),
            addr,
            buf,
        )
    }));
}
