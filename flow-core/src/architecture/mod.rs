/*!
This module contains all architecture definitions currently
supported by memflow.

Each architecture is wrapped in the `Architecture` enum
and all function calls are dispatched into their own
architecture specific sub-modules.

Each architecture also has a `ByteOrder` assigned to it.
When reading/writing data from/to the target it is necessary
that memflow know the proper byte order of the target system.
*/

pub mod x64;
pub mod x86;
pub mod x86_pae;

pub mod mmu_spec;

use mmu_spec::ArchMMUSpec;

use crate::error::{Error, Result};
use crate::iter::{PageChunks, SplitAtIndex};
use std::convert::TryFrom;

use log::trace;

use crate::mem::PhysicalMemory;
use crate::types::{Address, Length, PageType, PhysicalAddress};

use bumpalo::{collections::Vec as BumpVec, Bump};
use byteorder::{ByteOrder, LittleEndian};

/**
Identifies the byte order of a architecture
*/
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Endianess {
    /// little endianess
    LittleEndian,
    /// big endianess
    BigEndian,
}

/**
Describes the architecture to of a target.
The architecture will contain information about the pointer width,
byte order, page size and also how to translate virtual to physical memory.
*/
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Architecture {
    /**
    An empty architecture with some sensible defaults and no virt_to_phys translation.
    This is usually most useful when running automated tests.
    */
    Null,
    /// x86_64 architecture.
    X64,
    /**
    x86 architecture with physical address extensions.
    See [here](https://en.wikipedia.org/wiki/Physical_Address_Extension) for more information on the subject.
    */
    X86Pae,
    /// x86 architecture.
    X86,
}

/**
Converts a `u8` value to an `Architecture`.
This is usually helpful when serializing / deserializing data in a safe way.

# Examples

```
use flow_core::architecture::Architecture;
use std::convert::TryFrom;

pub fn test() {
    let arch = Architecture::try_from(1).unwrap();
    assert_eq!(arch, Architecture::X64);
}
```
*/
impl TryFrom<u8> for Architecture {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Architecture::Null),
            1 => Ok(Architecture::X64),
            2 => Ok(Architecture::X86Pae),
            3 => Ok(Architecture::X86),
            _ => Err(Error::new("Invalid Architecture value")),
        }
    }
}

#[allow(dead_code)]
impl Architecture {
    /**
    Converts a `Architecture` to a corresponding `u8` value.
    This is usually helpful when serializing / deserializing data in a safe way.

    # Examples

    ```
    use flow_core::architecture::Architecture;

    pub fn test() {
        let arch = Architecture::X64;
        assert_eq!(arch.as_u8(), 1);
    }
    ```
    */
    pub fn as_u8(self) -> u8 {
        match self {
            Architecture::Null => 0,
            Architecture::X64 => 1,
            Architecture::X86Pae => 2,
            Architecture::X86 => 3,
        }
    }

    /**
    Returns the number of bits of a pointers width on a `Architecture`.
    Currently this will either return 64 or 32 depending on the pointer width of the target.
    This function is handy in cases where you only want to know the pointer width of the target\
    but you don't want to match against all architecture.

    # Examples

    ```
    use flow_core::architecture::Architecture;

    pub fn test() {
        let arch = Architecture::X86Pae;
        assert_eq!(arch.bits(), 32);
    }
    ```
    */
    pub fn bits(self) -> u8 {
        match self {
            Architecture::Null => x64::bits(),
            Architecture::X64 => x64::bits(),
            Architecture::X86Pae => x86_pae::bits(),
            Architecture::X86 => x86::bits(),
        }
    }

    /**
    Returns a structure representing all paramters of the architecture's memory managment unit
    This structure represents various value used in virtual to physical address translation.
    */
    pub fn get_mmu_spec(self) -> ArchMMUSpec {
        match self {
            Architecture::X64 => x64::get_mmu_spec(),
            Architecture::X86 => x86::get_mmu_spec(),
            Architecture::X86Pae => x86_pae::get_mmu_spec(),
            _ => x64::get_mmu_spec(),
        }
    }

    /**
    Returns the byte order of an `Architecture`.
    This will either be `Endianess::LittleEndian` or `Endianess::BigEndian`.

    In most circumstances this will be `Endianess::LittleEndian` on all x86 and arm architectures.

    # Examples

    ```
    use flow_core::architecture::{Architecture, Endianess};

    pub fn test() {
        let arch = Architecture::X86;
        assert_eq!(arch.endianess(), Endianess::LittleEndian);
    }
    ```
    */
    pub fn endianess(self) -> Endianess {
        match self {
            Architecture::Null => x64::endianess(),
            Architecture::X64 => x64::endianess(),
            Architecture::X86Pae => x86_pae::endianess(),
            Architecture::X86 => x86::endianess(),
        }
    }

    /**
    Returns the smallest page size of an `Architecture`.

    In x86/64 and arm this will always return 4kb.

    # Examples

    ```
    use flow_core::architecture::Architecture;
    use flow_core::types::Length;

    pub fn test() {
        let arch = Architecture::X64;
        assert_eq!(arch.page_size(), Length::from_kb(4));
    }
    ```
    */
    pub fn page_size(self) -> Length {
        self.get_mmu_spec().page_size_level(1)
    }

    /**
    Returns the `Length` of a pointers width on a `Architecture`.

    This function will return the pointer width as a `Length` value.
    See `Architecture::bits()` for more information.

    # Examples

    ```
    use flow_core::architecture::Architecture;
    use flow_core::types::Length;

    pub fn test() {
        let arch = Architecture::X86;
        assert_eq!(arch.len_addr(), Length::from(4));
    }
    ```
    */
    pub fn len_addr(self) -> Length {
        match self {
            Architecture::Null => x64::len_addr(),
            Architecture::X64 => x64::len_addr(),
            Architecture::X86Pae => x86_pae::len_addr(),
            Architecture::X86 => x86::len_addr(),
        }
    }

    /**
    This function will do a virtual to physical memory translation for the `Architecture`.

    TODO: add more info how virt_to_phys works

    # Examples

    TODO: add example
    */
    pub fn virt_to_phys<T: PhysicalMemory>(
        self,
        mem: &mut T,
        dtb: Address,
        addr: Address,
    ) -> Result<PhysicalAddress> {
        let arena = Bump::new();
        let mut vec = BumpVec::new_in(&arena);
        self.virt_to_phys_iter(mem, dtb, Some((addr, false)).into_iter(), &mut vec, &arena);
        vec.pop().unwrap().0
    }

    pub fn virt_to_phys_iter<
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        VI: Iterator<Item = (Address, B)>,
        OV: Extend<(Result<PhysicalAddress>, Address, B)>,
    >(
        self,
        mem: &mut T,
        dtb: Address,
        addrs: VI,
        out: &mut OV,
        arena: &Bump,
    ) {
        match self {
            Architecture::Null => {
                out.extend(addrs.map(|(addr, buf)| (Ok(PhysicalAddress::from(addr)), addr, buf)))
            }
            _ => Self::virt_to_phys_iter_with_mmu(mem, dtb, addrs, out, arena, self.get_mmu_spec()),
        }
    }

    fn read_pt_address_iter<T: PhysicalMemory + ?Sized, B>(
        mem: &mut T,
        spec: &ArchMMUSpec,
        step: usize,
        addrs: &mut BumpVec<(Address, B, Address, [u8; 8])>,
    ) {
        let page_size = spec.pt_leaf_size(step);
        if cfg!(feature = "trace_mmu") {
            trace!("pt_leaf_size = {}", page_size);
        }

        mem.phys_read_iter(addrs.iter_mut().map(|(_, _, pt_addr, arr)| {
            arr.iter_mut().for_each(|x| *x = 0);
            (
                PhysicalAddress::with_page(*pt_addr, PageType::PAGE_TABLE, page_size),
                &mut arr[..],
            )
        }))
        .ok();

        addrs
            .iter_mut()
            .for_each(|(_, _, pt_addr, buf)| *pt_addr = Address::from(LittleEndian::read_u64(buf)));
    }

    fn virt_to_phys_iter_with_mmu<T, B, VI, OV>(
        mem: &mut T,
        dtb: Address,
        addrs: VI,
        out: &mut OV,
        arena: &Bump,
        spec: ArchMMUSpec,
    ) where
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        VI: Iterator<Item = (Address, B)>,
        OV: Extend<(Result<PhysicalAddress>, Address, B)>,
    {
        if cfg!(feature = "trace_mmu") {
            trace!("virt_to_phys_iter_with_mmu");
        }

        //TODO: build a tree to eliminate duplicate phys reads with multiple elements
        let mut data_to_translate = BumpVec::new_in(arena);

        data_to_translate.extend(addrs.map(|(addr, buf)| {
            (
                addr, buf, dtb,
                [0; 8], //TODO: What to do with this? PTE size may be 128-bit in further MMU impls
            )
        }));

        for pt_step in 0..spec.split_count() {
            if cfg!(feature = "trace_mmu") {
                trace!("pt_step = {}", pt_step);
            }

            let next_page_size = spec.page_size_step_unchecked(pt_step + 1);
            if cfg!(feature = "trace_mmu") {
                trace!("next_page_size = {}", next_page_size);
            }

            //Loop through the data in reverse order to allow the data buffer grow on the back when
            //memory regions are split
            for i in (0..data_to_translate.len()).rev() {
                let (addr, buf, pt_addr, tmp_arr) = data_to_translate.swap_remove(i);
                if cfg!(feature = "trace_mmu") {
                    trace!(
                        "checking addr={}; pt_addr={}; tmp_arr={:?}",
                        addr,
                        pt_addr,
                        tmp_arr
                    );
                }

                if !spec.check_entry(pt_addr, pt_step) {
                    //There has been an error in translation, push it to output with the associated buf
                    if cfg!(feature = "trace_mmu") {
                        trace!("check_entry failed");
                    }
                    out.extend(
                        Some((Err(Error::new("virt_to_phys failed")), addr, buf)).into_iter(),
                    );
                } else if spec.is_final_mapping(pt_addr, pt_step) {
                    //We reached an actual page. The translation was successful
                    if cfg!(feature = "trace_mmu") {
                        trace!(
                            "found final mapping: {:?}",
                            spec.get_phys_page(pt_addr, addr, pt_step)
                        );
                    }
                    out.extend(
                        Some((Ok(spec.get_phys_page(pt_addr, addr, pt_step)), addr, buf))
                            .into_iter(),
                    );
                } else {
                    //We still need to continue the page walk

                    //As an optimization, divide and conquer the input memory regions.
                    //Potential speedups of 4x for up to 2M sequential regions, and 2x for up to 1G sequential regions,
                    //assuming all pages are 4kb sized.
                    for (addr, buf) in buf.page_chunks(addr, next_page_size) {
                        let pt_addr = spec.vtop_step(pt_addr, addr, pt_step);
                        if cfg!(feature = "trace_mmu") {
                            trace!("pt_addr = {}", pt_addr);
                        }
                        data_to_translate.push((addr, buf, pt_addr, tmp_arr));
                    }
                }
            }

            if data_to_translate.is_empty() {
                break;
            } else {
                Self::read_pt_address_iter(mem, &spec, pt_step, &mut data_to_translate);
            }
        }

        debug_assert!(data_to_translate.is_empty());
    }
}
