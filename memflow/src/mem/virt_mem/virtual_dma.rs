use std::prelude::v1::*;

use super::{VirtualReadData, VirtualWriteData};
use crate::architecture::{ArchitectureObj, VirtualTranslate3};
use crate::error::{Error, *};
use crate::iter::FnExtend;
use crate::mem::{
    virt_translate::{DirectTranslate, VirtualTranslate, VirtualTranslate2, MemoryRange, VirtualTranslationCallback, VirtualTranslationFailCallback, VirtualTranslation, VirtualTranslationFail},
    PhysicalMemory, PhysicalReadData, PhysicalWriteData, VirtualMemory,
};
use crate::types::{Address, PhysicalAddress};

use bumpalo::{collections::Vec as BumpVec, Bump};

/// The VirtualDma struct provides a default implementation to access virtual memory
/// from user provided [`PhysicalMemory`] and [`VirtualTranslate2`] objects.
///
/// This struct implements [`VirtualMemory`] and allows the user to access the virtual memory of a process.
pub struct VirtualDma<T, V, D> {
    phys_mem: T,
    vat: V,
    proc_arch: ArchitectureObj,
    translator: D,
    arena: Bump,
}

impl<T: PhysicalMemory, D: VirtualTranslate3> VirtualDma<T, DirectTranslate, D> {
    /// Constructs a `VirtualDma` object from user supplied architectures and DTB.
    /// It creates a default `VirtualTranslate2` object using the `DirectTranslate` struct.
    ///
    /// If you want to use a cache for translating virtual to physical memory
    /// consider using the `VirtualDma::with_vat()` function and supply your own `VirtualTranslate2` object.
    ///
    /// # Examples
    ///
    /// Constructing a `VirtualDma` object with a given dtb and using it to read:
    /// ```
    /// use memflow::types::Address;
    /// use memflow::architecture::x86::x64;
    /// use memflow::mem::{PhysicalMemory, VirtualTranslate2, VirtualMemory, VirtualDma};
    /// use memflow::cglue::Fwd;
    ///
    /// fn read(phys_mem: Fwd<&mut impl PhysicalMemory>, vat: &mut impl VirtualTranslate2, dtb: Address, read_addr: Address) {
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
    /// # use memflow::mem::DirectTranslate;
    /// # use memflow::cglue::ForwardMut;
    /// # let mem = DummyMemory::new(size::mb(4));
    /// # let (mut os, dtb, virt_base) = DummyOs::new_and_dtb(mem, size::mb(2), &[255, 0, 255, 0, 255, 0, 255, 0]);
    /// # let mut vat = DirectTranslate::new();
    /// # read(os.forward_mut(), &mut vat, dtb, virt_base);
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

impl<T: PhysicalMemory, V: VirtualTranslate2, D: VirtualTranslate3> VirtualDma<T, V, D> {
    /// This function constructs a `VirtualDma` instance with a user supplied `VirtualTranslate2` object.
    /// It can be used when working with cached virtual to physical translations such as a Tlb.
    ///
    /// # Examples
    ///
    /// Constructing a `VirtualDma` object with VAT and using it to read:
    /// ```
    /// use memflow::types::Address;
    /// use memflow::architecture::x86::x64;
    /// use memflow::mem::{PhysicalMemory, VirtualTranslate2, VirtualMemory, VirtualDma};
    /// use memflow::cglue::Fwd;
    ///
    /// fn read(phys_mem: Fwd<&mut impl PhysicalMemory>, vat: impl VirtualTranslate2, dtb: Address, read_addr: Address) {
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
    /// # use memflow::mem::DirectTranslate;
    /// # use memflow::cglue::ForwardMut;
    /// # let mem = DummyMemory::new(size::mb(4));
    /// # let (mut os, dtb, virt_base) = DummyOs::new_and_dtb(mem, size::mb(2), &[255, 0, 255, 0, 255, 0, 255, 0]);
    /// # let mut vat = DirectTranslate::new();
    /// # read(os.forward_mut(), &mut vat, dtb, virt_base);
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
    pub fn translator(&self) -> &impl VirtualTranslate3 {
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
    
    pub fn phys_mem_ref(&self) -> &T {
        &self.phys_mem
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

impl<T: PhysicalMemory, V: VirtualTranslate2, D: VirtualTranslate3> VirtualMemory
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
            &mut FnExtend::new(|(a, b): (_, &mut [u8])| {
                translation.push(PhysicalReadData(a, b.into()))
            }),
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
            &mut FnExtend::new(|(a, b): (_, &[u8])| {
                translation.push(PhysicalWriteData(a, b.into()))
            }),
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
}

impl<T: PhysicalMemory, V: VirtualTranslate2, D: VirtualTranslate3> VirtualTranslate
    for VirtualDma<T, V, D>
{
    fn virt_to_phys_list(
        &mut self,
        addrs: &[MemoryRange],
        mut out: VirtualTranslationCallback,
        mut out_fail: VirtualTranslationFailCallback,
    ) {
        self.vat.virt_to_phys_iter(
            &mut self.phys_mem,
            &self.translator,
            addrs.iter().map(|v| (v.address, (v.address, v.size))),
            &mut FnExtend::new(|(a, b): (PhysicalAddress, (Address, usize))| {
                let _ = out.call(VirtualTranslation {
                    in_virtual: b.0,
                    size: b.1,
                    out_physical: a,
                });
            }),
            &mut FnExtend::new(|(_e, from, (_, size))| {
                let _ = out_fail.call(VirtualTranslationFail { from, size });
            }),
        )
    }
}

