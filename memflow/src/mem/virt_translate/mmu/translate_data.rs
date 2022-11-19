use crate::iter::SplitAtIndex;
use crate::types::{umem, Address};

use super::{ArchMmuDef, ArchMmuSpec, MmuTranslationBase};

use std::cmp::Ordering;

use super::MVec;

pub type TranslateVec<'a> = MVec<'a, TranslationChunk<Address>>;
pub type TranslateDataVec<'a, T> = MVec<'a, TranslateData<T>>;

unsafe fn shorten_datavec_lifetime<'a: 'b, 'b, O>(
    r: &'b mut TranslateDataVec<'a, O>,
) -> &'b mut TranslateDataVec<'b, O> {
    std::mem::transmute(r)
}

unsafe fn shorten_pair_lifetime<'a: 't, 'b: 't, 't, O>(
    r: &'t mut (TranslateVec<'a>, TranslateDataVec<'b, O>),
) -> &'t mut (TranslateVec<'t>, TranslateDataVec<'t, O>) {
    std::mem::transmute(r)
}

#[derive(Debug)]
pub struct TranslateData<T> {
    pub addr: Address,
    pub meta_addr: Address,
    pub buf: T,
}

impl<T: SplitAtIndex> TranslateData<T> {
    pub fn split_at_address(self, addr: Address) -> (Option<Self>, Option<Self>) {
        let sub = self.addr.to_umem();
        self.split_at(addr.to_umem().saturating_sub(sub))
    }

    pub fn split_at_address_rev(self, addr: Address) -> (Option<Self>, Option<Self>) {
        let base = self.addr + self.length();
        self.split_at_rev(base.to_umem().saturating_sub(addr.to_umem()))
    }
}

impl<T: SplitAtIndex> Ord for TranslateData<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.addr.cmp(&other.addr)
    }
}

impl<T: SplitAtIndex> Eq for TranslateData<T> {}

impl<T> PartialOrd for TranslateData<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.addr.partial_cmp(&other.addr)
    }
}

impl<T> PartialEq for TranslateData<T> {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr
    }
}

impl<T: SplitAtIndex> SplitAtIndex for TranslateData<T> {
    fn split_at(self, idx: umem) -> (Option<Self>, Option<Self>)
    where
        Self: Sized,
    {
        let addr = self.addr;
        let meta_addr = self.meta_addr;
        let (bleft, bright) = self.buf.split_at(idx);

        (
            bleft.map(|buf| TranslateData {
                addr,
                meta_addr,
                buf,
            }),
            bright.map(|buf| TranslateData {
                buf,
                addr: addr + idx,
                meta_addr: meta_addr + idx,
            }),
        )
    }

    unsafe fn split_at_mut(&mut self, idx: umem) -> (Option<Self>, Option<Self>)
    where
        Self: Sized,
    {
        let addr = self.addr;
        let meta_addr = self.meta_addr;
        let (bleft, bright) = self.buf.split_at_mut(idx);

        (
            bleft.map(|buf| TranslateData {
                addr,
                meta_addr,
                buf,
            }),
            bright.map(|buf| TranslateData {
                buf,
                addr: addr + idx,
                meta_addr: meta_addr + idx,
            }),
        )
    }

    fn length(&self) -> umem {
        self.buf.length()
    }

    fn size_hint(&self) -> usize {
        self.buf.size_hint()
    }
}

bitflags! {
    #[repr(transparent)]
    #[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
    #[cfg_attr(feature = "abi_stable", derive(::abi_stable::StableAbi))]
    pub struct FlagsType: u8 {
        const NONE = 0b00;
        // Maps MMUDef's writeable_bit
        const WRITEABLE = 0b01;
        // Maps MMUDef's nx_bit
        const NX = 0b10;
    }
}

/// Abstracts away a list of TranslateData in a splittable manner
#[derive(Debug)]
pub struct TranslationChunk<T> {
    pub pt_addr: T,
    pub addr_count: usize,
    pub min_addr: Address,
    max_addr: Address,
    pub step: usize,
    pub prev_flags: FlagsType,
}

impl FlagsType {
    pub fn nx(mut self, flag: bool) -> Self {
        self &= !(FlagsType::NX);
        if flag {
            self | FlagsType::NX
        } else {
            self
        }
    }

    pub fn writeable(mut self, flag: bool) -> Self {
        self &= !(FlagsType::WRITEABLE);
        if flag {
            self | FlagsType::WRITEABLE
        } else {
            self
        }
    }
}

impl TranslationChunk<Address> {
    pub fn update_flags(&mut self, mmu_def: &ArchMmuDef) {
        self.prev_flags = FlagsType::NONE
            .writeable((mmu_def.writeable_bit)(
                self.pt_addr,
                self.prev_flags.contains(FlagsType::WRITEABLE),
            ))
            .nx((mmu_def.nx_bit)(
                self.pt_addr,
                self.prev_flags.contains(FlagsType::NX),
            ));
    }
}

impl<T> TranslationChunk<T> {
    pub fn new(pt_addr: T, prev_flags: FlagsType) -> Self {
        let (min, max) = (!0u64, 0u64);
        Self::with_minmax(pt_addr, prev_flags, min.into(), max.into())
    }

    pub fn with_minmax(
        pt_addr: T,
        prev_flags: FlagsType,
        min_addr: Address,
        max_addr: Address,
    ) -> Self {
        Self {
            pt_addr,
            addr_count: 0,
            step: 0,
            min_addr,
            max_addr,
            prev_flags,
        }
    }
}

impl<T: MmuTranslationBase> TranslationChunk<T> {
    /// Pushes data to stack updating min/max bounds
    pub fn push_data<U: SplitAtIndex>(
        &mut self,
        data: TranslateData<U>,
        stack: &mut TranslateDataVec<U>,
    ) {
        self.min_addr = std::cmp::min(self.min_addr, data.addr);
        self.max_addr = std::cmp::max(self.max_addr, data.addr + data.length());
        self.addr_count += 1;
        stack.push(data);
    }

    /// Pops the address from stack without modifying bounds
    pub fn pop_data<U: SplitAtIndex>(
        &mut self,
        stack: &mut TranslateDataVec<U>,
    ) -> Option<TranslateData<U>> {
        if self.addr_count > 0 {
            self.addr_count -= 1;
            stack.pop()
        } else {
            None
        }
    }

    pub fn next_max_addr_count(&self, spec: &ArchMmuSpec) -> umem {
        let step_size = spec.page_size_step_unchecked(self.step + 1);

        let addr_diff = self.max_addr.wrapping_sub(self.min_addr).to_umem();
        let add = (addr_diff % step_size != 0) as umem;

        self.addr_count as umem * (addr_diff / step_size + add)
    }

    /// Splits the chunk into multiple smaller ones for the next VTOP step.
    pub fn split_chunk<U: SplitAtIndex>(
        mut self,
        spec: &ArchMmuSpec,
        (addr_stack, tmp_addr_stack): (&mut TranslateDataVec<U>, &mut TranslateDataVec<U>),
        out_target: &mut (TranslateVec, TranslateDataVec<U>),
        wait_target: &mut (TranslateVec, TranslateDataVec<U>),
    ) {
        // Safety:
        // We ideally would not do this, but honestly this is a better alternative
        // to lifetime torture.
        // The input vecs are allocated by the same functions, and the data that's being held
        // should not really be lifetime dependent in the context of VTOP
        let mut addr_stack = unsafe { shorten_datavec_lifetime(addr_stack) };
        let mut tmp_addr_stack = unsafe { shorten_datavec_lifetime(tmp_addr_stack) };
        let mut out_target = unsafe { shorten_pair_lifetime(out_target) };
        let mut wait_target = unsafe { shorten_pair_lifetime(wait_target) };

        let align_as = spec.page_size_step_unchecked(self.step);
        let step_size = spec.page_size_step_unchecked(self.step + 1);

        //TODO: mask out the addresses to limit them within address space
        //this is in particular for the first step where addresses are split between positive and
        //negative sides
        let upper = (self.max_addr - 1usize).as_mem_aligned(step_size).to_umem();
        let lower = self.min_addr.as_mem_aligned(step_size).to_umem();

        let mut cur_max_addr: umem = !0;

        // Walk in reverse so that lowest addresses always end up
        // first in the stack. This preserves translation order
        for (cnt, addr) in (0..=((upper - lower) / step_size))
            .map(|i| upper - i * step_size)
            .enumerate()
        {
            if addr > cur_max_addr {
                continue;
            }

            cur_max_addr = 0;

            // Also, we need to push the upper elements to the waiting stack preemptively...
            // This might result in slight performance loss, but keeps the order
            let remaining = (addr - lower) / step_size + 1;

            let (chunks_out, addrs_out) = if out_target.0.capacity() as umem
                >= out_target.0.len() as umem + remaining
                && out_target.1.capacity() as umem
                    >= out_target.1.len() as umem + self.addr_count as umem * remaining
            {
                &mut out_target
            } else {
                &mut wait_target
            };

            let addr = Address::from(addr);
            let addr_aligned = addr.as_mem_aligned(align_as);
            let index = (addr - addr_aligned) as umem / step_size;
            let (pt_addr, _) = self.pt_addr.get_pt_by_index(index as usize);
            let pt_addr = spec.vtop_step(pt_addr, addr, self.step);

            let mut new_chunk = TranslationChunk::new(pt_addr, self.prev_flags);

            // Go through each address and check it individually
            for _ in 0..self.addr_count {
                let data = self.pop_data(addr_stack).unwrap();

                debug_assert!(
                    data.addr >= self.min_addr,
                    "__ {} {:x}+{:x} | {:#?}",
                    cnt,
                    data.addr,
                    data.length(),
                    &self
                );
                debug_assert!(
                    data.addr + data.length() <= self.max_addr,
                    "{} {:x}+{:x} | {:#?}",
                    cnt,
                    data.addr,
                    data.length(),
                    &self
                );

                let (left, right) = data.split_at_address(addr);

                if let Some(data) = right {
                    new_chunk.push_data(data, addrs_out);
                }

                // There was some leftover data
                if let Some(data) = left {
                    cur_max_addr =
                        std::cmp::max((data.addr + data.length()).to_umem(), cur_max_addr);
                    self.push_data(data, tmp_addr_stack);
                }
            }

            if new_chunk.addr_count > 0 {
                new_chunk.step = self.step;
                chunks_out.push(new_chunk);
            }

            std::mem::swap(&mut addr_stack, &mut tmp_addr_stack);
        }

        debug_assert!(self.addr_count == 0);
    }
}
