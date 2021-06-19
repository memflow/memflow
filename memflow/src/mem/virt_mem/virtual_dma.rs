use std::prelude::v1::*;

use super::{VirtualReadData, VirtualWriteData};
use crate::architecture::{ArchitectureObj, ScopedVirtualTranslate};
use crate::error::{Error, ErrorKind, ErrorOrigin, PartialError, PartialResult, Result};
use crate::iter::FnExtend;
use crate::mem::{
    virt_translate::{DirectTranslate, VirtualTranslate},
    PhysicalMemory, PhysicalReadData, PhysicalWriteData, VirtualMemory,
};
use crate::types::{Address, Page, PhysicalAddress};

use bumpalo::{collections::Vec as BumpVec, Bump};
use itertools::Itertools;

/// The VirtualDma struct provides a default implementation to access virtual memory
/// from user provided [`PhysicalMemory`] and [`VirtualTranslate`] objects.
///
/// This struct implements [`VirtualMemory`] and allows the user to access the virtual memory of a process.
pub struct VirtualDma<T, V, D> {
    phys_mem: T,
    vat: V,
    proc_arch: ArchitectureObj,
    translator: D,
    arena: Bump,
}

impl<T: PhysicalMemory, D: ScopedVirtualTranslate> VirtualDma<T, DirectTranslate, D> {
    /// Constructs a `VirtualDma` object from user supplied architectures and DTB.
    /// It creates a default `VirtualTranslate` object using the `DirectTranslate` struct.
    ///
    /// If you want to use a cache for translating virtual to physical memory
    /// consider using the `VirtualDma::with_vat()` function and supply your own `VirtualTranslate` object.
    ///
    /// # Examples
    ///
    /// Constructing a `VirtualDma` object with a given dtb and using it to read:
    /// ```
    /// use memflow::types::Address;
    /// use memflow::architecture::x86::x64;
    /// use memflow::mem::{PhysicalMemory, VirtualTranslate, VirtualMemory, VirtualDma};
    /// use memflow::cglue::Fwd;
    ///
    /// fn read(phys_mem: Fwd<&mut impl PhysicalMemory>, vat: &mut impl VirtualTranslate, dtb: Address, read_addr: Address) {
    ///     let arch = x64::ARCH;
    ///     let translator = x64::new_translator(dtb);
    ///
    ///     let mut virt_mem = VirtualDma::new(phys_mem, arch, translator);
    ///
    ///     let mut addr = 0u64;
    ///     virt_mem.virt_read_into(read_addr, &mut addr).unwrap();
    ///     println!("addr: {:x}", addr);
    ///     # assert_eq!(addr, 0x00ff_00ff_00ff_00ff);
    /// }
    /// # use memflow::dummy::{DummyMemory, DummyOs};
    /// # use memflow::types::size;
    /// # use memflow::mem::{DirectTranslate, AsPhysicalMemory};
    /// # use memflow::cglue::ForwardMut;
    /// # let mem = DummyMemory::new(size::mb(4));
    /// # let (mut os, dtb, virt_base) = DummyOs::new_and_dtb(mem, size::mb(2), &[255, 0, 255, 0, 255, 0, 255, 0]);
    /// # let mut vat = DirectTranslate::new();
    /// # read(os.phys_mem().forward_mut(), &mut vat, dtb, virt_base);
    /// ```
    pub fn new(phys_mem: T, arch: impl Into<ArchitectureObj>, translator: D) -> Self {
        Self {
            phys_mem,
            vat: DirectTranslate::new(),
            proc_arch: arch.into(),
            translator,
            arena: Bump::new(),
        }
    }
}

impl<T: PhysicalMemory, V: VirtualTranslate, D: ScopedVirtualTranslate> VirtualDma<T, V, D> {
    /// This function constructs a `VirtualDma` instance with a user supplied `VirtualTranslate` object.
    /// It can be used when working with cached virtual to physical translations such as a Tlb.
    ///
    /// # Examples
    ///
    /// Constructing a `VirtualDma` object with VAT and using it to read:
    /// ```
    /// use memflow::types::Address;
    /// use memflow::architecture::x86::x64;
    /// use memflow::mem::{PhysicalMemory, VirtualTranslate, VirtualMemory, VirtualDma};
    /// use memflow::cglue::Fwd;
    ///
    /// fn read(phys_mem: Fwd<&mut impl PhysicalMemory>, vat: impl VirtualTranslate, dtb: Address, read_addr: Address) {
    ///     let arch = x64::ARCH;
    ///     let translator = x64::new_translator(dtb);
    ///
    ///     let mut virt_mem = VirtualDma::with_vat(phys_mem, arch, translator, vat);
    ///
    ///     let mut addr = 0u64;
    ///     virt_mem.virt_read_into(read_addr, &mut addr).unwrap();
    ///     println!("addr: {:x}", addr);
    ///     # assert_eq!(addr, 0x00ff_00ff_00ff_00ff);
    /// }
    /// # use memflow::dummy::{DummyMemory, DummyOs};
    /// # use memflow::types::size;
    /// # use memflow::mem::{DirectTranslate, AsPhysicalMemory};
    /// # use memflow::cglue::ForwardMut;
    /// # let mem = DummyMemory::new(size::mb(4));
    /// # let (mut os, dtb, virt_base) = DummyOs::new_and_dtb(mem, size::mb(2), &[255, 0, 255, 0, 255, 0, 255, 0]);
    /// # let mut vat = DirectTranslate::new();
    /// # read(os.phys_mem().forward_mut(), &mut vat, dtb, virt_base);
    /// ```
    pub fn with_vat(phys_mem: T, arch: impl Into<ArchitectureObj>, translator: D, vat: V) -> Self {
        Self {
            phys_mem,
            vat,
            proc_arch: arch.into(),
            translator,
            arena: Bump::new(),
        }
    }

    /// Returns the architecture of the system. The system architecture is used for virtual to physical translations.
    pub fn sys_arch(&self) -> ArchitectureObj {
        self.translator.arch()
    }

    /// Returns the architecture of the process for this context. The process architecture is mainly used to determine pointer sizes.
    pub fn proc_arch(&self) -> ArchitectureObj {
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
            _ => Err(PartialError::Error(Error(
                ErrorOrigin::VirtualMemory,
                ErrorKind::InvalidArchitecture,
            ))),
        }
    }

    /// Consumes this VirtualDma object, returning the underlying memory and vat objects
    pub fn into_inner(self) -> (T, V) {
        (self.phys_mem, self.vat)
    }

    pub fn mem_vat_pair(&mut self) -> (&mut T, &mut V) {
        (&mut self.phys_mem, &mut self.vat)
    }

    pub fn phys_mem(&mut self) -> &mut T {
        &mut self.phys_mem
    }

    pub fn vat(&mut self) -> &mut V {
        &mut self.vat
    }
}

impl<T, V, D> Clone for VirtualDma<T, V, D>
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
    for VirtualDma<T, V, D>
{
    fn virt_read_raw_list(&mut self, data: &mut [VirtualReadData]) -> PartialResult<()> {
        self.arena.reset();
        let mut translation = BumpVec::with_capacity_in(data.len(), &self.arena);

        let mut partial_read = false;
        self.vat.virt_to_phys_iter(
            &mut self.phys_mem,
            &self.translator,
            data.iter_mut()
                .map(|VirtualReadData(a, b)| (*a, &mut b[..])),
            &mut FnExtend::new(|(a, b)| translation.push(PhysicalReadData(a, b))),
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
            data.iter().copied().map(<_>::into),
            &mut FnExtend::new(|(a, b)| translation.push(PhysicalWriteData(a, b))),
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

    fn virt_translation_map_range(
        &mut self,
        start: Address,
        end: Address,
    ) -> Vec<(Address, usize, PhysicalAddress)> {
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
            .coalesce(|(ap, av), (bp, bv)| {
                if bv.0 == (av.0 + av.1) && bp.address() == (ap.address() + av.1) {
                    Ok((ap, (av.0, bv.0 + bv.1 - av.0)))
                } else {
                    Err(((ap, av), (bp, bv)))
                }
            })
            .map(|(p, (v, s))| (v, s, p))
            .collect()
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
