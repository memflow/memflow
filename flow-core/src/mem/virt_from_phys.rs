use super::vat;
use super::{
    vat::{VirtualAdressTranslator, VAT},
    virt_mem::{VirtualMemory, VirtualReadIterator, VirtualWriteIterator},
    PhysicalMemory,
};
use crate::architecture::Architecture;
use crate::error::{Error, Result};
use crate::types::{Address, Page};

pub struct VirtualFromPhysical<T: PhysicalMemory, V: VAT> {
    phys_mem: T,
    sys_arch: Architecture,
    vat: V,
    proc_arch: Architecture,
    dtb: Address,
}

impl<T: PhysicalMemory> VirtualFromPhysical<T, VirtualAdressTranslator> {
    pub fn new(phys_mem: T, sys_arch: Architecture, proc_arch: Architecture, dtb: Address) -> Self {
        Self {
            phys_mem,
            sys_arch,
            vat: VirtualAdressTranslator::new(sys_arch),
            proc_arch,
            dtb,
        }
    }
}

// TODO: with_process
impl<T: PhysicalMemory, V: VAT> VirtualFromPhysical<T, V> {
    pub fn with_vat(
        phys_mem: T,
        sys_arch: Architecture,
        proc_arch: Architecture,
        dtb: Address,
        vat: V,
    ) -> Self {
        Self {
            phys_mem,
            sys_arch,
            vat,
            proc_arch,
            dtb,
        }
    }

    pub fn sys_arch(&self) -> Architecture {
        self.sys_arch
    }

    pub fn vat(&mut self) -> &mut V {
        &mut self.vat
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

impl<T: PhysicalMemory, V: VAT> VirtualMemory for VirtualFromPhysical<T, V> {
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
