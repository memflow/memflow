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
use translate_data::{TranslateVec, TranslationChunk};

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
        addr_map: &mut BTreeMap<V, Address, ()>,
        addrs: &mut TranslateVec<'a, B>,
        pt_buf: &mut BumpVec<u8>,
        pt_read: &mut BumpVec<PhysicalReadData>,
        err_out: &mut FO,
    ) -> Result<()>
    where
        T: PhysicalMemory + ?Sized,
        FO: Extend<(Error, Address, B)>,
        V: Vector<vector_trees::btree::BVecTreeNode<Address, ()>>,
        B: SplitAtIndex,
    {
        //TODO: use spec.pt_leaf_size(step) (need to handle LittleEndian::read_u64)
        let pte_size = 8;
        let page_size = spec.pt_leaf_size(step);

        //pt_buf.clear();
        pt_buf.resize(pte_size * addrs.len(), 0);

        debug_assert!(pt_read.is_empty());

        //This is safe, because pt_read gets cleared at the end of the function
        let pt_read: &mut BumpVec<PhysicalReadData> = unsafe { std::mem::transmute(pt_read) };

        for (chunk, tr_chunk) in pt_buf.chunks_exact_mut(pte_size).zip(addrs.iter()) {
            pt_read.push((
                PhysicalAddress::with_page(tr_chunk.pt_addr, PageType::PAGE_TABLE, page_size),
                chunk,
            ));
        }

        mem.phys_read_raw_list(pt_read)?;

        //Filter out duplicate reads
        //Ideally, we would want to append all duplicates to the existing list, but they would mostly
        //only occur, in strange kernel side situations when building the page map,
        //and having such handling may end up highly inefficient (due to having to use map, and remapping it)
        addr_map.clear();

        //Okay, so this is extremely useful in one element reads.
        //We kind of have a local on-stack cache to check against
        //before a) checking in the set, and b) pushing to the set
        let mut prev_addr: Option<Address> = None;

        for i in (0..addrs.len()).rev() {
            let mut chunk = addrs.swap_remove(i);
            let (_, buf) = pt_read.swap_remove(i);
            let pt_addr = Address::from(LittleEndian::read_u64(&buf[..]));

            if spec.pte_addr_mask(chunk.pt_addr, step) != spec.pte_addr_mask(pt_addr, step)
                && (prev_addr.is_none()
                    || (prev_addr.unwrap() != pt_addr && !addr_map.contains_key(&pt_addr)))
            {
                chunk.pt_addr = pt_addr;

                if let Some(pa) = prev_addr {
                    addr_map.insert(pa, ());
                }

                prev_addr = Some(pt_addr);
                addrs.push(chunk);
                continue;
            }

            err_out.extend(
                chunk
                    .vec
                    .into_iter()
                    .map(|entry| (Error::VirtualTranslate, entry.addr, entry.buf)),
            );
        }

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

        //TODO: Calculate and reserve enough data in the data_to_translate vectors
        //TODO: precalc vtop_step bit split sum / transform the splits to a lookup table
        //TODO: Improve filtering speed (vec reserve)
        //TODO: Optimize BTreeMap

        data_to_translate.push(TranslationChunk::new(dtb, {
            let mut vec = BumpVec::with_capacity_in(addrs.size_hint().0, arena);
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
                let tr_chunk = data_to_translate.swap_remove(i);
                vtop_trace!(
                    "checking pt_addr={:x}, elems={:x}",
                    tr_chunk.pt_addr,
                    tr_chunk.vec.len()
                );

                if !spec.check_entry(tr_chunk.pt_addr, pt_step)
                    || (pt_step > 0
                        && dtb.as_u64() == spec.pte_addr_mask(tr_chunk.pt_addr, pt_step))
                {
                    //There has been an error in translation, push it to output with the associated buf
                    vtop_trace!("check_entry failed");
                    out_fail.extend(
                        tr_chunk
                            .vec
                            .into_iter()
                            .map(|entry| (Error::VirtualTranslate, entry.addr, entry.buf)),
                    );
                } else if spec.is_final_mapping(tr_chunk.pt_addr, pt_step) {
                    //We reached an actual page. The translation was successful
                    vtop_trace!("found final mapping: {:x}", tr_chunk.pt_addr);
                    let pt_addr = tr_chunk.pt_addr;
                    out.extend(tr_chunk.vec.into_iter().map(|entry| {
                        (spec.get_phys_page(pt_addr, entry.addr, pt_step), entry.buf)
                    }));
                } else {
                    //We still need to continue the page walk

                    let min_addr = tr_chunk.min_addr();

                    //As an optimization, divide and conquer the input memory regions.
                    //VTOP speedup is insane. Visible in large sequential or chunked reads.
                    for (_, (_, mut chunk)) in
                        (arena, tr_chunk).page_chunks(min_addr, next_page_size)
                    {
                        let pt_addr = spec.vtop_step(chunk.pt_addr, chunk.min_addr(), pt_step);
                        chunk.pt_addr = pt_addr;
                        data_to_translate.push(chunk);
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
                        .flat_map(|chunk| chunk.vec.into_iter())
                        .map(|data| (err, data.addr, data.buf)),
                );
                return;
            }
        }

        debug_assert!(data_to_translate.is_empty());
    }
}
