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
    pub kernel_dtb: Address,

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
        let kernel_guid = kernel::ntos::find_guid(&mut virt_mem, kernel_base)?;
        info!("kernel_guid={:?}", kernel_guid);

        // find eprocess base
        let eprocess_base = kernel::sysproc::find(&mut virt_mem, &start_block, kernel_base)?;
        info!("eprocess_base={:x}", eprocess_base);

        // start_block only contains the winload's dtb which might
        // be different to the one used in the actual kernel
        // so we might read the real dtb here in the future
        info!("start_block.dtb={:x}", start_block.dtb);
        let kernel_dtb = start_block.dtb;
        //let kernel_dtb = virt_mem.virt_read_addr(eprocess_base + /*self.offsets.kproc_dtb*/ Length::from(0x18))?;
        info!("kernel_dtb={:x}", kernel_dtb);

        Ok(Self {
            start_block,

            kernel_base,
            kernel_size,
            kernel_guid,
            kernel_dtb,

            eprocess_base,
        })
    }
}
