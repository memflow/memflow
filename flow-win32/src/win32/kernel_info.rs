use crate::error::Result;
use crate::kernel::{self, ntos::Win32GUID, StartBlock};

use log::info;

use flow_core::architecture::Architecture;
use flow_core::mem::{PhysicalMemory, VirtualFromPhysical};
use flow_core::types::Address;

#[derive(Debug, Clone)]
pub struct KernelInfo {
    pub start_block: StartBlock,

    pub kernel_base: Address,
    pub kernel_size: usize,
    pub kernel_guid: Win32GUID,
    pub kernel_dtb: Address,

    pub eprocess_base: Address,
}

impl KernelInfo {
    pub fn builder<T: PhysicalMemory>() -> KernelInfoBuilder<T> {
        KernelInfoBuilder::default()
    }
}

pub struct KernelInfoBuilder<T: PhysicalMemory> {
    mem: Option<T>,
    arch: Option<Architecture>,
    kernel_hint: Option<Address>,
    dtb: Option<Address>,
}

impl<T: PhysicalMemory> Default for KernelInfoBuilder<T> {
    fn default() -> Self {
        Self {
            mem: None,
            arch: None,
            kernel_hint: None,
            dtb: None,
        }
    }
}

impl<T: PhysicalMemory> KernelInfoBuilder<T> {
    pub fn build(self) -> Result<KernelInfo> {
        let mut mem = self.mem.ok_or("mem must be initialized")?;

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
            let mut sb = kernel::lowstub::find(&mut mem, self.arch)?;
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
            &mut mem,
            start_block.arch,
            start_block.arch,
            start_block.dtb,
        );

        // find ntoskrnl.exe base
        let (kernel_base, kernel_size) = kernel::ntos::find(&mut virt_mem, &start_block)?;
        info!("kernel_base={} kernel_size={}", kernel_base, kernel_size);

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

        Ok(KernelInfo {
            start_block,

            kernel_base,
            kernel_size,
            kernel_guid,
            kernel_dtb,

            eprocess_base,
        })
    }

    pub fn mem(mut self, mem: T) -> Self {
        self.mem = Some(mem);
        self
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
