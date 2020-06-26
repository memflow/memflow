use crate::types::Address;
use std::iter::*;

pub trait SplitAtIndex {
    fn split_at(&mut self, idx: usize) -> (Self, Option<Self>)
    where
        Self: Sized;

    fn length(&self) -> usize;

    fn size_hint(&self) -> usize {
        self.length()
    }
}

impl SplitAtIndex for bool {
    fn split_at(&mut self, _: usize) -> (Self, Option<Self>) {
        (*self, None)
    }

    fn length(&self) -> usize {
        1
    }
}

impl SplitAtIndex for u64 {
    fn split_at(&mut self, idx: usize) -> (Self, Option<Self>) {
        if (*self as usize) < idx {
            (*self, None)
        } else {
            (idx as u64, Some(*self - idx as u64))
        }
    }

    fn length(&self) -> usize {
        *self as usize
    }

    fn size_hint(&self) -> usize {
        std::mem::size_of_val(self)
    }
}

impl<T> SplitAtIndex for &[T] {
    fn split_at(&mut self, idx: usize) -> (Self, Option<Self>) {
        let (left, right) = (*self).split_at(core::cmp::min(self.len(), idx));
        (left, if right.is_empty() { None } else { Some(right) })
    }

    fn length(&self) -> usize {
        self.len()
    }
}

impl<T> SplitAtIndex for &mut [T] {
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
                let next_len = (self.cur_address + self.page_size).as_page_aligned(self.page_size)
                    - self.cur_address
                    + self.cur_off;
                let (head, tail) = buf.split_at(next_len);
                if tail.is_some() && !(self.check_split_fn)(self.cur_address, &head, tail.as_ref())
                {
                    self.cur_off = next_len;
                } else {
                    let (head, tail) = buf.split_at(next_len);
                    self.v = tail;
                    self.cur_address += next_len;
                    return Some((self.cur_address - next_len, head));
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
