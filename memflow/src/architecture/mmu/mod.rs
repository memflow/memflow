pub(crate) mod def;
pub(crate) mod spec;
pub(crate) mod translate_data;

use crate::error::Error;
use crate::iter::SplitAtIndex;
use crate::types::Address;
pub(crate) use def::ArchMMUDef;
pub(crate) use fixed_slice_vec::FixedSliceVec as MVec;
pub(crate) use spec::ArchMMUSpec;
use translate_data::{TranslateDataVec, TranslateVec, TranslationChunk};

pub trait MMUTranslationBase: Clone + Copy + core::fmt::Debug {
    /// Retrieves page table address by virtual address
    fn get_pt_by_virt_addr(&self, address: Address) -> Address;

    /// Retrieves page table address, and its index by index within
    /// For instance, on ARM index 257 would return kernel page table
    /// address, and index 1. On X86, however, this is a no-op that returns
    /// underlying page table Address and `idx`.
    fn get_pt_by_index(&self, idx: usize) -> (Address, usize);

    /// Retrieves number of page tables used by translation base. 1 on X86,
    /// 1-2 on ARM (Win32 ARM merges both page tables)
    fn pt_count(&self) -> usize;

    fn virt_addr_filter<B: SplitAtIndex, O: Extend<(Error, Address, B)>>(
        &self,
        spec: &ArchMMUSpec,
        addr: (Address, B),
        work_group: (&mut TranslationChunk<Self>, &mut TranslateDataVec<B>),
        out_fail: &mut O,
    );

    fn fill_init_chunk<VI, FO, B>(
        &self,
        spec: &ArchMMUSpec,
        out_fail: &mut FO,
        addrs: &mut VI,
        (next_work_addrs, tmp_addrs): (&mut TranslateDataVec<B>, &mut TranslateDataVec<B>),
        work_vecs: &mut (TranslateVec, TranslateDataVec<B>),
        wait_vecs: &mut (TranslateVec, TranslateDataVec<B>),
    ) where
        VI: Iterator<Item = (Address, B)>,
        FO: Extend<(Error, Address, B)>,
        B: SplitAtIndex,
    {
        let mut init_chunk = TranslationChunk::new(*self);

        let working_addr_count = work_vecs.1.capacity();

        for (_, data) in (0..working_addr_count).zip(addrs) {
            self.virt_addr_filter(spec, data, (&mut init_chunk, next_work_addrs), out_fail);
            if init_chunk.next_max_addr_count(spec) >= working_addr_count {
                break;
            }
        }

        if init_chunk.addr_count > 0 {
            init_chunk.split_chunk(spec, (next_work_addrs, tmp_addrs), work_vecs, wait_vecs);
        }
    }
}

impl MMUTranslationBase for Address {
    fn get_pt_by_virt_addr(&self, _: Address) -> Address {
        *self
    }

    fn get_pt_by_index(&self, idx: usize) -> (Address, usize) {
        (*self, idx)
    }

    fn pt_count(&self) -> usize {
        1
    }

    fn virt_addr_filter<B, O>(
        &self,
        spec: &ArchMMUSpec,
        addr: (Address, B),
        work_group: (&mut TranslationChunk<Self>, &mut TranslateDataVec<B>),
        out_fail: &mut O,
    ) where
        B: SplitAtIndex,
        O: Extend<(Error, Address, B)>,
    {
        spec.virt_addr_filter(addr, work_group, out_fail);
    }
}
