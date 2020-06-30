use crate::types::Address;
use std::iter::*;

pub trait SplitAtIndex {
    fn split_at(&mut self, idx: usize) -> (Self, Option<Self>)
    where
        Self: Sized,
    {
        self.split_inclusive_at(idx - 1)
    }

    fn split_inclusive_at(&mut self, idx: usize) -> (Self, Option<Self>)
    where
        Self: Sized;

    fn split_at_rev(&mut self, idx: usize) -> (Option<Self>, Self)
    where
        Self: Sized,
    {
        let (left, right) = self.split_at(self.length() - idx);
        (
            if left.length() == 0 { None } else { Some(left) },
            right.unwrap(),
        )
    }

    fn length(&self) -> usize;

    fn size_hint(&self) -> usize {
        self.length()
    }
}

impl SplitAtIndex for bool {
    fn split_inclusive_at(&mut self, _: usize) -> (Self, Option<Self>) {
        (*self, None)
    }

    fn split_at_rev(&mut self, _: usize) -> (Option<Self>, Self) {
        (None, *self)
    }

    fn length(&self) -> usize {
        1
    }
}

impl SplitAtIndex for usize {
    fn split_inclusive_at(&mut self, idx: usize) -> (Self, Option<Self>) {
        if *self == 0 || *self - 1 <= idx {
            (*self, None)
        } else {
            (idx + 1, Some(*self - idx - 1))
        }
    }

    fn split_at(&mut self, idx: usize) -> (Self, Option<Self>) {
        if (*self as usize) <= idx {
            (*self, None)
        } else {
            (idx, Some(*self - idx))
        }
    }

    fn length(&self) -> usize {
        *self
    }

    fn size_hint(&self) -> usize {
        std::mem::size_of_val(self)
    }
}

impl<T: SplitAtIndex> SplitAtIndex for (Address, T) {
    fn split_inclusive_at(&mut self, idx: usize) -> (Self, Option<Self>) {
        let (left, right) = self.1.split_inclusive_at(idx);

        if let Some(right) = right {
            let left_len = left.length();
            ((self.0, left), Some((self.0 + left_len, right)))
        } else {
            ((self.0, left), None)
        }
    }

    fn split_at(&mut self, idx: usize) -> (Self, Option<Self>) {
        let (left, right) = self.1.split_at(idx);

        if let Some(right) = right {
            let left_len = left.length();
            ((self.0, left), Some((self.0 + left_len, right)))
        } else {
            ((self.0, left), None)
        }
    }

    fn length(&self) -> usize {
        self.1.length()
    }

    fn size_hint(&self) -> usize {
        std::mem::size_of_val(&self.0) + self.1.size_hint()
    }
}

impl<T> SplitAtIndex for &[T] {
    fn split_inclusive_at(&mut self, idx: usize) -> (Self, Option<Self>) {
        let mid = core::cmp::min(self.len(), core::cmp::min(self.len(), idx) + 1);
        let (left, right) = (*self).split_at(mid);
        (left, if right.is_empty() { None } else { Some(right) })
    }

    fn split_at(&mut self, idx: usize) -> (Self, Option<Self>) {
        let (left, right) = (*self).split_at(core::cmp::min(self.len(), idx));
        (left, if right.is_empty() { None } else { Some(right) })
    }

    fn length(&self) -> usize {
        self.len()
    }
}

impl<T> SplitAtIndex for &mut [T] {
    fn split_inclusive_at(&mut self, idx: usize) -> (Self, Option<Self>) {
        let mid = core::cmp::min(self.len(), core::cmp::min(self.len(), idx) + 1);
        let ptr = self.as_mut_ptr();
        (
            unsafe { core::slice::from_raw_parts_mut(ptr, mid) },
            if mid != self.len() {
                Some(unsafe { core::slice::from_raw_parts_mut(ptr.add(mid), self.len() - mid) })
            } else {
                None
            },
        )
    }

    fn split_at(&mut self, idx: usize) -> (Self, Option<Self>) {
        let mid = core::cmp::min(self.len(), idx);
        let ptr = self.as_mut_ptr();
        (
            unsafe { core::slice::from_raw_parts_mut(ptr, mid) },
            if mid != self.len() {
                Some(unsafe { core::slice::from_raw_parts_mut(ptr.add(mid), self.len() - mid) })
            } else {
                None
            },
        )
    }

    fn length(&self) -> usize {
        self.len()
    }
}

pub struct PageChunkIterator<T: SplitAtIndex, FS> {
    v: Option<T>,
    cur_address: Address,
    page_size: usize,
    check_split_fn: FS,
    cur_off: usize,
}

impl<T: SplitAtIndex, FS> PageChunkIterator<T, FS> {
    pub fn new(buf: T, start_address: Address, page_size: usize, check_split_fn: FS) -> Self {
        Self {
            v: Some(buf),
            cur_address: start_address,
            page_size,
            check_split_fn,
            cur_off: 0,
        }
    }
}

impl<T: SplitAtIndex, FS: FnMut(Address, &T, Option<&T>) -> bool> Iterator
    for PageChunkIterator<T, FS>
{
    type Item = (Address, T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let v = core::mem::replace(&mut self.v, None);

        if let Some(mut buf) = v {
            loop {
                let end_len = Address::from(
                    self.cur_address
                        .as_u64()
                        .wrapping_add(self.page_size as u64),
                )
                .as_page_aligned(self.page_size)
                .as_usize()
                .wrapping_sub(self.cur_address.as_usize() + 1)
                .wrapping_add(self.cur_off);

                let (head, tail) = buf.split_inclusive_at(end_len);
                if tail.is_some() && !(self.check_split_fn)(self.cur_address, &head, tail.as_ref())
                {
                    self.cur_off = end_len + 1;
                } else {
                    let (head, tail) = buf.split_inclusive_at(end_len);
                    self.v = tail;
                    let next_address =
                        Address::from(self.cur_address.as_usize().wrapping_add(end_len + 1));
                    let ret = Some((self.cur_address, head));
                    self.cur_address = next_address;
                    return ret;
                }
            }
        }

        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if let Some(buf) = &self.v {
            let n = ((self.cur_address + buf.size_hint() - 1).as_page_aligned(self.page_size)
                - self.cur_address.as_page_aligned(self.page_size))
                / self.page_size
                + 1;
            (n, Some(n))
        } else {
            (0, Some(0))
        }
    }

    #[inline]
    fn count(self) -> usize {
        self.size_hint().0
    }
}
