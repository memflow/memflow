use memflow_core::{
    architecture::{x86, Architecture, ScopedVirtualTranslate},
    iter::SplitAtIndex,
    mem::{PhysicalMemory, VirtualDMA, VirtualMemory, VirtualTranslate},
    types::{Address, PhysicalAddress},
};

#[derive(Debug, Clone, Copy)]
pub struct Win32VirtualTranslate {
    pub sys_arch: &'static dyn Architecture,
    pub dtb: Address,
}

impl Win32VirtualTranslate {
    pub fn new(sys_arch: &'static dyn Architecture, dtb: Address) -> Self {
        Self { sys_arch, dtb }
    }

    pub fn virt_mem<T: PhysicalMemory, V: VirtualTranslate>(
        self,
        mem: T,
        vat: V,
        proc_arch: &'static dyn Architecture,
    ) -> impl VirtualMemory {
        VirtualDMA::with_vat(mem, proc_arch, self, vat)
    }
}

impl ScopedVirtualTranslate for Win32VirtualTranslate {
    fn virt_to_phys_iter<
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        VI: Iterator<Item = (Address, B)>,
        VO: Extend<(PhysicalAddress, B)>,
        FO: Extend<(memflow_core::Error, Address, B)>,
    >(
        &self,
        mem: &mut T,
        addrs: VI,
        out: &mut VO,
        out_fail: &mut FO,
        arena: &memflow_core::architecture::Bump,
    ) {
        let translator = x86::new_translator(self.dtb, self.sys_arch).unwrap();
        translator.virt_to_phys_iter(mem, addrs, out, out_fail, arena)
    }

    fn translation_table_id(&self, _address: Address) -> usize {
        self.dtb.as_u64().overflowing_shr(12).0 as usize
    }

    fn arch(&self) -> &dyn Architecture {
        self.sys_arch
    }
}
