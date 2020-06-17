use crate::types::{Address, Length};
use std::iter::*;

pub trait SplitAtIndex {
    fn split_at(&mut self, idx: Length) -> (Self, Option<Self>)
    where
        Self: Sized;

    fn length(&self) -> Length;
}

impl SplitAtIndex for bool {
    fn split_at(&mut self, _: Length) -> (Self, Option<Self>) {
        (*self, None)
    }

    fn length(&self) -> Length {
        Length::from(1)
    }
}

impl<T> SplitAtIndex for &[T] {
    fn split_at(&mut self, idx: Length) -> (Self, Option<Self>) {
        let (left, right) = (*self).split_at(core::cmp::min(self.len(), idx.as_usize()));
        (left, if right.is_empty() { None } else { Some(right) })
    }

    fn length(&self) -> Length {
        Length::from(self.len())
    }
}

impl<T> SplitAtIndex for &mut [T] {
    fn split_at(&mut self, idx: Length) -> (Self, Option<Self>) {
        let mid = core::cmp::min(self.len(), idx.as_usize());
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

    fn length(&self) -> Length {
        Length::from(self.len())
    }
}

pub struct PageChunkIterator<T: SplitAtIndex, FS> {
    v: Option<T>,
    cur_address: Address,
    page_size: Length,
    check_split_fn: FS,
    cur_off: Length,
}

impl<T: SplitAtIndex, FS> PageChunkIterator<T, FS> {
    pub fn new(buf: T, start_address: Address, page_size: Length, check_split_fn: FS) -> Self {
        Self {
            v: Some(buf),
            cur_address: start_address,
            page_size,
            check_split_fn,
            cur_off: Length::from(0),
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
            let n = ((self.cur_address + buf.length() - Length::from(1))
                .as_page_aligned(self.page_size)
                - self.cur_address.as_page_aligned(self.page_size))
            .as_usize()
                / self.page_size.as_usize()
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
