use super::vat;
use super::{
    vat::VirtualAdressTranslator,
    virt_mem::{VirtualMemory, VirtualReadIterator, VirtualWriteIterator},
    PhysicalMemory,
};
use crate::architecture::Architecture;
use crate::error::Error;
use crate::types::{Address, Page};
use crate::Result;

pub struct VirtualFromPhysical<T: PhysicalMemory> {
    phys_mem: T,
    sys_arch: Architecture,
    vat: VirtualAdressTranslator,
    proc_arch: Architecture,
    dtb: Address,
}

impl<T: PhysicalMemory> VirtualFromPhysical<T> {
    pub fn new(phys_mem: T, sys_arch: Architecture, dtb: Address) -> Self {
        Self {
            phys_mem,
            sys_arch,
            vat: VirtualAdressTranslator::new(sys_arch),
            proc_arch: sys_arch,
            dtb,
        }
    }

    pub fn with_proc_arch(
        phys_mem: T,
        sys_arch: Architecture,
        proc_arch: Architecture,
        dtb: Address,
    ) -> Self {
        Self {
            phys_mem,
            sys_arch,
            vat: VirtualAdressTranslator::new(sys_arch),
            proc_arch,
            dtb,
        }
    }

    pub fn sys_arch(&self) -> Architecture {
        self.sys_arch
    }

    pub fn proc_arch(&self) -> Architecture {
        self.proc_arch
    }

    pub fn dtb(&self) -> Address {
        self.dtb
    }

    pub fn virt_read_addr(&mut self, addr: Address) -> Result<Address> {
        match self.proc_arch.bits() {
            64 => self.virt_read_addr64(addr),
            32 => self.virt_read_addr32(addr),
            _ => Err(Error::new("invalid instruction set address size")),
        }
    }
}

impl<T: PhysicalMemory> VirtualMemory for VirtualFromPhysical<T> {
    fn virt_read_raw_iter<'a, VI: VirtualReadIterator<'a>>(&mut self, iter: VI) -> Result<()> {
        vat::virt_read_raw_iter(
            &mut self.phys_mem,
            &mut self.vat,
            self.sys_arch,
            self.dtb,
            iter,
        )
    }

    fn virt_write_raw_iter<'a, VI: VirtualWriteIterator<'a>>(&mut self, iter: VI) -> Result<()> {
        vat::virt_write_raw_iter(
            &mut self.phys_mem,
            &mut self.vat,
            self.sys_arch,
            self.dtb,
            iter,
        )
    }

    fn virt_page_info(&mut self, addr: Address) -> Result<Page> {
        vat::virt_page_info(
            &mut self.phys_mem,
            &mut self.vat,
            self.sys_arch,
            self.dtb,
            addr,
        )
    }
}
