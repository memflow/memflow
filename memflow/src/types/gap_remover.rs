use crate::mem::mem_data::{MemoryRange, MemoryRangeCallback};
use crate::types::{imem, umem, Address, PageType};
use cglue::prelude::v1::*;
use std::prelude::v1::*;

pub struct GapRemover<'a> {
    map: rangemap::RangeMap<Address, PageType>,
    out: Option<MemoryRangeCallback<'a>>,
    gap_size: imem,
    start: Address,
    end: Address,
}

impl<'a> GapRemover<'a> {
    pub fn new(out: MemoryRangeCallback<'a>, gap_size: imem, start: Address, end: Address) -> Self {
        Self {
            map: Default::default(),
            out: Some(out),
            gap_size,
            start,
            end,
        }
    }

    pub fn push_range(&mut self, CTup3(in_virtual, size, page_type): MemoryRange) {
        self.map.insert(in_virtual..(in_virtual + size), page_type);
    }
}

impl<'a> Drop for GapRemover<'a> {
    fn drop(&mut self) {
        self.map
            .gaps(&(self.start..self.end))
            .filter_map(|r| {
                assert!(r.end >= r.start);
                if self.gap_size >= 0 && (r.end - r.start) as umem <= self.gap_size as umem {
                    if r.start.to_umem() > 0 {
                        let next = r.end;
                        let prev = r.start - 1 as umem;
                        match (self.map.get(&prev), self.map.get(&next)) {
                            (Some(p1), Some(p2)) if p1 == p2 => Some((r, *p2)),
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|(r, p)| self.map.insert(r, p));

        self.map
            .iter()
            .map(|(r, p)| {
                let address = r.start;
                assert!(r.end >= address);
                let size = r.end - address;
                CTup3(address, size as umem, *p)
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
