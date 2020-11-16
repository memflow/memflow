use super::{ArchMMUSpec, MMUTranslationBase};
use crate::iter::SplitAtIndex;
use crate::types::Address;
use log::trace;
use std::cmp::Ordering;

use super::MVec;

use std::time::{Duration, Instant};

pub type TranslateVec<'a> = MVec<'a, TranslationChunk<Address>>;
pub type TranslateDataVec<'a, T> = MVec<'a, TranslateData<T>>;

#[derive(Debug)]
pub struct TranslateData<T> {
    pub addr: Address,
    pub buf: T,
}

/*impl<T> TranslateData<T> {
    pub fn new((addr, buf): (Address, T)) -> Self {
        Self { addr, buf }
    }
}*/

impl<T: SplitAtIndex> TranslateData<T> {
    pub fn split_at_address(&mut self, addr: Address) -> (Self, Option<Self>) {
        if addr < self.addr {
            self.split_at(0)
        } else {
            self.split_at(addr - self.addr)
        }
    }

    /*pub fn split_inclusive_at_address(&mut self, addr: Address) -> (Self, Option<Self>) {
        if addr < self.addr {
            self.split_at(0)
        } else {
            self.split_inclusive_at(addr - self.addr)
        }
    }*/

    pub fn split_at_address_rev(&mut self, addr: Address) -> (Option<Self>, Self) {
        if addr > self.addr + self.length() {
            self.split_at_rev(0)
        } else {
            self.split_at_rev((self.addr + self.length()) - addr)
        }
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
    fn split_inclusive_at(&mut self, idx: usize) -> (Self, Option<Self>)
    where
        Self: Sized,
    {
        let addr = self.addr;

        let (bleft, bright) = self.buf.split_inclusive_at(idx);
        let bl_len = bleft.length();

        (
            TranslateData { addr, buf: bleft },
            bright.map(|buf| TranslateData {
                buf,
                addr: addr + bl_len,
            }),
        )
    }

    fn split_at(&mut self, idx: usize) -> (Self, Option<Self>)
    where
        Self: Sized,
    {
        let addr = self.addr;
        let (bleft, bright) = self.buf.split_at(idx);
        let bl_len = bleft.length();

        (
            TranslateData { addr, buf: bleft },
            bright.map(|buf| TranslateData {
                buf,
                addr: addr + bl_len,
            }),
        )
    }

    fn length(&self) -> usize {
        self.buf.length()
    }

    fn size_hint(&self) -> usize {
        self.buf.size_hint()
    }
}

/// Abstracts away a list of TranslateData in a splittable manner
#[derive(Debug)]
pub struct TranslationChunk<T> {
    pub pt_addr: T,
    pub addr_count: usize,
    min_addr: Address,
    max_addr: Address,
    pub step: usize,
}

impl<T> TranslationChunk<T> {
    pub fn new(pt_addr: T) -> Self {
        let (min, max) = (!0u64, 0u64);
        Self::with_minmax(pt_addr, min.into(), max.into())
    }

    pub fn with_minmax(pt_addr: T, min_addr: Address, max_addr: Address) -> Self {
        Self {
            pt_addr,
            addr_count: 0,
            step: 0,
            min_addr,
            max_addr,
        }
    }
}

impl<T: MMUTranslationBase> TranslationChunk<T> {
    /// Pushes data to stack updating min/max bounds
    pub fn push_data<U: SplitAtIndex /*, O: Extend<TranslateData<U>>*/>(
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

    pub fn verify_bounds<'a, U: SplitAtIndex + 'a, I: Iterator<Item = &'a TranslateData<U>>>(
        &self,
        iter: &mut I,
    ) -> bool {
        for e in iter.take(self.addr_count) {
            if e.addr < self.min_addr || e.addr + e.length() > self.max_addr {
                trace!(
                    "Bound verification failed! {:#?} {:x}+{:x}",
                    &self,
                    e.addr,
                    e.length()
                );
                return false;
            }
        }
        true
    }

    // TODO: This needs a drop impl that consumes the iterator!!!
    /*pub fn into_addr_iter<U: SplitAtIndex>(
        self,
        addr_stack: &mut TranslateDataVec<U>,
    ) -> impl Iterator<Item = TranslateData<U>> {
        (0..self.addr_count).map(|_| addr_stack.pop().unwrap())
    }*/

    pub fn next_max_addr_count(&self, spec: &ArchMMUSpec) -> usize {
        let step_size = spec.page_size_step_unchecked(self.step + 1);

        let add = if (self.max_addr - self.min_addr) % step_size != 0 {
            1
        } else {
            0
        };

        self.addr_count * ((self.max_addr - self.min_addr) / step_size + add)
    }

    pub fn split_chunk<'a, U: SplitAtIndex /*, O: Extend<TranslationChunk<Address>>*/>(
        mut self,
        spec: &ArchMMUSpec,
        (addr_stack, tmp_addr_stack): (&mut TranslateDataVec<U>, &mut TranslateDataVec<U>),
        (chunks_out, addrs_out): (&mut TranslateVec, &mut TranslateDataVec<U>),
    ) {
        let mut addr_stack: &mut TranslateDataVec<U> = unsafe { std::mem::transmute(addr_stack) };
        let mut tmp_addr_stack: &mut TranslateDataVec<U> =
            unsafe { std::mem::transmute(tmp_addr_stack) };

        let align_as = spec.page_size_step_unchecked(self.step);
        let step_size = spec.page_size_step_unchecked(self.step + 1);

        debug_assert!(self.verify_bounds(&mut addr_stack.iter().rev()), "SPL 1");

        //TODO: mask out the addresses to limit them within address space
        //this is in particular for the first step where addresses are split between positive and
        //negative sides
        let upper: u64 = (self.max_addr - 1).as_page_aligned(step_size).as_u64();
        let lower: u64 = self.min_addr.as_page_aligned(step_size).as_u64();

        debug_assert!(self.step == 0 || (upper - lower) <= align_as as _);

        // Walk in reverse so that lowest addresses always end up
        // first in the stack. This preserves translation order
        for (cnt, addr) in (lower..=upper).rev().step_by(step_size).enumerate() {
            debug_assert!(self.step == 0 || cnt < 0x200);

            let addr = Address::from(addr);
            let index = (addr - addr.as_page_aligned(align_as)) / step_size;
            let (pt_addr, _) = self.pt_addr.get_pt_by_index(index);
            let pt_addr = spec.vtop_step(pt_addr, addr, self.step);

            let mut new_chunk = TranslationChunk::new(pt_addr);

            for _ in 0..self.addr_count {
                // TODO: We need to remove the None check here
                let mut data = self.pop_data(addr_stack).unwrap();

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

                /*if data.addr > addr + (step_size - 1) {
                    self.push_data(data, tmp_addr_stack);
                    continue;
                }

                // Must be inclusive, otherwise we will get an overflow
                let (left, right) = data.split_inclusive_at_address(addr + (step_size - 1));

                debug_assert!(left.length() <= step_size);

                new_chunk.push_data(left, addrs_out);

                if let Some(data) = right {
                    self.push_data(data, tmp_addr_stack);
                }*/

                //trace!("{:x}-{:x} | {:x}", data.addr + data.length(), data.length(), addr + (step_size - 1));

                if data.addr + data.length() <= addr {
                    self.push_data(data, tmp_addr_stack);
                    continue;
                }

                let (left, right) = data.split_at_address(addr);

                let data = right.unwrap();
                {
                    debug_assert!(data.length() <= step_size);
                    new_chunk.push_data(data, addrs_out);
                    //debug_assert!(new_chunk.verify_bounds(&mut addrs_out.iter().rev()), "PSP2");
                }

                if left.length() > 0 {
                    self.push_data(left, tmp_addr_stack);
                }
                //debug_assert!(self.verify_bounds(&mut tmp_addr_stack.iter().rev()), "SP2");
            }

            if new_chunk.addr_count > 0 {
                new_chunk.step = self.step;
                //debug_assert!(new_chunk.verify_bounds(&mut addrs_out.iter().rev()), "PSP");
                chunks_out.push(new_chunk);
            }

            let t_addr_stack = addr_stack;
            addr_stack = tmp_addr_stack;
            tmp_addr_stack = t_addr_stack;
            //std::mem::swap(&mut addr_stack, &mut tmp_addr_stack);
        }

        debug_assert!(self.addr_count == 0);
    }
}
