use crate::address::{Address, Length};
use std::iter::*;

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

    #[inline]
    fn nth(&mut self, pages_add: usize) -> Option<Self::Item> {
        let split_len = if pages_add > 0 {
            self.cur_address.as_page_aligned(self.page_size) + self.page_size * pages_add
                - self.cur_address
        } else {
            0.into()
        };

        if split_len.as_usize() >= self.v.len() {
            self.v = &mut [];
            None
        } else {
            let tmp = std::mem::replace(&mut self.v, &mut []);
            self.cur_address += split_len;
            let ending = tmp.split_at_mut(split_len.as_usize()).1;
            let ending_split = std::cmp::min(
                Length::from(ending.len()),
                self.cur_address.as_page_aligned(self.page_size) + self.page_size
                    - self.cur_address,
            );
            self.cur_address += ending_split;
            let split = ending.split_at_mut(ending_split.as_usize());
            self.v = split.1;
            Some((self.cur_address - Length::from(ending_split), split.0))
        }
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let pages_add = self.size_hint().0;
        if pages_add == 0 {
            None
        } else {
            let split_len = self.cur_address.as_page_aligned(self.page_size)
                + self.page_size * (pages_add - 1)
                - self.cur_address;
            Some((
                self.cur_address + split_len,
                self.v.split_at_mut(split_len.as_usize()).1,
            ))
        }
    }
}
