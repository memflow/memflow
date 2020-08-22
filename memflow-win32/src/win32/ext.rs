use memflow_core::architecture::x86;
use memflow_core::architecture::{AddressTranslator, Architecture};
use memflow_core::mem::{
    CloneableVirtualMemory, PhysicalMemory, VirtualFromPhysical, VirtualMemory, VirtualMemoryBox,
    VirtualTranslate,
};
use memflow_core::types::Address;
use std::ptr;

pub fn make_virt_mem_clone<
    T: PhysicalMemory + Clone + 'static,
    V: VirtualTranslate + Clone + 'static,
>(
    mem: T,
    vat: V,
    proc_arch: &'static dyn Architecture,
    sys_arch: &'static dyn Architecture,
    dtb: Address,
) -> VirtualMemoryBox {
    Box::new(VirtualFromPhysical::with_vat(
        mem,
        proc_arch,
        x86::new_translator(dtb, sys_arch).unwrap(),
        vat,
    ))
}

pub fn make_virt_mem<'a, T: PhysicalMemory + 'a, V: VirtualTranslate + 'a>(
    mem: T,
    vat: V,
    proc_arch: &'static dyn Architecture,
    sys_arch: &'static dyn Architecture,
    dtb: Address,
) -> Box<dyn VirtualMemory + 'a> {
    Box::new(VirtualFromPhysical::with_vat(
        mem,
        proc_arch,
        x86::new_translator(dtb, sys_arch).unwrap(),
        vat,
    ))
}
