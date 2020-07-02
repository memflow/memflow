use crate::error::{Error, Result};
use crate::types::Address;

use crate::iter::SplitAtIndex;

use std::default::Default;
use std::fmt;
use std::prelude::v1::*;

/// The `MemoryMap`struct provides a mechanism to map addresses from the linear address space
/// that memflow uses internally to hardware specific memory regions.
///
/// All memory addresses will be bounds checked.
///
/// # Examples
///
/// ```
/// use flow_core::mem::MemoryMap;
///
/// let mut map = MemoryMap::new();
/// map.push(0x1000.into(), 0x1000, 0.into());      // push region from 0x1000 - 0x1FFF
/// map.push(0x3000.into(), 0x1000, 0x2000.into()); // push region from 0x3000 - 0x3FFFF
///
/// println!("{:?}", map);
///
/// let hw_addr = map.map(0x10ff.into());
/// ```
#[derive(Clone)]
pub struct MemoryMap {
    mappings: Vec<MemoryMapping>,
}

#[derive(Clone)]
struct MemoryMapping {
    base: Address,
    size: usize,
    real_base: Address,
}

impl Default for MemoryMap {
    fn default() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }
}

impl MemoryMap {
    /// Constructs a new memory map.
    ///
    /// This function is identical to `MemoryMap::default()`.
    pub fn new() -> Self {
        MemoryMap::default()
    }

    /// Adds a new memory mapping to this memory map by specifying base address and size of the mapping.
    ///
    /// When adding overlapping memory regions this function will panic!
    pub fn push(&mut self, base: Address, size: usize, real_base: Address) {
        // bounds check
        for m in self.mappings.iter() {
            let start = base;
            let end = base + size;
            if m.base <= start && start < m.base + m.size || m.base <= end && end < m.base + m.size
            {
                // overlapping memory regions should not be possible
                panic!();
            }
        }

        self.mappings.push(MemoryMapping {
            base,
            size,
            real_base,
        });

        self.mappings
            .sort_by(|a, b| a.base.partial_cmp(&b.base).unwrap());
    }

    /// Adds a new memory mapping to this memory map by specifying a range (base address and end addresses) of the mapping.
    ///
    /// When adding overlapping memory regions this function will panic!
    pub fn push_range(&mut self, base: Address, end: Address, real_base: Address) {
        self.push(base, end - base, real_base)
    }

    /// Maps a linear address range to a hardware address range.
    ///
    /// Invalid regions get pushed to the `out_fail` parameter
    pub fn map<'a, T: 'a + SplitAtIndex, V: Extend<(Address, T)>>(
        &'a self,
        addr: Address,
        buf: T,
        out_fail: &'a mut V,
    ) -> impl Iterator<Item = (Address, T)> + 'a {
        MemoryMapIterator::new(&self.mappings, Some((addr, buf)).into_iter(), out_fail)
    }

    /// Maps a address range iterator to a hardware address range.
    ///
    /// Invalid regions get pushed to the `out_fail` parameter
    pub fn map_iter<
        'a,
        T: 'a + SplitAtIndex,
        I: 'a + Iterator<Item = (Address, T)>,
        V: Extend<(Address, T)>,
    >(
        &'a self,
        iter: I,
        out_fail: &'a mut V,
    ) -> impl Iterator<Item = (Address, T)> + 'a {
        MemoryMapIterator::new(&self.mappings, iter, out_fail)
    }
}

pub struct MemoryMapIterator<'a, I, T, F> {
    map: &'a Vec<MemoryMapping>,
    in_iter: I,
    fail_out: &'a mut F,
    cur_elem: Option<(Address, T)>,
    cur_map_pos: usize,
}

impl<'a, I: Iterator<Item = (Address, T)>, T: SplitAtIndex, F: Extend<(Address, T)>>
    MemoryMapIterator<'a, I, T, F>
{
    fn new(map: &'a Vec<MemoryMapping>, in_iter: I, fail_out: &'a mut F) -> Self {
        Self {
            map,
            in_iter,
            fail_out,
            cur_elem: None,
            cur_map_pos: 0,
        }
    }

    fn get_next(&mut self) -> Option<(Address, T)> {
        if let Some((mut addr, mut buf)) = self.cur_elem.take() {
            for (i, map_elem) in self.map.iter().enumerate().skip(self.cur_map_pos) {
                if map_elem.base + map_elem.size > addr {
                    let offset = map_elem
                        .base
                        .as_usize()
                        .checked_sub(addr.as_usize())
                        .unwrap_or(0);

                    let (left_reject, right) = buf.split_at(offset);

                    if left_reject.length() > 0 {
                        self.fail_out.extend(Some((addr, left_reject)));
                    }

                    addr += offset;

                    if let Some(mut leftover) = right {
                        let off = map_elem.base + map_elem.size - addr;
                        let (ret, keep) = leftover.split_at(off);

                        self.cur_elem = keep
                            .map(|x| {
                                //If memory is in right order, this will skip the current mapping,
                                //but not reset the search
                                self.cur_map_pos = i + 1;
                                (addr + ret.length(), x)
                            })
                            .or_else(|| {
                                self.cur_map_pos = 0;
                                self.in_iter.next()
                            });

                        let off = addr - map_elem.base;
                        return Some((map_elem.real_base + off, ret));
                    }

                    break;
                }
            }
        }
        None
    }
}

impl<'a, I: Iterator<Item = (Address, T)>, T: SplitAtIndex, F: Extend<(Address, T)>> Iterator
    for MemoryMapIterator<'a, I, T, F>
{
    type Item = (Address, T);

    fn next(&mut self) -> Option<Self::Item> {
        //Could optimize this and move over to new method, but would need to fuse the iter
        if self.cur_elem.is_none() {
            self.cur_elem = self.in_iter.next();
        }

        let mut ret = None;

        while self.cur_elem.is_some() {
            ret = self.get_next();

            if ret.is_some() {
                break;
            }

            self.cur_elem = self.in_iter.next();
            self.cur_map_pos = 0;
        }

        ret
    }
}

impl fmt::Debug for MemoryMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, m) in self.mappings.iter().enumerate() {
            if i > 0 {
                write!(f, "\n{:?}", m)?;
            } else {
                write!(f, "{:?}", m)?;
            }
        }
        Ok(())
    }
}

impl fmt::Debug for MemoryMapping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MemoryMapping: base={:x} size={:x} real_base={:x}",
            self.base, self.size, self.real_base
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping() {
        let mut map = MemoryMap::new();
        map.push(0x1000.into(), 0x1000, 0.into());
        map.push(0x3000.into(), 0x1000, 0x2000.into());

        assert_eq!(map.map(0x10ff.into()), Ok(Address::from(0x00ff)));
        assert_eq!(map.map(0x30ff.into()), Ok(Address::from(0x20ff)));
    }

    #[test]
    fn test_mapping_edges() {
        let mut map = MemoryMap::new();
        map.push(0x1000.into(), 0x1000, 0.into());
        map.push(0x3000.into(), 0x1000, 0x2000.into());

        assert_eq!(map.map(0x3000.into()), Ok(Address::from(0x2000)));
        assert_eq!(map.map(0x3fff.into()), Ok(Address::from(0x2fff)));
        assert_eq!(map.map(0x2fff.into()).is_err(), true);
        assert_eq!(map.map(0x4000.into()).is_err(), true);
    }

    #[test]
    fn test_mapping_out_of_bounds() {
        let mut map = MemoryMap::new();
        map.push(0x1000.into(), 0x1000, 0.into());
        map.push(0x3000.into(), 0x1000, 0x2000.into());

        assert_eq!(map.map(0x00ff.into()).is_err(), true);
        assert_eq!(map.map(0x20ff.into()).is_err(), true);
        assert_eq!(map.map(0x4000.into()).is_err(), true);
        assert_eq!(map.map(0x40ff.into()).is_err(), true);
    }

    #[test]
    fn test_mapping_range() {
        let mut map = MemoryMap::new();
        map.push_range(0x1000.into(), 0x2000.into(), 0.into());
        map.push_range(0x3000.into(), 0x4000.into(), 0x2000.into());

        assert_eq!(map.map(0x10ff.into()), Ok(Address::from(0x00ff)));
        assert_eq!(map.map(0x30ff.into()), Ok(Address::from(0x20ff)));
    }

    #[test]
    fn test_mapping_range_edge() {
        let mut map = MemoryMap::new();
        map.push_range(0x1000.into(), 0x2000.into(), 0.into());
        map.push_range(0x3000.into(), 0x4000.into(), 0x2000.into());

        assert_eq!(map.map(0x3000.into()), Ok(Address::from(0x2000)));
        assert_eq!(map.map(0x3fff.into()), Ok(Address::from(0x2fff)));
        assert_eq!(map.map(0x2fff.into()).is_err(), true);
        assert_eq!(map.map(0x4000.into()).is_err(), true);
    }

    #[test]
    fn test_mapping_range_close() {
        let mut map = MemoryMap::new();
        map.push_range(0x1000.into(), 0x2000.into(), 0.into());
        map.push_range(0x2000.into(), 0x3000.into(), 0x2000.into());

        assert_eq!(map.map(0x2000.into()), Ok(Address::from(0x2000)));
        assert_eq!(map.map(0x2fff.into()), Ok(Address::from(0x2fff)));
        assert_eq!(map.map(0x3fff.into()).is_err(), true);
        assert_eq!(map.map(0x3000.into()).is_err(), true);
    }

    #[test]
    #[should_panic]
    fn test_overlapping_regions_base() {
        let mut map = MemoryMap::new();
        map.push_range(0x1000.into(), 0x2000.into(), 0.into());

        // should panic
        map.push_range(0x10ff.into(), 0x20ff.into(), 0.into());
    }

    #[test]
    #[should_panic]
    fn test_overlapping_regions_size() {
        let mut map = MemoryMap::new();
        map.push_range(0x1000.into(), 0x2000.into(), 0.into());

        // should panic
        map.push_range(0x00ff.into(), 0x10ff.into(), 0.into());
    }

    #[test]
    #[should_panic]
    fn test_overlapping_regions_contained() {
        let mut map = MemoryMap::new();
        map.push_range(0x1000.into(), 0x3000.into(), 0.into());

        // should panic
        map.push_range(0x2000.into(), 0x20ff.into(), 0.into());
    }
}
