mod masks;

use flow_core::mem::PhysicalMemory;

// TODO: how do we abstract different architectures?
pub trait VirtualAddressTranslation64 {
    fn virt_to_phys(cr3: u64, addr: u64) -> u64;
}

// TODO: add architecture agnostic trait
impl<T: PhysicalMemory> VirtualAddressTranslation64 for T {
    fn virt_to_phys(cr3: u64, addr: u64) -> u64 {
        let mask = masks::make_bit_mask(12, 51);
        0
    }
}
