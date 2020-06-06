use crate::error::Result;
use crate::kernel::{self, ntos::Win32GUID, StartBlock};

use log::info;

use flow_core::mem::{PhysicalMemory, VirtualFromPhysical};
use flow_core::types::{Address, Length};

#[derive(Debug, Clone)]
pub struct KernelInfo {
    pub start_block: StartBlock,

    pub kernel_base: Address,
    pub kernel_size: Length,
    pub kernel_guid: Win32GUID,

    pub eprocess_base: Address,
}

impl KernelInfo {
    pub fn find<T: PhysicalMemory>(mut phys_mem: T) -> Result<Self> {
        /*
        Options:
        - supply cr3 (dtb)
        - supply kernel hint
        */

        // find start_block in lowstub base
        let start_block = kernel::lowstub::find(&mut phys_mem)?;
        info!(
            "arch={:?} va={:x} dtb={:x}",
            start_block.arch, start_block.va, start_block.dtb
        );

        // construct virtual memory object for start_block
        let mut virt_mem = VirtualFromPhysical::new(
            &mut phys_mem,
            start_block.arch,
            start_block.arch,
            start_block.dtb,
        );

        // find ntoskrnl.exe base
        let (kernel_base, kernel_size) = kernel::ntos::find(&mut virt_mem, &start_block)?;
        info!("kernel_base={:x}", kernel_base);

        // get ntoskrnl.exe guid
        let kernel_guid = kernel::ntos::find_guid(&mut virt_mem, kernel_base, kernel_size)?;
        info!("kernel_guid={:?}", kernel_guid);

        // find eprocess base
        let eprocess_base = kernel::sysproc::find(&mut virt_mem, &start_block, kernel_base)?;
        info!("eprocess_base={:x}", eprocess_base);

        Ok(Self {
            start_block,

            kernel_base,
            kernel_size,
            kernel_guid,

            eprocess_base,
        })
    }
}
