use memflow_core::architecture::x86;
use memflow_core::architecture::{AddressTranslator, Architecture};
use memflow_core::mem::{PhysicalMemory, VirtualFromPhysical, VirtualMemory, VirtualTranslate};
use memflow_core::types::Address;
use std::ptr;

pub fn make_virt_mem<'a, T: PhysicalMemory + 'a, V: VirtualTranslate + 'a>(
    mem: T,
    vat: V,
    proc_arch: &'static dyn Architecture,
    sys_arch: &'static dyn Architecture,
    dtb: Address,
) -> Box<dyn VirtualMemory + 'a> {
    if ptr::eq(sys_arch, x86::x64::ARCH) {
        Box::new(VirtualFromPhysical::with_vat(
            mem,
            proc_arch,
            x86::x64::new_translator(dtb),
            vat,
        ))
    } else if ptr::eq(sys_arch, x86::x32::ARCH) {
        Box::new(VirtualFromPhysical::with_vat(
            mem,
            proc_arch,
            x86::x32::new_translator(dtb),
            vat,
        ))
    } else if ptr::eq(sys_arch, x86::x32_pae::ARCH) {
        Box::new(VirtualFromPhysical::with_vat(
            mem,
            proc_arch,
            x86::x32_pae::new_translator(dtb),
            vat,
        ))
    } else {
        panic!(
            "Invalid architecture {:?} {:?} {:?} {:?}",
            sys_arch as *const _,
            x86::x64::ARCH as *const _,
            x86::x32::ARCH as *const _,
            x86::x32_pae::ARCH as *const _
        );
    }
}
