use crate::mem::mem_data::{MemData, MemoryRange, MemoryRangeCallback};
use crate::types::{imem, umem, Address};
use cglue::prelude::v1::*;

pub struct GapRemover<'a> {
    set: rangemap::RangeSet<Address>,
    out: Option<MemoryRangeCallback<'a>>,
    gap_size: imem,
    start: Address,
    end: Address,
}

impl<'a> GapRemover<'a> {
    pub fn new(out: MemoryRangeCallback<'a>, gap_size: imem, start: Address, end: Address) -> Self {
        Self {
            set: Default::default(),
            out: Some(out),
            gap_size,
            start,
            end,
        }
    }

    pub fn push_range(&mut self, MemData(in_virtual, size): MemoryRange) {
        self.set.insert(in_virtual..(in_virtual + size));
    }
}

impl<'a> Drop for GapRemover<'a> {
    fn drop(&mut self) {
        self.set
            .gaps(&(self.start..self.end))
            .filter(|r| {
                assert!(r.end >= r.start);
                self.gap_size >= 0 && (r.end - r.start) as umem <= self.gap_size as umem
            })
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|r| self.set.insert(r));

        self.set
            .iter()
            .map(|r| {
                let address = r.start;
                assert!(r.end >= address);
                let size = r.end - address;
                MemData(address, size as umem)
            })
            .feed_into(self.out.take().unwrap());
    }
}

impl<'a> Extend<MemoryRange> for GapRemover<'a> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = MemoryRange>,
    {
        iter.into_iter().for_each(|r| self.push_range(r));
    }
}
