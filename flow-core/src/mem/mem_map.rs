use crate::error::{Error, Result};
use crate::types::Address;

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
    /// This function is identical to `MemoryMap::default()`.
    pub fn new() -> Self {
        MemoryMap::default()
    }

    /// Adds a new memory mapping to this memory map by specifying base address and size of the mapping.
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

        // sort by biggest size (so the biggest mappings will be scanned first)
        self.mappings
            .sort_by(|a, b| b.size.partial_cmp(&a.size).unwrap());
    }

    /// Adds a new memory mapping to this memory map by specifying a range (base address and end addresses) of the mapping.
    /// When adding overlapping memory regions this function will panic!
    pub fn push_range(&mut self, base: Address, end: Address, real_base: Address) {
        self.push(base, end - base, real_base)
    }

    /// Maps a linear address to a hardware address.
    /// Returns `Error::Bounds` if the address is not contained within any memory region.
    pub fn map(&self, addr: Address) -> Result<Address> {
        let mapping = self
            .mappings
            .iter()
            .find(|m| m.base <= addr && addr < m.base + m.size)
            .ok_or_else(|| Error::Bounds)?;

        if mapping.base == mapping.real_base {
            Ok(addr)
        } else {
            // subtract first so we don't run into potential wrapping issues
            Ok((addr - mapping.base + mapping.real_base.as_usize()).into())
        }
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
