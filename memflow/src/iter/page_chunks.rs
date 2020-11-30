use crate::types::Address;
use std::iter::*;

pub trait SplitAtIndex {
    /// Split data at a given index
    ///
    /// This method will split the underlying data at a given index into up to 2 possible values.
    ///
    /// What a split means very much depends on the underlying type. sizes are split literally,
    /// into 2 sizes, one being up to idx, the other being what's left over. Slices are split into
    /// subslices. (Address, impl SplitAtIndex) pairs are split very much like slices (with Address
    /// describing the starting address of the data, and the second element being pretty much
    /// anything).
    ///
    /// But the core idea is - to allow splittable data, be split, in a generic way.
    fn split_at(self, idx: usize) -> (Option<Self>, Option<Self>)
    where
        Self: Sized;

    /// Split data using mutable reference
    ///
    /// This should behave the same as split_at, but work with mutable ref being input, instead of
    /// the actual value being consumed. This is useful when splitting slices and needing to
    /// unsplit them.
    ///
    /// # Safety:
    ///
    /// Mutating self reference and returned values after the split is undefined behaviour,
    /// because both self, and returned values can point to the same mutable region
    /// (for example: &mut [u8])
    unsafe fn split_at_mut(&mut self, idx: usize) -> (Option<Self>, Option<Self>)
    where
        Self: Sized;

    /// Inclusive version of `split_at`
    ///
    /// This is effectively split_at(idx + 1), with a safeguard for idx == usize::MAX.
    fn split_inclusive_at(self, idx: usize) -> (Option<Self>, Option<Self>)
    where
        Self: Sized,
    {
        if idx == core::usize::MAX {
            (Some(self), None)
        } else {
            self.split_at(idx + 1)
        }
    }

    /// Inclusive version of `split_at_mut`
    ///
    /// This is effectively split_at_mut(idx + 1), with a safeguard for idx == usize::MAX.
    ///
    /// # Safety:
    ///
    /// The same safety rules apply as with `split_at_mut`. Mutating the value after the function
    /// call is undefined, and should not be done until returned values are dropped.
    unsafe fn split_inclusive_at_mut(&mut self, idx: usize) -> (Option<Self>, Option<Self>)
    where
        Self: Sized,
    {
        if idx == core::usize::MAX {
            let (_, right) = self.split_at_mut(0);
            (right, None)
        } else {
            self.split_at_mut(idx + 1)
        }
    }

    /// Reverse version of `split_at`
    ///
    /// This will perform splits with index offsetting from the end of the data
    fn split_at_rev(self, idx: usize) -> (Option<Self>, Option<Self>)
    where
        Self: Sized,
    {
        if let Some(idx) = self.length().checked_sub(idx) {
            self.split_inclusive_at(idx)
        } else {
            (None, Some(self))
        }
    }

    /// Returns the length of the data
    ///
    /// This is the length in terms of how many indexes can be used to split the data.
    fn length(&self) -> usize;

    /// Returns an allocation size hint for the data
    ///
    /// This is purely a hint, but not really an exact value of how much data needs allocating.
    fn size_hint(&self) -> usize {
        self.length()
    }
}

impl SplitAtIndex for usize {
    fn split_at(self, idx: usize) -> (Option<Self>, Option<Self>) {
        if idx == 0 {
            (None, Some(self))
        } else if self <= idx {
            (Some(self), None)
        } else {
            (Some(idx), Some(self - idx))
        }
    }

    unsafe fn split_at_mut(&mut self, idx: usize) -> (Option<Self>, Option<Self>) {
        (*self).split_at(idx)
    }

    fn length(&self) -> usize {
        *self
    }

    fn size_hint(&self) -> usize {
        1
    }
}

impl<T: SplitAtIndex> SplitAtIndex for (Address, T) {
    fn split_at(self, idx: usize) -> (Option<Self>, Option<Self>) {
        let (left, right) = self.1.split_at(idx);

        if let Some(left) = left {
            let left_len = left.length();
            (Some((self.0, left)), Some(self.0 + left_len).zip(right))
        } else {
            (None, Some(self.0).zip(right))
        }
    }

    unsafe fn split_at_mut(&mut self, idx: usize) -> (Option<Self>, Option<Self>) {
        let (left, right) = self.1.split_at_mut(idx);

        if let Some(left) = left {
            let left_len = left.length();
            (Some((self.0, left)), Some(self.0 + left_len).zip(right))
        } else {
            (None, Some(self.0).zip(right))
        }
    }

    fn length(&self) -> usize {
        self.1.length()
    }

    fn size_hint(&self) -> usize {
        self.1.size_hint()
    }
}

impl<T> SplitAtIndex for &[T] {
    fn split_at(self, idx: usize) -> (Option<Self>, Option<Self>) {
        let (left, right) = (*self).split_at(core::cmp::min(self.len(), idx));
        (
            if left.is_empty() { None } else { Some(left) },
            if right.is_empty() { None } else { Some(right) },
        )
    }

    unsafe fn split_at_mut(&mut self, idx: usize) -> (Option<Self>, Option<Self>) {
        let (left, right) = (*self).split_at(core::cmp::min(self.len(), idx));
        (
            if left.is_empty() { None } else { Some(left) },
            if right.is_empty() { None } else { Some(right) },
        )
    }

    fn length(&self) -> usize {
        self.len()
    }
}

impl<T> SplitAtIndex for &mut [T] {
    fn split_at(self, idx: usize) -> (Option<Self>, Option<Self>) {
        let (left, right) = (*self).split_at_mut(core::cmp::min(self.len(), idx));
        (
            if left.is_empty() { None } else { Some(left) },
            if right.is_empty() { None } else { Some(right) },
        )
    }

    unsafe fn split_at_mut(&mut self, idx: usize) -> (Option<Self>, Option<Self>) {
        let mid = core::cmp::min(self.len(), idx);
        let ptr = self.as_mut_ptr();
        (
            if mid != 0 {
                Some(core::slice::from_raw_parts_mut(ptr, mid))
            } else {
                None
            },
            if mid != self.len() {
                Some(core::slice::from_raw_parts_mut(
                    ptr.add(mid),
                    self.len() - mid,
                ))
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
        let v = self.v.take();

        if let Some(mut buf) = v {
            loop {
                let end_len = Address::from(
                    self.cur_address
                        .as_u64()
                        .wrapping_add(self.page_size as u64),
                )
                .as_page_aligned(self.page_size)
                .as_usize()
                .wrapping_sub(self.cur_address.as_usize())
                .wrapping_sub(1)
                .wrapping_add(self.cur_off);

                let (head, tail) = unsafe { buf.split_inclusive_at_mut(end_len) };
                let head = head.unwrap();
                if tail.is_some() && !(self.check_split_fn)(self.cur_address, &head, tail.as_ref())
                {
                    self.cur_off = end_len + 1;
                } else {
                    self.v = tail;
                    let next_address =
                        Address::from(self.cur_address.as_usize().wrapping_add(end_len + 1));
                    let ret = Some((self.cur_address, head));
                    self.cur_address = next_address;
                    self.cur_off = 0;
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
}
