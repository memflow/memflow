use crate::address::{Address, Length};
use std::iter::*;

pub struct PageChunks<'a, T: 'a> {
    v: &'a [T],
    cur_address: Address,
    page_size: Length,
}

impl<'a, T> PageChunks<'a, T> {
    pub fn create_from(buf: &'a [T], start_address: Address, page_size: Length) -> Self {
        Self {
            v: buf,
            cur_address: start_address,
            page_size,
        }
    }
}

impl<'a, T> Iterator for PageChunks<'a, T> {
    type Item = (Address, &'a [T]);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.v.is_empty() {
            None
        } else {
            let next_len = std::cmp::min(
                Length::from(self.v.len()),
                (self.cur_address + self.page_size).as_page_aligned(self.page_size)
                    - self.cur_address,
            );
            let tmp = std::mem::replace(&mut self.v, &[]);
            let (head, tail) = tmp.split_at(next_len.as_usize());
            self.v = tail;
            self.cur_address += next_len;
            Some((self.cur_address - next_len, head))
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.v.is_empty() {
            (0, Some(0))
        } else {
            let n = ((self.cur_address + Length::from(self.v.len() - 1))
                .as_page_aligned(self.page_size)
                - self.cur_address.as_page_aligned(self.page_size))
            .as_usize()
                / self.page_size.as_usize()
                + 1;
            (n, Some(n))
        }
    }

    #[inline]
    fn count(self) -> usize {
        self.size_hint().0
    }
}

pub struct PageChunksMut<'a, T: 'a> {
    v: &'a mut [T],
    cur_address: Address,
    page_size: Length,
}

impl<'a, T> PageChunksMut<'a, T> {
    pub fn create_from(buf: &'a mut [T], start_address: Address, page_size: Length) -> Self {
        Self {
            v: buf,
            cur_address: start_address,
            page_size,
        }
    }
}

impl<'a, T> Iterator for PageChunksMut<'a, T> {
    type Item = (Address, &'a mut [T]);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.v.is_empty() {
            None
        } else {
            let next_len = std::cmp::min(
                Length::from(self.v.len()),
                (self.cur_address + self.page_size).as_page_aligned(self.page_size)
                    - self.cur_address,
            );
            let tmp = std::mem::replace(&mut self.v, &mut []);
            let (head, tail) = tmp.split_at_mut(next_len.as_usize());
            self.v = tail;
            self.cur_address += next_len;
            Some((self.cur_address - next_len, head))
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.v.is_empty() {
            (0, Some(0))
        } else {
            let n = ((self.cur_address + Length::from(self.v.len() - 1))
                .as_page_aligned(self.page_size)
                - self.cur_address.as_page_aligned(self.page_size))
            .as_usize()
                / self.page_size.as_usize()
                + 1;
            (n, Some(n))
        }
    }

    #[inline]
    fn count(self) -> usize {
        self.size_hint().0
    }
}
