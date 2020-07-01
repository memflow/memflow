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
pub mod translate_data;

#[macro_use]
pub mod vtop_macros;

use mmu_spec::ArchMMUSpec;
use translate_data::{TranslateData, TranslateVec};

use crate::error::{Error, Result};
use crate::iter::{PageChunks, SplitAtIndex};
use std::convert::TryFrom;

use crate::mem::{PhysicalMemory, PhysicalReadData};
use crate::types::{Address, PageType, PhysicalAddress};

use bumpalo::{collections::Vec as BumpVec, Bump};
use byteorder::{ByteOrder, LittleEndian};
use vector_trees::{BVecTreeMap as BTreeMap, Vector};

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
            _ => Err(Error::InvalidArchitecture),
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
    use flow_core::types::size;

    pub fn test() {
        let arch = Architecture::X64;
        assert_eq!(arch.page_size(), size::kb(4));
    }
    ```
    */
    pub fn page_size(self) -> usize {
        self.get_mmu_spec().page_size_level(1)
    }

    /**
    Returns the `usize` of a pointers width on a `Architecture`.

    This function will return the pointer width as a `usize` value.
    See `Architecture::bits()` for more information.

    # Examples

    ```
    use flow_core::architecture::Architecture;

    pub fn test() {
        let arch = Architecture::X86;
        assert_eq!(arch.size_addr(), 4);
    }
    ```
    */
    pub fn size_addr(self) -> usize {
        self.get_mmu_spec().addr_size as usize
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
        let mut vec_fail = BumpVec::new_in(&arena);
        self.virt_to_phys_iter(
            mem,
            dtb,
            Some((addr, false)).into_iter(),
            &mut vec,
            &mut vec_fail,
            &arena,
        );
        if let Some(ret) = vec.pop() {
            Ok(ret.0)
        } else {
            Err(vec_fail.pop().unwrap().0)
        }
    }

    pub fn virt_to_phys_iter<
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        VI: Iterator<Item = (Address, B)>,
        VO: Extend<(PhysicalAddress, B)>,
        FO: Extend<(Error, Address, B)>,
    >(
        self,
        mem: &mut T,
        dtb: Address,
        addrs: VI,
        out: &mut VO,
        out_fail: &mut FO,
        arena: &Bump,
    ) {
        match self {
            Architecture::Null => {
                out.extend(addrs.map(|(addr, buf)| (PhysicalAddress::from(addr), buf)))
            }
            _ => Self::virt_to_phys_iter_with_mmu(
                mem,
                dtb,
                addrs,
                out,
                out_fail,
                arena,
                self.get_mmu_spec(),
            ),
        }
    }

    //TODO: Clean this up to have less args
    #[allow(clippy::too_many_arguments)]
    fn read_pt_address_iter<'a, T, B, V, FO>(
        mem: &mut T,
        spec: &ArchMMUSpec,
        step: usize,
        addr_map: &mut BTreeMap<V, Address, usize>,
        addrs: &mut TranslateVec<'a, B>,
        pt_buf: &mut BumpVec<u8>,
        pt_read: &mut BumpVec<PhysicalReadData>,
        err_out: &mut FO,
    ) -> Result<()>
    where
        T: PhysicalMemory + ?Sized,
        FO: Extend<(Error, Address, B)>,
        V: Vector<vector_trees::btree::BVecTreeNode<Address, usize>>,
    {
        //TODO: use spec.pt_leaf_size(step) (need to handle LittleEndian::read_u64)
        let pte_size = 8;
        let page_size = spec.pt_leaf_size(step);

        pt_buf.clear();
        pt_buf.resize(pte_size * addrs.len(), 0);

        debug_assert!(pt_read.is_empty());

        //This is safe, because pt_read gets cleared at the end of the function
        let pt_read: &mut BumpVec<PhysicalReadData> = unsafe { std::mem::transmute(pt_read) };

        for (chunk, &(addr, _)) in pt_buf.chunks_exact_mut(pte_size).zip(addrs.iter()) {
            pt_read.push((
                PhysicalAddress::with_page(addr, PageType::PAGE_TABLE, page_size),
                chunk,
            ));
        }

        mem.phys_read_raw_list(pt_read)?;

        //Not clearing would eliminate null reads
        //addr_map.clear();

        //Filter out duplicate reads
        for i in (0..addrs.len()).rev() {
            let (orig_pt_addr, vec) = addrs.swap_remove(i);
            let (_, buf) = pt_read.swap_remove(i);
            let pt_addr = Address::from(LittleEndian::read_u64(&buf[..]));

            if spec.pte_addr_mask(orig_pt_addr, step) != spec.pte_addr_mask(pt_addr, step)
                && addr_map.get_mut(&pt_addr).is_none()
            {
                addr_map.insert(pt_addr, i);
                addrs.push((pt_addr, vec));
                continue;
            }

            err_out.extend(
                vec.into_iter()
                    .map(|entry| (Error::VirtualTranslate, entry.addr, entry.buf)),
            );
        }

        addr_map.clear();
        pt_read.clear();

        Ok(())
    }

    fn virt_to_phys_iter_with_mmu<T, B, VI, VO, FO>(
        mem: &mut T,
        dtb: Address,
        addrs: VI,
        out: &mut VO,
        out_fail: &mut FO,
        arena: &Bump,
        spec: ArchMMUSpec,
    ) where
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        VI: Iterator<Item = (Address, B)>,
        VO: Extend<(PhysicalAddress, B)>,
        FO: Extend<(Error, Address, B)>,
    {
        vtop_trace!("virt_to_phys_iter_with_mmu");

        let mut data_to_translate = BumpVec::new_in(arena);
        let mut data_pt_read: BumpVec<PhysicalReadData> = BumpVec::new_in(arena);
        let mut data_pt_buf = BumpVec::new_in(arena);
        let mut data_to_translate_map = BTreeMap::new_in(BumpVec::new_in(arena));

        data_to_translate.push((dtb, {
            let mut vec = BumpVec::new_in(arena);
            addrs.for_each(|data| spec.virt_addr_filter(data, &mut vec, out_fail));
            vec
        }));

        for pt_step in 0..spec.split_count() {
            vtop_trace!(
                "pt_step = {}, data_to_translate.len() = {:x}",
                pt_step,
                data_to_translate.len()
            );

            let next_page_size = spec.page_size_step_unchecked(pt_step + 1);

            vtop_trace!("next_page_size = {:x}", next_page_size);

            //Loop through the data in reverse order to allow the data buffer grow on the back when
            //memory regions are split
            for i in (0..data_to_translate.len()).rev() {
                let (pt_addr, vec) = data_to_translate.swap_remove(i);
                vtop_trace!("checking pt_addr={:x}, elems={:x}", pt_addr, vec.len());

                if !spec.check_entry(pt_addr, pt_step)
                    || (pt_step > 0 && dtb.as_u64() == spec.pte_addr_mask(pt_addr, pt_step))
                {
                    //There has been an error in translation, push it to output with the associated buf
                    vtop_trace!("check_entry failed");
                    out_fail.extend(
                        vec.into_iter()
                            .map(|entry| (Error::VirtualTranslate, entry.addr, entry.buf)),
                    );
                } else if spec.is_final_mapping(pt_addr, pt_step) {
                    //We reached an actual page. The translation was successful
                    vtop_trace!("found final mapping: {:x}", pt_addr);
                    out.extend(vec.into_iter().map(|entry| {
                        (spec.get_phys_page(pt_addr, entry.addr, pt_step), entry.buf)
                    }));
                } else {
                    //We still need to continue the page walk
                    for e in vec.into_iter() {
                        //As an optimization, divide and conquer the input memory regions.
                        //Potential speedups of 4x for up to 2M sequential regions, and 2x for up to 1G sequential regions,
                        //assuming all pages are 4kb sized.
                        //TODO: have the list sorted so we can split it up more efficiently
                        for (addr, buf) in e.buf.page_chunks(e.addr, next_page_size) {
                            let pt_addr = spec.vtop_step(pt_addr, addr, pt_step);
                            vtop_trace!("pt_addr = {:x}", pt_addr);

                            let entry = data_to_translate_map.get_mut(&pt_addr);

                            let eidx = if let Some(idx) = entry {
                                *idx
                            } else {
                                let ret = data_to_translate.len();
                                data_to_translate.push((pt_addr, BumpVec::new_in(arena)));
                                data_to_translate_map.insert(pt_addr, ret);
                                ret
                            };

                            /*data_to_translate_map.entry(pt_addr).or_insert_with(|| {
                                let ret = data_to_translate.len();
                                data_to_translate.push((pt_addr, BumpVec::new_in(arena), [0; 8]));
                                ret
                            });*/

                            let e = data_to_translate.get_mut(eidx).unwrap();

                            e.1.push(TranslateData { addr, buf });
                        }
                    }
                }
            }

            if data_to_translate.is_empty() {
                break;
            }

            if let Err(err) = Self::read_pt_address_iter(
                mem,
                &spec,
                pt_step,
                &mut data_to_translate_map,
                &mut data_to_translate,
                &mut data_pt_buf,
                &mut data_pt_read,
                out_fail,
            ) {
                vtop_trace!("read_pt_address_iter failure: {}", err);
                out_fail.extend(
                    data_to_translate
                        .into_iter()
                        .flat_map(|(_, vec)| vec.into_iter())
                        .map(|data| (err, data.addr, data.buf)),
                );
                return;
            }
        }

        debug_assert!(data_to_translate.is_empty());
    }
}
