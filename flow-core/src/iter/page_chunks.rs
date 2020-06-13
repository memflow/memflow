use crate::types::{Address, Length};
use std::iter::*;

pub trait SplitAtIndex {
    fn split_at(self, idx: Length) -> (Self, Option<Self>)
    where
        Self: Sized;

    fn length(&self) -> Length;
}

impl SplitAtIndex for bool {
    fn split_at(self, _: Length) -> (Self, Option<Self>) {
        (self, None)
    }

    fn length(&self) -> Length {
        Length::from(1)
    }
}

impl<T> SplitAtIndex for &[T] {
    fn split_at(self, idx: Length) -> (Self, Option<Self>) {
        let (left, right) = self.split_at(core::cmp::min(self.len(), idx.as_usize()));
        (left, if right.is_empty() { None } else { Some(right) })
    }

    fn length(&self) -> Length {
        Length::from(self.len())
    }
}

impl<T> SplitAtIndex for &mut [T] {
    fn split_at(self, idx: Length) -> (Self, Option<Self>) {
        let (left, right) = self.split_at_mut(core::cmp::min(self.len(), idx.as_usize()));
        (left, if right.is_empty() { None } else { Some(right) })
    }

    fn length(&self) -> Length {
        Length::from(self.len())
    }
}

pub struct PageChunkIterator<T: SplitAtIndex> {
    v: Option<T>,
    cur_address: Address,
    page_size: Length,
}

impl<T: SplitAtIndex> PageChunkIterator<T> {
    pub fn new(buf: T, start_address: Address, page_size: Length) -> Self {
        Self {
            v: Some(buf),
            cur_address: start_address,
            page_size,
        }
    }
}

impl<T: SplitAtIndex> Iterator for PageChunkIterator<T> {
    type Item = (Address, T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let v = core::mem::replace(&mut self.v, None);

        if let Some(buf) = v {
            let next_len = (self.cur_address + self.page_size).as_page_aligned(self.page_size)
                - self.cur_address;
            let (head, tail) = buf.split_at(next_len);
            self.v = tail;
            self.cur_address += next_len;
            Some((self.cur_address - next_len, head))
        } else {
            None
        }
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
