use crate::error::Result;
use crate::kernel::{self, StartBlock};
use crate::kernel::{Win32GUID, Win32Version};

use log::info;

use memflow_core::architecture::Architecture;
use memflow_core::mem::{PhysicalMemory, VirtualFromPhysical};
use memflow_core::types::Address;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct KernelInfo {
    pub start_block: StartBlock,

    pub kernel_base: Address,
    pub kernel_size: usize,

    pub kernel_guid: Option<Win32GUID>,
    pub kernel_winver: Option<Win32Version>,

    pub eprocess_base: Address,
}

impl KernelInfo {
    pub fn scanner<T: PhysicalMemory>(mem: T) -> KernelInfoScanner<T> {
        KernelInfoScanner::new(mem)
    }
}

pub struct KernelInfoScanner<T> {
    mem: T,
    arch: Option<Architecture>,
    kernel_hint: Option<Address>,
    dtb: Option<Address>,
}

impl<T: PhysicalMemory> KernelInfoScanner<T> {
    pub fn new(mem: T) -> Self {
        Self {
            mem,
            arch: None,
            kernel_hint: None,
            dtb: None,
        }
    }

    pub fn scan(mut self) -> Result<KernelInfo> {
        let start_block = if self.arch.is_some() && self.dtb.is_some() && self.kernel_hint.is_some()
        {
            // construct start block from user supplied hints
            StartBlock {
                arch: self.arch.unwrap(),
                kernel_hint: self.kernel_hint.unwrap(),
                dtb: self.dtb.unwrap(),
            }
        } else {
            // find start_block in lowstub base
            let mut sb = kernel::lowstub::find(&mut self.mem, self.arch)?;
            if self.kernel_hint.is_some() && sb.kernel_hint.is_null() {
                sb.kernel_hint = self.kernel_hint.unwrap()
            }
            // dtb is always set in lowstub::find()
            sb
        };

        info!(
            "arch={:?} kernel_hint={:x} dtb={:x}",
            start_block.arch, start_block.kernel_hint, start_block.dtb
        );

        // construct virtual memory object for start_block
        let mut virt_mem = VirtualFromPhysical::new(
            &mut self.mem,
            start_block.arch,
            start_block.arch,
            start_block.dtb,
        );

        // find ntoskrnl.exe base
        let (kernel_base, kernel_size) = kernel::ntos::find(&mut virt_mem, &start_block)?;
        info!("kernel_base={} kernel_size={}", kernel_base, kernel_size);

        // get ntoskrnl.exe guid
        let kernel_guid = kernel::ntos::find_guid(&mut virt_mem, kernel_base).ok();
        info!("kernel_guid={:?}", kernel_guid);

        let kernel_winver = kernel::ntos::find_winver(&mut virt_mem, kernel_base).ok();
        info!("kernel_winver={:?}", kernel_winver);

        // find eprocess base
        let eprocess_base = kernel::sysproc::find(&mut virt_mem, &start_block, kernel_base)?;
        info!("eprocess_base={:x}", eprocess_base);

        // start_block only contains the winload's dtb which might
        // be different to the one used in the actual kernel.
        // see Kernel::new() for more information.
        info!("start_block.dtb={:x}", start_block.dtb);

        Ok(KernelInfo {
            start_block,

            kernel_base,
            kernel_size,

            kernel_guid,
            kernel_winver,

            eprocess_base,
        })
    }

    pub fn arch(mut self, arch: Architecture) -> Self {
        self.arch = Some(arch);
        self
    }

    pub fn kernel_hint(mut self, kernel_hint: Address) -> Self {
        self.kernel_hint = Some(kernel_hint);
        self
    }

    pub fn dtb(mut self, dtb: Address) -> Self {
        self.dtb = Some(dtb);
        self
    }
}
