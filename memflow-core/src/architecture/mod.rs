/*!
Module for handling different architectures in memflow.

Each architecture is wrapped in the `Architecture` enum
and all function calls are dispatched into their own
architecture specific sub-modules.

Each architecture also has a `ByteOrder` assigned to it.
When reading/writing data from/to the target it is necessary
that memflow know the proper byte order of the target system.
*/

pub mod x86;

mod mmu_spec;

pub use mmu_spec::ArchMMUSpec;

use crate::error::{Error, Result};
use crate::iter::{FnExtend, SplitAtIndex};
use crate::mem::PhysicalMemory;

use crate::types::{Address, PhysicalAddress};
pub use bumpalo::{collections::Vec as BumpVec, Bump};

/// Identifies the byte order of a architecture
///
/// This enum is used when reading/writing to/from the memory of a target system.
/// The memory will be automatically converted to the endianess memflow is currently running on.
///
/// See the [wikipedia article](https://en.wikipedia.org/wiki/Endianness) for more information on the subject.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub enum Endianess {
    /// Little Endianess
    LittleEndian,
    /// Big Endianess
    BigEndian,
}

/// Translates virtual memory to physical using internal translation base (usually a process' dtb)
///
/// This trait abstracts virtual address translation for a single virtual memory scope.
/// On x86 architectures, it is a single `Address` - a CR3 register. But other architectures may
/// use multiple translation bases, or use a completely different translation mechanism (MIPS).
pub trait ScopedVirtualTranslate: Clone + Copy + Send {
    fn virt_to_phys<T: PhysicalMemory>(
        &self,
        mem: &mut T,
        addr: Address,
    ) -> Result<PhysicalAddress> {
        let arena = Bump::new();
        let mut output = None;
        let mut success = FnExtend::new(|elem: (PhysicalAddress, _)| {
            if output.is_none() {
                output = Some(elem.0);
            }
        });
        let mut output_err = None;
        let mut fail = FnExtend::new(|elem: (Error, _, _)| output_err = Some(elem.0));
        self.virt_to_phys_iter(
            mem,
            Some((addr, 1)).into_iter(),
            &mut success,
            &mut fail,
            &arena,
        );
        output.map(Ok).unwrap_or_else(|| Err(output_err.unwrap()))
    }

    fn virt_to_phys_iter<
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        VI: Iterator<Item = (Address, B)>,
        VO: Extend<(PhysicalAddress, B)>,
        FO: Extend<(Error, Address, B)>,
    >(
        &self,
        mem: &mut T,
        addrs: VI,
        out: &mut VO,
        out_fail: &mut FO,
        arena: &Bump,
    );

    fn translation_table_id(&self, address: Address) -> usize;

    fn arch(&self) -> &dyn Architecture;
}

pub trait Architecture: Send + Sync {
    /// Returns the number of bits of a pointers width on a `Architecture`.
    /// Currently this will either return 64 or 32 depending on the pointer width of the target.
    /// This function is handy in cases where you only want to know the pointer width of the target\
    /// but you don't want to match against all architecture.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow_core::architecture::x86::x32_pae;
    ///
    /// let arch = x32_pae::ARCH;
    /// assert_eq!(arch.bits(), 32);
    /// ```
    fn bits(&self) -> u8;

    /// Returns the byte order of an `Architecture`.
    /// This will either be `Endianess::LittleEndian` or `Endianess::BigEndian`.
    ///
    /// In most circumstances this will be `Endianess::LittleEndian` on all x86 and arm architectures.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow_core::architecture::{x86::x32, Endianess};
    ///
    /// let arch = x32::ARCH;
    /// assert_eq!(arch.endianess(), Endianess::LittleEndian);
    /// ```
    fn endianess(&self) -> Endianess;

    /// Returns the smallest page size of an `Architecture`.
    ///
    /// In x86/64 and arm this will always return 4kb.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow_core::architecture::x86::x64;
    /// use memflow_core::types::size;
    ///
    /// let arch = x64::ARCH;
    /// assert_eq!(arch.page_size(), size::kb(4));
    /// ```
    fn page_size(&self) -> usize;

    /// Returns the `usize` of a pointers width on a `Architecture`.
    ///
    /// This function will return the pointer width as a `usize` value.
    /// See `Architecture::bits()` for more information.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow_core::architecture::x86::x32;
    ///
    /// let arch = x32::ARCH;
    /// assert_eq!(arch.size_addr(), 4);
    /// ```
    fn size_addr(&self) -> usize;

    /// Returns the address space range in bits for the `Architecture`.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow_core::architecture::x86::x32_pae;
    ///
    /// let arch = x32_pae::ARCH;
    /// assert_eq!(arch.address_space_bits(), 36);
    ///
    /// ```
    fn address_space_bits(&self) -> u8;
}

impl<'a> std::fmt::Debug for &'a dyn Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("&dyn Architecture")
            .field("bits", &self.bits())
            .field("endianess", &self.endianess())
            .field("page_size", &self.page_size())
            .field("size_addr", &self.size_addr())
            .field("address_space_bits", &self.address_space_bits())
            .finish()
    }
}
