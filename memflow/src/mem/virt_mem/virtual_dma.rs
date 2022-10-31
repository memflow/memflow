use std::prelude::v1::*;

use crate::architecture::{ArchitectureObj, Endianess};
use crate::error::{Error, Result, *};
use crate::mem::memory_view::*;
use crate::mem::{
    mem_data::*,
    virt_translate::{
        DirectTranslate, VirtualTranslate, VirtualTranslate2, VirtualTranslate3,
        VirtualTranslation, VirtualTranslationCallback, VirtualTranslationFail,
        VirtualTranslationFailCallback,
    },
    MemoryView, PhysicalMemory, PhysicalMemoryMetadata,
};
use crate::types::{umem, Address, PhysicalAddress};
use cglue::tuple::*;

use bumpalo::{collections::Vec as BumpVec, Bump};
use cglue::callback::FromExtend;

/// The VirtualDma struct provides a default implementation to access virtual memory
/// from user provided [`PhysicalMemory`] and [`VirtualTranslate2`] objects.
///
/// This struct implements [`MemoryView`] and allows the user to access the virtual memory of a process.
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
    /// use memflow::mem::{PhysicalMemory, VirtualTranslate2, MemoryView, VirtualDma};
    /// use memflow::cglue::Fwd;
    ///
    /// fn read(phys_mem: Fwd<&mut impl PhysicalMemory>, vat: &mut impl VirtualTranslate2, dtb: Address, read_addr: Address) {
    ///     let arch = x64::ARCH;
    ///     let translator = x64::new_translator(dtb);
    ///
    ///     let mut virt_mem = VirtualDma::new(phys_mem, arch, translator);
    ///
    ///     let mut addr = 0u64;
    ///     virt_mem.read_into(read_addr, &mut addr).unwrap();
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
    /// use memflow::mem::{PhysicalMemory, VirtualTranslate2, MemoryView, VirtualDma};
    /// use memflow::cglue::Fwd;
    ///
    /// fn read(phys_mem: Fwd<&mut impl PhysicalMemory>, vat: impl VirtualTranslate2, dtb: Address, read_addr: Address) {
    ///     let arch = x64::ARCH;
    ///     let translator = x64::new_translator(dtb);
    ///
    ///     let mut virt_mem = VirtualDma::with_vat(phys_mem, arch, translator, vat);
    ///
    ///     let mut addr = 0u64;
    ///     virt_mem.read_into(read_addr, &mut addr).unwrap();
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

    /// Replaces current process architecture with a new one.
    pub fn set_proc_arch(&mut self, new_arch: ArchitectureObj) -> ArchitectureObj {
        core::mem::replace(&mut self.proc_arch, new_arch)
    }

    /// Returns the Directory Table Base of this process..
    pub fn translator(&self) -> &D {
        &self.translator
    }

    /// Replace current translator with a new one.
    pub fn set_translator(&mut self, new_translator: D) -> D {
        core::mem::replace(&mut self.translator, new_translator)
    }

    /// A wrapper around `read_addr64` and `read_addr32` that will use the pointer size of this context's process.
    /// TODO: do this in virt mem
    pub fn read_addr(&mut self, addr: Address) -> PartialResult<Address> {
        match self.proc_arch.bits() {
            64 => self.read_addr64(addr),
            32 => self.read_addr32(addr),
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

#[allow(clippy::needless_option_as_deref)]
impl<T: PhysicalMemory, V: VirtualTranslate2, D: VirtualTranslate3> MemoryView
    for VirtualDma<T, V, D>
{
    fn read_raw_iter<'a>(
        &mut self,
        MemOps {
            inp,
            out,
            mut out_fail,
        }: ReadRawMemOps,
    ) -> Result<()> {
        self.arena.reset();

        let mut translation = BumpVec::with_capacity_in(inp.size_hint().0, &self.arena);
        let phys_mem = &mut self.phys_mem;

        self.vat.virt_to_phys_iter(
            phys_mem,
            &self.translator,
            inp,
            &mut translation.from_extend(),
            &mut (&mut |(_, CTup3(_, meta, buf)): (_, _)| {
                opt_call(out_fail.as_deref_mut(), CTup2(meta, buf))
            })
                .into(),
        );

        MemOps::with_raw(translation.into_iter(), out, out_fail, |data| {
            phys_mem.phys_read_raw_iter(data)
        })
    }

    fn write_raw_iter(
        &mut self,
        MemOps {
            inp,
            out,
            mut out_fail,
        }: WriteRawMemOps,
    ) -> Result<()> {
        self.arena.reset();

        let mut translation = BumpVec::with_capacity_in(inp.size_hint().0, &self.arena);
        let phys_mem = &mut self.phys_mem;

        self.vat.virt_to_phys_iter(
            phys_mem,
            &self.translator,
            inp,
            &mut translation.from_extend(),
            &mut (&mut |(_, CTup3(_, meta, buf)): (_, _)| {
                opt_call(out_fail.as_deref_mut(), CTup2(meta, buf))
            })
                .into(),
        );

        MemOps::with_raw(translation.into_iter(), out, out_fail, |data| {
            phys_mem.phys_write_raw_iter(data)
        })
    }

    fn metadata(&self) -> MemoryViewMetadata {
        let PhysicalMemoryMetadata {
            max_address,
            real_size,
            readonly,
            ..
        } = self.phys_mem.metadata();

        MemoryViewMetadata {
            max_address,
            real_size,
            readonly,
            little_endian: self.proc_arch.endianess() == Endianess::LittleEndian,
            arch_bits: self.proc_arch.bits(),
        }
    }
}

impl<T: PhysicalMemory, V: VirtualTranslate2, D: VirtualTranslate3> VirtualTranslate
    for VirtualDma<T, V, D>
{
    fn virt_to_phys_list(
        &mut self,
        addrs: &[VtopRange],
        mut out: VirtualTranslationCallback,
        mut out_fail: VirtualTranslationFailCallback,
    ) {
        self.vat.virt_to_phys_iter(
            &mut self.phys_mem,
            &self.translator,
            addrs
                .iter()
                .map(|&CTup2(address, size)| CTup3(address, address, size)),
            &mut (&mut |CTup3(a, b, c): CTup3<PhysicalAddress, Address, umem>| {
                out.call(VirtualTranslation {
                    in_virtual: b,
                    size: c,
                    out_physical: a,
                })
            })
                .into(),
            &mut (&mut |(_e, CTup3(from, _, size))| {
                out_fail.call(VirtualTranslationFail { from, size })
            })
                .into(),
        )
    }
}
