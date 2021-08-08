use std::convert::TryInto;

use memflow::{
    architecture::{
        arm, x86, ArchitectureIdent, ArchitectureObj, VirtualTranslate3, VtopFailureCallback,
        VtopOutputCallback,
    },
    iter::SplitAtIndex,
    mem::{MemData, MemoryView, PhysicalMemory, VirtualDma, VirtualTranslate2},
    types::Address,
};

#[derive(Debug, Clone, Copy)]
pub struct Win32VirtualTranslate {
    pub sys_arch: ArchitectureObj,
    pub dtb: Address,
}

impl Win32VirtualTranslate {
    pub fn new(arch: ArchitectureIdent, dtb: Address) -> Self {
        Self {
            sys_arch: arch.into(),
            dtb,
        }
    }

    pub fn virt_mem<T: PhysicalMemory, V: VirtualTranslate2>(
        self,
        mem: T,
        vat: V,
        proc_arch: ArchitectureObj,
    ) -> impl MemoryView {
        VirtualDma::with_vat(mem, proc_arch, self, vat)
    }
}

impl VirtualTranslate3 for Win32VirtualTranslate {
    fn virt_to_phys_iter<
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        VI: Iterator<Item = MemData<Address, B>>,
    >(
        &self,
        mem: &mut T,
        addrs: VI,
        out: &mut VtopOutputCallback<B>,
        out_fail: &mut VtopFailureCallback<B>,
        tmp_buf: &mut [std::mem::MaybeUninit<u8>],
    ) {
        if let Ok(translator) = x86::new_translator(self.dtb, self.sys_arch) {
            translator.virt_to_phys_iter(mem, addrs, out, out_fail, tmp_buf)
        } else if let Ok(translator) = arm::new_translator_nonsplit(self.dtb, self.sys_arch) {
            translator.virt_to_phys_iter(mem, addrs, out, out_fail, tmp_buf)
        } else {
            panic!("Invalid architecture");
        }
    }

    fn translation_table_id(&self, _address: Address) -> usize {
        self.dtb.to_umem().overflowing_shr(12).0.try_into().unwrap()
    }

    fn arch(&self) -> ArchitectureObj {
        self.sys_arch
    }
}
