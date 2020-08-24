use memflow_core::architecture::x86;
use memflow_core::architecture::Architecture;
use memflow_core::mem::{
    PhysicalMemory, VirtualDMA, VirtualMemory, VirtualMemoryBox, VirtualTranslate,
};
use memflow_core::types::Address;

use super::vat::Win32VirtualTranslate;

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
    Box::new(VirtualDMA::with_vat(
        mem,
        proc_arch,
        Win32VirtualTranslate { dtb, sys_arch },
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
    Box::new(VirtualDMA::with_vat(
        mem,
        proc_arch,
        Win32VirtualTranslate { dtb, sys_arch },
        vat,
    ))
}
