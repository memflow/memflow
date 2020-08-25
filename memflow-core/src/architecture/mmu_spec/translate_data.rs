use crate::iter::SplitAtIndex;
use crate::types::Address;
use bumpalo::{collections::Vec as BumpVec, Bump};
use std::cmp::Ordering;

pub type TranslateVec<'a, T> = BumpVec<'a, TranslationChunk<'a, T>>;

pub struct TranslateData<T> {
    pub addr: Address,
    pub buf: T,
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
pub struct TranslationChunk<'a, T> {
    pub pt_addr: Address,
    pub vec: BumpVec<'a, TranslateData<T>>,
    min_addr: Address,
    max_addr: Address,
}

impl<'a, T> TranslationChunk<'a, T> {
    pub fn min_addr(&self) -> Address {
        self.min_addr
    }

    pub fn max_addr(&self) -> Address {
        self.max_addr
    }
}

impl<'a, T: SplitAtIndex> TranslationChunk<'a, T> {
    pub fn new(pt_addr: Address, vec: BumpVec<'a, TranslateData<T>>) -> Self {
        let (min, max) = vec.iter().fold((!0u64, 0u64), |(cmin, cmax), elem| {
            (
                std::cmp::min(cmin, elem.addr.as_u64()),
                std::cmp::max(cmax, elem.addr.as_u64() + elem.length() as u64),
            )
        });

        Self::with_minmax(pt_addr, vec, min.into(), max.into()) //std::cmp::max(min, max).into())
    }

    pub fn with_minmax(
        pt_addr: Address,
        vec: BumpVec<'a, TranslateData<T>>,
        min_addr: Address,
        max_addr: Address,
    ) -> Self {
        Self {
            pt_addr,
            vec,
            min_addr,
            max_addr,
        }
    }

    pub fn recalc_minmax(&mut self) {
        let (min, max) = self.vec.iter().fold((!0u64, 0u64), |(cmin, cmax), elem| {
            (
                std::cmp::min(cmin, elem.addr.as_u64()),
                std::cmp::max(cmax, elem.addr.as_u64() + elem.length() as u64),
            )
        });

        self.min_addr = min.into();
        self.max_addr = max.into();
    }

    pub fn consume_mut(&mut self, arena: &'a Bump) -> Self {
        let pt_addr = std::mem::replace(&mut self.pt_addr, Address::null());
        let vec = std::mem::replace(&mut self.vec, BumpVec::new_in(arena));
        let min_addr = std::mem::replace(&mut self.min_addr, Address::invalid());
        let max_addr = std::mem::replace(&mut self.max_addr, Address::null());

        Self {
            pt_addr,
            vec,
            min_addr,
            max_addr,
        }
    }

    pub fn merge_with(&mut self, mut other: Self) {
        //if other has a vec with larger capacity, then first swap them
        if self.vec.capacity() < other.vec.capacity() {
            std::mem::swap(self, &mut other);
        }

        self.vec.extend(other.vec.into_iter());

        self.min_addr = std::cmp::min(self.min_addr, other.min_addr);
        self.max_addr = std::cmp::max(self.max_addr, other.max_addr);
    }

    pub fn split_at_inclusive(mut self, idx: usize, arena: &'a Bump) -> (Self, Option<Self>) {
        let len = self.max_addr - self.min_addr;

        if len <= idx {
            (self, None)
        } else {
            let mut vec_right = BumpVec::new_in(arena);
            let min_addr = self.min_addr;
            let end_addr = min_addr + std::cmp::min(len - 1, idx);
            let pt_addr = self.pt_addr;

            let mut left_min = Address::invalid();
            let mut left_max = Address::null();

            let mut right_min = Address::invalid();
            let mut right_max = Address::null();

            for i in (0..self.vec.len()).rev() {
                let data = self.vec.get_mut(i).unwrap();
                if data.addr <= end_addr {
                    let idx = end_addr - data.addr;
                    //Need to remove empty ones
                    let (left, right) = data.split_inclusive_at(idx);
                    if left.length() > 0 {
                        left_min = std::cmp::min(left_min, left.addr);
                        left_max = std::cmp::max(left_max, left.addr + left.length());
                        *data = left;
                    } else {
                        self.vec.swap_remove(i);
                    }
                    if let Some(right) = right {
                        right_min = std::cmp::min(right_min, right.addr);
                        right_max = std::cmp::max(right_max, right.addr + right.length());
                        vec_right.push(right);
                    }
                } else {
                    right_min = std::cmp::min(right_min, data.addr);
                    right_max = std::cmp::max(right_max, data.addr + data.length());
                    vec_right.push(self.vec.swap_remove(i));
                }
            }

            self.min_addr = left_min;
            self.max_addr = left_max;

            if vec_right.is_empty() {
                (self, None)
            } else {
                (
                    self,
                    Some(TranslationChunk::with_minmax(
                        pt_addr, vec_right, right_min, right_max,
                    )),
                )
            }
        }
    }
}

impl<'a, T: SplitAtIndex> SplitAtIndex for (&'a Bump, TranslationChunk<'a, T>) {
    fn split_at(&mut self, idx: usize) -> (Self, Option<Self>) {
        if idx == 0 {
            let chunk = self.1.consume_mut(self.0);
            ((self.0, self.1.consume_mut(self.0)), Some((self.0, chunk)))
        } else {
            self.split_inclusive_at(idx - 1)
        }
    }

    fn split_inclusive_at(&mut self, idx: usize) -> (Self, Option<Self>) {
        let chunk = self.1.consume_mut(self.0);
        let (left, right) = chunk.split_at_inclusive(idx, self.0);
        ((self.0, left), right.map(|x| (self.0, x)))
    }

    fn unsplit(&mut self, left: Self, right: Option<Self>) {
        self.1.merge_with(left.1);
        if let Some(chunk) = right {
            self.1.merge_with(chunk.1);
        }
    }

    fn length(&self) -> usize {
        self.1.max_addr() - self.1.min_addr()
    }
}
