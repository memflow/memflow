use std::prelude::v1::*;

use super::{VirtualReadData, VirtualWriteData};
use crate::architecture::{Architecture, ScopedVirtualTranslate};
use crate::error::{Error, PartialError, PartialResult, Result};
use crate::iter::FnExtend;
use crate::mem::{
    virt_translate::{DirectTranslate, VirtualTranslate},
    PhysicalMemory, VirtualMemory,
};
use crate::process::OsProcessInfo;
use crate::types::{Address, Page};

use bumpalo::{collections::Vec as BumpVec, Bump};
use itertools::Itertools;

/// The `VirtualFromPhysical` struct provides a default implementation to access virtual memory
/// from user provided `PhysicalMemory` and `VirtualTranslate` objects.
///
/// This struct implements `VirtualMemory` and allows the user to access the virtual memory of a process.
pub struct VirtualFromPhysical<T, V, D> {
    phys_mem: T,
    vat: V,
    proc_arch: &'static dyn Architecture,
    translator: D,
    arena: Bump,
}

impl<T: PhysicalMemory, D: ScopedVirtualTranslate> VirtualFromPhysical<T, DirectTranslate, D> {
    /// Constructs a `VirtualFromPhysical` object from user supplied architectures and DTB.
    /// It creates a default `VirtualTranslate` object using the `DirectTranslate` struct.
    ///
    /// If you want to use a cache for translating virtual to physical memory
    /// consider using the `VirtualFromPhysical::with_vat()` function and supply your own `VirtualTranslate` object.
    ///
    /// # Examples
    ///
    /// Constructing a `VirtualFromPhysical` object with a given dtb and using it to read:
    /// ```
    /// use memflow_core::types::Address;
    /// use memflow_core::architecture::Architecture;
    /// use memflow_core::mem::{PhysicalMemory, VirtualTranslate, VirtualMemory, VirtualFromPhysical};
    ///
    /// fn read<T: PhysicalMemory, V: VirtualTranslate>(phys_mem: &mut T, vat: &mut V, dtb: Address, read_addr: Address) {
    ///     let arch = Architecture::X64;
    ///
    ///     let mut virt_mem = VirtualFromPhysical::new(phys_mem, arch, arch, dtb);
    ///
    ///     let mut addr = 0u64;
    ///     virt_mem.virt_read_into(read_addr, &mut addr).unwrap();
    ///     println!("addr: {:x}", addr);
    ///     # assert_eq!(addr, 0x00ff_00ff_00ff_00ff);
    /// }
    /// # use memflow_core::dummy::DummyMemory;
    /// # use memflow_core::types::size;
    /// # use memflow_core::mem::DirectTranslate;
    /// # let (mut mem, dtb, virt_base) = DummyMemory::new_and_dtb(size::mb(4), size::mb(2), &[255, 0, 255, 0, 255, 0, 255, 0]);
    /// # let mut vat = DirectTranslate::new();
    /// # read(&mut mem, &mut vat, dtb, virt_base);
    /// ```
    pub fn new(phys_mem: T, proc_arch: &'static dyn Architecture, translator: D) -> Self {
        Self {
            phys_mem,
            vat: DirectTranslate::new(),
            proc_arch,
            translator,
            arena: Bump::new(),
        }
    }

    /*/// This function constructs a `VirtualFromPhysical` instance for a given process.
    /// It creates a default `VirtualTranslate` object using the `DirectTranslate` struct.
    ///
    /// If you want to use a cache for translating virtual to physical memory
    /// consider using the `VirtualFromPhysical::with_vat()` function and supply your own `VirtualTranslate` object.
    ///
    /// # Examples
    ///
    /// Constructing a `VirtualFromPhysical` object from a `OsProcessInfo` and using it to read:
    /// ```
    /// use memflow_core::types::Address;
    /// use memflow_core::mem::{PhysicalMemory, VirtualTranslate, VirtualMemory, VirtualFromPhysical};
    /// use memflow_core::process::OsProcessInfo;
    ///
    /// fn read<T: PhysicalMemory, P: OsProcessInfo>(phys_mem: &mut T, process_info: P, read_addr: Address) {
    ///     let mut virt_mem = VirtualFromPhysical::from_process_info(phys_mem, process_info);
    ///
    ///     let mut addr = 0u64;
    ///     virt_mem.virt_read_into(read_addr, &mut addr).unwrap();
    ///     println!("addr: {:x}", addr);
    ///     # assert_eq!(addr, 0x00ff_00ff_00ff_00ff);
    /// }
    /// # use memflow_core::dummy::DummyMemory;
    /// # use memflow_core::types::size;
    /// # use memflow_core::mem::DirectTranslate;
    /// # let mut mem = DummyMemory::new(size::mb(4));
    /// # let proc = mem.alloc_process(size::mb(2), &[255, 0, 255, 0, 255, 0, 255, 0]);
    /// # let addr = proc.address();
    /// # read(&mut mem, proc, addr);
    /// ```
    /// TODO: Remove, or fix (Object Safety)
    pub fn from_process_info<U: OsProcessInfo>(phys_mem: T, process_info: U) -> Self {
        Self {
            phys_mem,
            vat: DirectTranslate::new(),
            proc_arch: process_info.proc_arch(),
            dtb: process_info.dtb(),
            arena: Bump::new(),
        }
    }*/
}

impl<T: PhysicalMemory, V: VirtualTranslate, D: ScopedVirtualTranslate>
    VirtualFromPhysical<T, V, D>
{
    /// This function constructs a `VirtualFromPhysical` instance with a user supplied `VirtualTranslate` object.
    /// It can be used when working with cached virtual to physical translations such as a TLB.
    ///
    /// # Examples
    ///
    /// Constructing a `VirtualFromPhysical` object with VAT and using it to read:
    /// ```
    /// use memflow_core::types::Address;
    /// use memflow_core::architecture::Architecture;
    /// use memflow_core::mem::{PhysicalMemory, VirtualTranslate, VirtualMemory, VirtualFromPhysical};
    ///
    /// fn read<T: PhysicalMemory, V: VirtualTranslate>(phys_mem: &mut T, vat: V, dtb: Address, read_addr: Address) {
    ///     let arch = Architecture::X64;
    ///
    ///     let mut virt_mem = VirtualFromPhysical::with_vat(phys_mem, arch, arch, dtb, vat);
    ///
    ///     let mut addr = 0u64;
    ///     virt_mem.virt_read_into(read_addr, &mut addr).unwrap();
    ///     println!("addr: {:x}", addr);
    ///     # assert_eq!(addr, 0x00ff_00ff_00ff_00ff);
    /// }
    /// # use memflow_core::dummy::DummyMemory;
    /// # use memflow_core::types::size;
    /// # use memflow_core::mem::DirectTranslate;
    /// # let (mut mem, dtb, virt_base) = DummyMemory::new_and_dtb(size::mb(4), size::mb(2), &[255, 0, 255, 0, 255, 0, 255, 0]);
    /// # let mut vat = DirectTranslate::new();
    /// # read(&mut mem, &mut vat, dtb, virt_base);
    /// ```
    pub fn with_vat(
        phys_mem: T,
        proc_arch: &'static dyn Architecture,
        translator: D,
        vat: V,
    ) -> Self {
        Self {
            phys_mem,
            vat,
            proc_arch,
            translator,
            arena: Bump::new(),
        }
    }

    /// Returns the architecture of the system. The system architecture is used for virtual to physical translations.
    pub fn sys_arch(&self) -> &dyn Architecture {
        self.translator.arch()
    }

    /// Returns the architecture of the process for this context. The process architecture is mainly used to determine pointer sizes.
    pub fn proc_arch(&self) -> &dyn Architecture {
        self.proc_arch
    }

    /// Returns the Directory Table Base of this process.
    pub fn translator(&self) -> &impl ScopedVirtualTranslate {
        &self.translator
    }

    /// A wrapper around `virt_read_addr64` and `virt_read_addr32` that will use the pointer size of this context's process.
    pub fn virt_read_addr(&mut self, addr: Address) -> PartialResult<Address> {
        match self.proc_arch.bits() {
            64 => self.virt_read_addr64(addr),
            32 => self.virt_read_addr32(addr),
            _ => Err(PartialError::Error(Error::InvalidArchitecture)),
        }
    }

    /// Consume the self object and returns the containing memory connection
    pub fn destroy(self) -> T {
        self.phys_mem
    }
}

impl<T, V, D> Clone for VirtualFromPhysical<T, V, D>
where
    T: Clone,
    V: Clone,
    D: Clone,
{
    fn clone(&self) -> Self {
        Self {
            phys_mem: self.phys_mem.clone(),
            vat: self.vat.clone(),
            proc_arch: self.proc_arch,
            translator: self.translator.clone(),
            arena: Bump::new(),
        }
    }
}

impl<T: PhysicalMemory, V: VirtualTranslate, D: ScopedVirtualTranslate> VirtualMemory
    for VirtualFromPhysical<T, V, D>
{
    fn virt_read_raw_list(&mut self, data: &mut [VirtualReadData]) -> PartialResult<()> {
        self.arena.reset();
        let mut translation = BumpVec::with_capacity_in(data.len(), &self.arena);

        let mut partial_read = false;
        self.vat.virt_to_phys_iter(
            &mut self.phys_mem,
            &self.translator,
            data.iter_mut().map(|(a, b)| (*a, &mut b[..])),
            &mut translation,
            &mut FnExtend::new(|(_, _, out): (_, _, &mut [u8])| {
                for v in out.iter_mut() {
                    *v = 0;
                }
                partial_read = true;
            }),
        );

        self.phys_mem.phys_read_raw_list(&mut translation)?;
        if !partial_read {
            Ok(())
        } else {
            Err(PartialError::PartialVirtualRead(()))
        }
    }

    fn virt_write_raw_list(&mut self, data: &[VirtualWriteData]) -> PartialResult<()> {
        self.arena.reset();
        let mut translation = BumpVec::with_capacity_in(data.len(), &self.arena);

        let mut partial_read = false;
        self.vat.virt_to_phys_iter(
            &mut self.phys_mem,
            &self.translator,
            data.iter().copied(),
            &mut translation,
            &mut FnExtend::new(|(_, _, _): (_, _, _)| {
                partial_read = true;
            }),
        );

        self.phys_mem.phys_write_raw_list(&translation)?;
        if !partial_read {
            Ok(())
        } else {
            Err(PartialError::PartialVirtualRead(()))
        }
    }

    fn virt_page_info(&mut self, addr: Address) -> Result<Page> {
        let paddr = self
            .vat
            .virt_to_phys(&mut self.phys_mem, &self.translator, addr)?;
        Ok(paddr.containing_page())
    }

    fn virt_page_map_range(
        &mut self,
        gap_length: usize,
        start: Address,
        end: Address,
    ) -> Vec<(Address, usize)> {
        self.arena.reset();
        let mut out = BumpVec::new_in(&self.arena);

        self.vat.virt_to_phys_iter(
            &mut self.phys_mem,
            &self.translator,
            Some((start, (start, end - start))).into_iter(),
            &mut out,
            &mut FnExtend::void(),
        );

        out.sort_by(|(_, (a, _)), (_, (b, _))| a.cmp(b));

        out.into_iter()
            .map(|(_, a)| a)
            .coalesce(|a, b| {
                if b.0 - (a.0 + a.1) <= gap_length {
                    Ok((a.0, b.0 + b.1 - a.0))
                } else {
                    Err((a, b))
                }
            })
            .collect()
    }
}
