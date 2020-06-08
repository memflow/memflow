use super::{VirtualReadIterator, VirtualWriteIterator};
use crate::architecture::Architecture;
use crate::error::{Error, Result};
use crate::mem::{
    virt_translate::{translate_arch, TranslateArch, VirtualTranslate},
    PhysicalMemory, VirtualMemory,
};
use crate::process::OsProcessInfo;
use crate::types::{Address, Page};

/**
The `VirtualFromPhysical` struct provides a default implementation to access virtual memory
from user provided `PhysicalMemory` and `VirtualTranslate` objects.

This struct implements `VirtualMemory` and allows the user to access the virtual memory of a process.
*/
pub struct VirtualFromPhysical<T: PhysicalMemory, V: VirtualTranslate> {
    phys_mem: T,
    sys_arch: Architecture,
    vat: V,
    proc_arch: Architecture,
    dtb: Address,
}

impl<T: PhysicalMemory> VirtualFromPhysical<T, TranslateArch> {
    /**
    Constructs a `VirtualFromPhysical` object from user supplied architectures and DTB.
    It creates a default `VirtualTranslate` object using the `TranslateArch` struct.

    If you want to use a cache for translating virtual to physical memory
    consider using the `VirtualFromPhysical::with_vat()` function and supply your own `VirtualTranslate` object.

    # Examples

    Constructing a `VirtualFromPhysical` object with a given dtb and using it to read:
    ```
    use flow_core::types::Address;
    use flow_core::architecture::Architecture;
    use flow_core::mem::{PhysicalMemory, VirtualTranslate, VirtualMemory, VirtualFromPhysical};

    fn read<T: PhysicalMemory, V: VirtualTranslate>(phys_mem: &mut T, vat: &mut V) {
        let arch = Architecture::X64;
        let dtb = Address::NULL;

        let mut virt_mem = VirtualFromPhysical::new(phys_mem, arch, arch, dtb);

        let mut addr = 0u64;
        virt_mem.virt_read_into(Address::from(0x1000), &mut addr).unwrap();
        println!("addr: {:x}", addr);
    }
    ```
    */
    pub fn new(phys_mem: T, sys_arch: Architecture, proc_arch: Architecture, dtb: Address) -> Self {
        Self {
            phys_mem,
            sys_arch,
            vat: TranslateArch::new(sys_arch),
            proc_arch,
            dtb,
        }
    }

    /**
    This function constructs a `VirtualFromPhysical` instance for a given process.
    It creates a default `VirtualTranslate` object using the `TranslateArch` struct.

    If you want to use a cache for translating virtual to physical memory
    consider using the `VirtualFromPhysical::with_vat()` function and supply your own `VirtualTranslate` object.

    # Examples

    Constructing a `VirtualFromPhysical` object from a `OsProcessInfo` and using it to read:
    ```
    use flow_core::types::Address;
    use flow_core::mem::{PhysicalMemory, VirtualTranslate, VirtualMemory, VirtualFromPhysical};
    use flow_core::process::OsProcessInfo;

    fn read<T: PhysicalMemory, P: OsProcessInfo>(phys_mem: &mut T, process_info: P) {
        let mut virt_mem = VirtualFromPhysical::from_process_info(phys_mem, process_info);

        let mut addr = 0u64;
        virt_mem.virt_read_into(Address::from(0x1000), &mut addr).unwrap();
        println!("addr: {:x}", addr);
    }
    ```
     */
    pub fn from_process_info<U: OsProcessInfo>(phys_mem: T, process_info: U) -> Self {
        Self {
            phys_mem,
            sys_arch: process_info.sys_arch(),
            vat: TranslateArch::new(process_info.sys_arch()),
            proc_arch: process_info.proc_arch(),
            dtb: process_info.dtb(),
        }
    }
}

impl<T: PhysicalMemory, V: VirtualTranslate> VirtualFromPhysical<T, V> {
    /**
    This function constructs a `VirtualFromPhysical` instance with a user supplied `VirtualTranslate` object.
    It can be used when working with cached virtual to physical translations such as a TLB.

    # Examples

    Constructing a `VirtualFromPhysical` object with VAT and using it to read:
    ```
    use flow_core::types::Address;
    use flow_core::architecture::Architecture;
    use flow_core::mem::{PhysicalMemory, VirtualTranslate, VirtualMemory, VirtualFromPhysical};

    fn read<T: PhysicalMemory, V: VirtualTranslate>(phys_mem: &mut T, vat: V) {
        let arch = Architecture::X64;
        let dtb = Address::NULL;

        let mut virt_mem = VirtualFromPhysical::with_vat(phys_mem, arch, arch, dtb, vat);

        let mut addr = 0u64;
        virt_mem.virt_read_into(Address::from(0x1000), &mut addr).unwrap();
        println!("addr: {:x}", addr);
    }
    ```
    */
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

    /// Returns the architecture of the system. The system architecture is used for virtual to physical translations.
    pub fn sys_arch(&self) -> Architecture {
        self.sys_arch
    }

    /// Returns the architecture of the process for this context. The process architecture is mainly used to determine pointer sizes.
    pub fn proc_arch(&self) -> Architecture {
        self.proc_arch
    }

    /// Returns the Directory Table Base of this process.
    pub fn dtb(&self) -> Address {
        self.dtb
    }

    /// A wrapper around `virt_read_addr64` and `virt_read_addr32` that will use the pointer size of this context's process.
    pub fn virt_read_addr(&mut self, addr: Address) -> Result<Address> {
        match self.proc_arch.bits() {
            64 => self.virt_read_addr64(addr),
            32 => self.virt_read_addr32(addr),
            _ => Err(Error::new("invalid instruction set address size")),
        }
    }
}

impl<T: PhysicalMemory, V: VirtualTranslate> VirtualMemory for VirtualFromPhysical<T, V> {
    fn virt_read_raw_iter<'a, VI: VirtualReadIterator<'a>>(&mut self, iter: VI) -> Result<()> {
        translate_arch::virt_read_raw_iter(&mut self.phys_mem, &mut self.vat, self.dtb, iter)
    }

    fn virt_write_raw_iter<'a, VI: VirtualWriteIterator<'a>>(&mut self, iter: VI) -> Result<()> {
        translate_arch::virt_write_raw_iter(&mut self.phys_mem, &mut self.vat, self.dtb, iter)
    }

    fn virt_page_info(&mut self, addr: Address) -> Result<Page> {
        translate_arch::virt_page_info(&mut self.phys_mem, &mut self.vat, self.dtb, addr)
    }
}
