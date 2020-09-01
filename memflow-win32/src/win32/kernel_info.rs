use crate::error::Result;
use crate::kernel::{self, StartBlock};
use crate::kernel::{Win32GUID, Win32Version};

use log::{info, warn};

use memflow::architecture::ArchitectureObj;
use memflow::mem::{DirectTranslate, PhysicalMemory, VirtualDMA};
use memflow::types::Address;

use super::Win32VirtualTranslate;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct KernelInfo {
    pub start_block: StartBlock,

    pub kernel_base: Address,
    pub kernel_size: usize,

    pub kernel_guid: Option<Win32GUID>,
    pub kernel_winver: Win32Version,

    pub eprocess_base: Address,
}

impl KernelInfo {
    pub fn scanner<T: PhysicalMemory>(mem: T) -> KernelInfoScanner<T> {
        KernelInfoScanner::new(mem)
    }
}

pub struct KernelInfoScanner<T> {
    mem: T,
    arch: Option<ArchitectureObj>,
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
        let start_block = if let (Some(arch), Some(dtb), Some(kernel_hint)) =
            (self.arch, self.dtb, self.kernel_hint)
        {
            // construct start block from user supplied hints
            StartBlock {
                arch,
                kernel_hint,
                dtb,
            }
        } else {
            let mut sb = kernel::start_block::find(&mut self.mem, self.arch)?;
            if self.kernel_hint.is_some() && sb.kernel_hint.is_null() {
                sb.kernel_hint = self.kernel_hint.unwrap()
            }
            // dtb is always set in start_block::find()
            sb
        };

        self.scan_block(start_block).or_else(|_| {
            let start_block = kernel::start_block::find_fallback(&mut self.mem, start_block.arch)?;
            self.scan_block(start_block)
        })
    }

    fn scan_block(&mut self, start_block: StartBlock) -> Result<KernelInfo> {
        info!(
            "arch={:?} kernel_hint={:x} dtb={:x}",
            start_block.arch, start_block.kernel_hint, start_block.dtb
        );

        // construct virtual memory object for start_block
        let mut virt_mem = VirtualDMA::with_vat(
            &mut self.mem,
            start_block.arch,
            Win32VirtualTranslate::new(start_block.arch, start_block.dtb),
            DirectTranslate::new(),
        );

        // find ntoskrnl.exe base
        let (kernel_base, kernel_size) = kernel::ntos::find(&mut virt_mem, &start_block)?;
        info!("kernel_base={} kernel_size={}", kernel_base, kernel_size);

        // get ntoskrnl.exe guid
        let kernel_guid = kernel::ntos::find_guid(&mut virt_mem, kernel_base).ok();
        info!("kernel_guid={:?}", kernel_guid);

        let kernel_winver = kernel::ntos::find_winver(&mut virt_mem, kernel_base).ok();

        if kernel_winver.is_none() {
            warn!("Failed to retrieve kernel version! Some features may be disabled.");
        }

        let kernel_winver = kernel_winver.unwrap_or_default();

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

    pub fn arch(mut self, arch: ArchitectureObj) -> Self {
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
