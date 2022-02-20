use crate::iter::SplitAtIndex;
use crate::types::{umem, Address, PhysicalAddress};

use crate::mem::mem_data::opt_call;
use cglue::callback::*;
use cglue::tuple::*;
use std::cmp::Ordering;
use std::convert::TryInto;
use std::default::Default;
use std::fmt;
use std::prelude::v1::*;

// those only required when compiling under std environment
#[cfg(feature = "std")]
use crate::error::{Error, ErrorKind, ErrorOrigin, Result};

/// The `MemoryMap`struct provides a mechanism to map addresses from the linear address space
/// that memflow uses internally to hardware specific memory regions.
///
/// All memory addresses will be bounds checked.
///
/// # Examples
///
/// ```
/// use memflow::prelude::{MemoryMap, CTup2, umem};
///
/// let mut map = MemoryMap::new();
/// map.push_remap(0x1000.into(), 0x1000, 0.into());      // push region from 0x1000 - 0x1FFF
/// map.push_remap(0x3000.into(), 0x1000, 0x2000.into()); // push region from 0x3000 - 0x3FFFF
///
/// println!("{:?}", map);
///
/// // handle unmapped memory regions
/// let failed = &mut |CTup2(a, b)| {
///     println!("Unmapped: {} {}", a, b);
///     true
/// };
///
/// let hw_addr = map.map(0x10ff.into(), 8 as umem, Some(failed));
/// ```
#[derive(Clone)]
pub struct MemoryMap<M> {
    mappings: Vec<MemoryMapping<M>>,
}

impl<M> std::convert::AsRef<MemoryMap<M>> for MemoryMap<M> {
    fn as_ref(&self) -> &Self {
        self
    }
}

#[derive(Clone)]
pub struct MemoryMapping<M> {
    base: Address,
    output: std::cell::RefCell<M>, // TODO: why refcell?
}

impl<M> MemoryMapping<M> {
    pub fn base(&self) -> Address {
        self.base
    }

    pub fn output(&self) -> std::cell::Ref<M> {
        self.output.borrow()
    }
}

impl<M: SplitAtIndex> Default for MemoryMap<M> {
    fn default() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }
}

type InnerIter<M> = std::vec::IntoIter<MemoryMapping<M>>;
type InnerFunc<T, M> = fn(MemoryMapping<M>) -> T;

impl<M: SplitAtIndex> IntoIterator for MemoryMap<M> {
    type Item = (Address, M);
    type IntoIter = std::iter::Map<InnerIter<M>, InnerFunc<Self::Item, M>>;

    fn into_iter(self) -> Self::IntoIter {
        self.mappings
            .into_iter()
            .map(|map| (map.base, map.output.into_inner()))
    }
}

impl<M: SplitAtIndex> MemoryMap<M> {
    /// Constructs a new memory map.
    ///
    /// This function is identical to `MemoryMap::default()`.
    pub fn new() -> Self {
        MemoryMap::default()
    }

    // Returns `true` if there are no memory mappings.
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    /// Iterator over memory mappings
    pub fn iter(&self) -> impl Iterator<Item = &MemoryMapping<M>> {
        self.mappings.iter()
    }

    /// Maps a linear address range to a hardware address range.
    ///
    /// Output element lengths will both match, so there is no need to do additonal clipping
    /// (for buf-to-buf copies).
    ///
    /// Invalid regions get pushed to the `out_fail` parameter. This function requries `self`
    pub fn map<'a, T: 'a + SplitAtIndex, V: Callbackable<CTup2<Address, T>>>(
        &'a self,
        addr: Address,
        buf: T,
        out_fail: Option<&'a mut V>,
    ) -> impl Iterator<Item = CTup3<M, Address, T>> + 'a {
        MemoryMapIterator::new(
            &self.mappings,
            Some(CTup3(addr, addr, buf)).into_iter(),
            out_fail,
        )
    }

    /// Maps a address range iterator to an address range.
    ///
    /// Output element lengths will both match, so there is no need to do additonal clipping
    /// (for buf-to-buf copies).
    ///
    /// Invalid regions get pushed to the `out_fail` parameter
    pub fn map_base_iter<
        'a,
        T: 'a + SplitAtIndex,
        I: 'a + Iterator<Item = CTup3<Address, Address, T>>,
        V: Callbackable<CTup2<Address, T>>,
    >(
        &'a self,
        iter: I,
        out_fail: Option<&'a mut V>,
    ) -> MemoryMapIterator<'a, I, M, T, V> {
        MemoryMapIterator::new(&self.mappings, iter, out_fail)
    }

    /// Maps a address range iterator to a hardware address range.
    ///
    /// Output element lengths will both match, so there is no need to do additonal clipping
    /// (for buf-to-buf copies).
    ///
    /// Invalid regions get pushed to the `out_fail` parameter
    pub fn map_iter<
        'a,
        T: 'a + SplitAtIndex,
        I: 'a + Iterator<Item = CTup3<PhysicalAddress, Address, T>>,
        V: Callbackable<CTup2<Address, T>>,
    >(
        &'a self,
        iter: I,
        out_fail: Option<&'a mut V>,
    ) -> MemoryMapIterator<'a, impl Iterator<Item = CTup3<Address, Address, T>> + 'a, M, T, V> {
        MemoryMapIterator::new(
            &self.mappings,
            iter.map(|CTup3(addr, meta_addr, buf)| CTup3(addr.address(), meta_addr, buf)),
            out_fail,
        )
    }

    /// Adds a new memory mapping to this memory map.
    ///
    /// When adding overlapping memory regions this function will panic!
    pub fn push(&mut self, base: Address, output: M) -> &mut Self {
        let mapping = MemoryMapping {
            base,
            output: output.into(),
        };

        let mut shift_idx = self.mappings.len();

        // bounds check. In reverse order, because most likely
        // all mappings will be inserted in increasing order
        for (i, m) in self.mappings.iter().enumerate().rev() {
            let start = base;
            let end = base + mapping.output.borrow().length();
            if m.base <= start && start < m.base + m.output.borrow().length()
                || m.base <= end && end < m.base + m.output.borrow().length()
            {
                // overlapping memory regions should not be possible
                panic!(
                    "MemoryMap::push overlapping regions: {:x}-{:x} ({:x}) | {:x}-{:x} ({:x})",
                    base,
                    end,
                    mapping.output.borrow().length(),
                    m.base,
                    m.base + m.output.borrow().length(),
                    m.output.borrow().length()
                );
            } else if m.base + m.output.borrow().length() <= start {
                shift_idx = i + 1;
                break;
            }
        }

        self.mappings.insert(shift_idx, mapping);

        self
    }
}

#[cfg(feature = "serde")]
#[allow(unused)]
#[derive(::serde::Deserialize)]
struct MemoryMapFile {
    #[serde(rename = "range")]
    ranges: Vec<MemoryMapFileRange>,
}

#[cfg(feature = "serde")]
#[allow(unused)]
#[derive(::serde::Deserialize)]
struct MemoryMapFileRange {
    base: u64,
    length: u64,
    real_base: Option<u64>,
}

// FFI Safe MemoryMapping type for `MemoryMap<(Address, umem)>`.
// TODO: this could be removed if the RefCell requirement above would be removed.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "abi_stable", derive(::abi_stable::StableAbi))]
pub struct PhysicalMemoryMapping {
    pub base: Address,
    pub size: umem,
    pub real_base: Address,
}

impl MemoryMap<(Address, umem)> {
    /// Constructs a new memory map by parsing the mapping table from a [TOML](https://toml.io/) file.
    ///
    /// The file must contain a mapping table in the following format:
    ///
    /// ```toml
    /// [[range]]
    /// base=0x1000
    /// length=0x1000
    ///
    /// [[range]]
    /// base=0x2000
    /// length=0x1000
    /// real_base=0x3000
    /// ```
    ///
    /// The `real_base` parameter is optional. If it is not set there will be no re-mapping.
    #[cfg(feature = "memmapfiles")]
    pub fn open<P: AsRef<::std::path::Path>>(path: P) -> Result<Self> {
        let contents = ::std::fs::read_to_string(path).map_err(|err| {
            Error(ErrorOrigin::MemoryMap, ErrorKind::UnableToReadFile)
                .log_error(format!("unable to open the memory mapping file: {}", err))
        })?;
        let mappings: MemoryMapFile = ::toml::from_str(&contents).map_err(|err| {
            Error(ErrorOrigin::MemoryMap, ErrorKind::UnableToReadFile).log_error(format!(
                "unable to parse the memory mapping toml file: {}",
                err
            ))
        })?;

        let mut result = MemoryMap::new();
        for range in mappings.ranges.iter() {
            let real_base = range.real_base.unwrap_or(range.base);
            result.push_range(
                range.base.into(),
                (range.base + range.length).into(),
                real_base.into(),
            );
        }

        Ok(result)
    }

    /// Returns the highest memory address that can be read.
    pub fn max_address(&self) -> Address {
        self.mappings
            .iter()
            .map(|m| m.base() + m.output.borrow().1)
            .max()
            .unwrap_or_else(|| umem::MAX.into())
            - 1_usize
    }

    // Returns the real size the current memory mappings cover
    pub fn real_size(&self) -> umem {
        self.mappings.iter().fold(0, |s, m| s + m.output.borrow().1)
    }

    /// Adds a new memory mapping to this memory map by specifying base address and size of the mapping.
    ///
    /// When adding overlapping memory regions this function will panic!
    pub fn push_remap(&mut self, base: Address, size: umem, real_base: Address) -> &mut Self {
        self.push(base, (real_base, size))
    }

    /// Adds a new memory mapping to this memory map by specifying a range (base address and end addresses) of the mapping.
    ///
    /// When adding overlapping memory regions this function will panic!
    ///
    /// If end < base, the function will do nothing
    pub fn push_range(&mut self, base: Address, end: Address, real_base: Address) -> &mut Self {
        if end > base {
            self.push_remap(base, (end - base) as umem, real_base)
        } else {
            self
        }
    }

    /// Transform address mapping into mutable buffer mapping
    ///
    /// It will take the output address-size pair, and create mutable slice references to them.
    ///
    /// # Safety
    ///
    /// The address mappings must be valid for the given lifetime `'a`, and should not
    /// be aliased by any other memory references for fully defined behaviour.
    ///
    /// However, aliasing *should* be fine for volatile memory cases such as analyzing running VM,
    /// since there are no safety guarantees anyways.
    pub unsafe fn into_bufmap_mut<'a>(self) -> MemoryMap<&'a mut [u8]> {
        let mut ret_map = MemoryMap::new();

        self.into_iter()
            .map(|(base, (real_base, size))| {
                (
                    base,
                    std::slice::from_raw_parts_mut(
                        real_base.to_umem() as _,
                        size.try_into().unwrap(),
                    ),
                )
            })
            .for_each(|(base, buf)| {
                ret_map.push(base, buf);
            });

        ret_map
    }

    /// Transform address mapping buffer buffer mapping
    ///
    /// It will take the output address-size pair, and create slice references to them.
    ///
    /// # Safety
    ///
    /// The address mappings must be valid for the given lifetime `'a`.
    pub unsafe fn into_bufmap<'a>(self) -> MemoryMap<&'a [u8]> {
        let mut ret_map = MemoryMap::new();

        self.into_iter()
            .map(|(base, (real_base, size))| {
                (
                    base,
                    std::slice::from_raw_parts(real_base.to_umem() as _, size.try_into().unwrap()),
                )
            })
            .for_each(|(base, buf)| {
                ret_map.push(base, buf);
            });

        ret_map
    }

    // TODO: into/from trait impls
    pub fn into_vec(self) -> Vec<PhysicalMemoryMapping> {
        self.iter()
            .map(|m| PhysicalMemoryMapping {
                base: m.base(),
                size: m.output().1,
                real_base: m.output().0,
            })
            .collect::<Vec<_>>()
    }

    pub fn from_vec(mem_map: Vec<PhysicalMemoryMapping>) -> Self {
        let mut map = Self::new();
        for mapping in mem_map.iter() {
            map.push_range(mapping.base, mapping.base + mapping.size, mapping.real_base);
        }
        map
    }
}

const MIN_BSEARCH_THRESH: usize = 32;

pub type MapFailCallback<'a, T> = OpaqueCallback<'a, CTup3<Address, Address, T>>;

pub struct MemoryMapIterator<'a, I, M, T, C> {
    map: &'a [MemoryMapping<M>],
    in_iter: I,
    fail_out: Option<&'a mut C>,
    cur_elem: Option<CTup3<Address, Address, T>>,
    cur_map_pos: usize,
}

#[allow(clippy::needless_option_as_deref)]
impl<
        'a,
        I: Iterator<Item = CTup3<Address, Address, T>>,
        M: SplitAtIndex,
        T: SplitAtIndex,
        C: Callbackable<CTup2<Address, T>>,
    > MemoryMapIterator<'a, I, M, T, C>
{
    fn new(map: &'a [MemoryMapping<M>], in_iter: I, fail_out: Option<&'a mut C>) -> Self {
        Self {
            map,
            in_iter,
            fail_out,
            cur_elem: None,
            cur_map_pos: 0,
        }
    }

    pub fn fail_out(&mut self) -> Option<&mut C> {
        self.fail_out.as_deref_mut()
    }

    fn get_next(&mut self) -> Option<CTup3<M, Address, T>> {
        if let Some(CTup3(mut addr, mut meta_addr, buf)) = self.cur_elem.take() {
            if self.map.len() >= MIN_BSEARCH_THRESH && self.cur_map_pos == 0 {
                self.cur_map_pos = match self.map.binary_search_by(|map_elem| {
                    if map_elem.base > addr {
                        Ordering::Greater
                    } else if map_elem.base + map_elem.output.borrow().length() <= addr {
                        Ordering::Less
                    } else {
                        Ordering::Equal
                    }
                }) {
                    Ok(idx) | Err(idx) => idx,
                };
            }

            for (i, map_elem) in self.map.iter().enumerate().skip(self.cur_map_pos) {
                let output = &mut *map_elem.output.borrow_mut();
                if map_elem.base + output.length() > addr {
                    let offset: umem = map_elem.base.to_umem().saturating_sub(addr.to_umem());

                    let (left_reject, right) = buf.split_at(offset);

                    if let Some(left_reject) = left_reject {
                        opt_call(self.fail_out.as_deref_mut(), CTup2(meta_addr, left_reject));
                    }

                    addr += offset;
                    meta_addr += offset;

                    if let Some(leftover) = right {
                        let off = map_elem.base.to_umem() + output.length() - addr.to_umem();
                        let (ret, keep) = leftover.split_at(off);
                        let ret_length = ret.as_ref().map(|r| r.length()).unwrap_or_default();

                        let cur_map_pos = &mut self.cur_map_pos;
                        let in_iter = &mut self.in_iter;

                        self.cur_elem = keep
                            .map(|x| {
                                //If memory is in right order, this will skip the current mapping,
                                //but not reset the search
                                *cur_map_pos = i + 1;
                                CTup3(addr + ret_length, meta_addr + ret_length, x)
                            })
                            .or_else(|| {
                                *cur_map_pos = 0;
                                in_iter.next()
                            });

                        let off = addr.to_umem() - map_elem.base.to_umem();
                        let split_left = unsafe { output.split_at_mut(off).1 };
                        return split_left
                            .unwrap()
                            .split_at(ret_length)
                            .0
                            .zip(ret)
                            .map(|(a, b)| (a, meta_addr, b))
                            .map(<_>::into);
                    }

                    return None;
                }
            }

            let _ = opt_call(self.fail_out.as_deref_mut(), CTup2(meta_addr, buf));
        }
        None
    }
}

impl<
        'a,
        I: Iterator<Item = CTup3<Address, Address, T>>,
        M: SplitAtIndex,
        T: SplitAtIndex,
        C: Callbackable<CTup2<Address, T>>,
    > Iterator for MemoryMapIterator<'a, I, M, T, C>
{
    type Item = CTup3<M, Address, T>;

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

impl<M> fmt::Debug for MemoryMap<M>
where
    MemoryMapping<M>: fmt::Debug,
{
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

impl fmt::Debug for MemoryMapping<(Address, umem)> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MemoryMapping: base={:x} size={:x} real_base={:x}",
            self.base,
            self.output.borrow().1,
            self.output.borrow().0
        )
    }
}

impl fmt::Debug for MemoryMapping<&mut [u8]> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MemoryMapping: base={:x} size={:x} real_base={:?}",
            self.base,
            self.output.borrow().len(),
            self.output.borrow().as_ptr()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping() {
        let mut map = MemoryMap::new();
        map.push_remap(0x1000.into(), 0x1000, 0.into());
        map.push_remap(0x3000.into(), 0x1000, 0x2000.into());

        let mut void_panic = |x| panic!("Should not have mapped {:?}", x);
        assert_eq!(
            (map.map::<umem, _>(0x10ff.into(), 1, Some(&mut void_panic))
                .next()
                .unwrap()
                .0)
                .0,
            Address::from(0x00ff)
        );
        assert_eq!(
            (map.map::<umem, _>(0x30ff.into(), 1, Some(&mut void_panic))
                .next()
                .unwrap()
                .0)
                .0,
            Address::from(0x20ff)
        );
    }

    #[test]
    fn test_mapping_edges() {
        let mut map = MemoryMap::new();
        map.push_remap(0x1000.into(), 0x1000, 0.into());
        map.push_remap(0x3000.into(), 0x1000, 0x2000.into());

        let mut void_panic = |x| panic!("Should not have mapped {:?}", x);
        let mut void = |_| true;

        assert_eq!(
            (map.map::<umem, _>(0x3000.into(), 1, Some(&mut void_panic))
                .next()
                .unwrap()
                .0)
                .0,
            Address::from(0x2000)
        );
        assert_eq!(
            (map.map::<umem, _>(0x3fff.into(), 1, Some(&mut void_panic))
                .next()
                .unwrap()
                .0)
                .0,
            Address::from(0x2fff)
        );
        assert_eq!(
            map.map::<umem, _>(0x2fff.into(), 1, Some(&mut void)).next(),
            None
        );
        assert_eq!(
            map.map::<umem, _>(0x4000.into(), 1, Some(&mut void)).next(),
            None
        );
    }

    #[test]
    fn test_mapping_out_of_bounds() {
        let mut map = MemoryMap::new();
        map.push_remap(0x1000.into(), 0x1000, 0.into());
        map.push_remap(0x3000.into(), 0x1000, 0x2000.into());

        let mut void = vec![];
        let mut cbvoid: OpaqueCallback<_> = (&mut void).into();
        assert_eq!(
            map.map::<umem, _>(0x00ff.into(), 1, Some(&mut cbvoid))
                .next(),
            None
        );
        assert_eq!(
            map.map::<umem, _>(0x20ff.into(), 1, Some(&mut cbvoid))
                .next(),
            None
        );
        assert_eq!(
            map.map::<umem, _>(0x4000.into(), 1, Some(&mut cbvoid))
                .next(),
            None
        );
        assert_eq!(
            map.map::<umem, _>(0x40ff.into(), 1, Some(&mut cbvoid))
                .next(),
            None
        );

        assert_eq!(void.len(), 4);
    }

    #[test]
    fn test_mapping_range() {
        let mut map = MemoryMap::new();
        map.push_range(0x1000.into(), 0x2000.into(), 0.into());
        map.push_range(0x3000.into(), 0x4000.into(), 0x2000.into());

        let mut void_panic = |x| panic!("Should not have mapped {:?}", x);
        assert_eq!(
            (map.map::<umem, _>(0x10ff.into(), 1, Some(&mut void_panic))
                .next()
                .unwrap()
                .0)
                .0,
            Address::from(0x00ff)
        );
        assert_eq!(
            (map.map::<umem, _>(0x30ff.into(), 1, Some(&mut void_panic))
                .next()
                .unwrap()
                .0)
                .0,
            Address::from(0x20ff)
        );
    }

    #[test]
    fn test_mapping_range_edge() {
        let mut map = MemoryMap::new();
        map.push_range(0x1000.into(), 0x2000.into(), 0.into());
        map.push_range(0x3000.into(), 0x4000.into(), 0x2000.into());

        let mut void_panic = |x| panic!("Should not have mapped {:?}", x);
        let mut void = |_| true;

        assert_eq!(
            (map.map::<umem, _>(0x3000.into(), 1, Some(&mut void_panic))
                .next()
                .unwrap()
                .0)
                .0,
            Address::from(0x2000)
        );
        assert_eq!(
            (map.map::<umem, _>(0x3fff.into(), 1, Some(&mut void_panic))
                .next()
                .unwrap()
                .0)
                .0,
            Address::from(0x2fff)
        );
        assert_eq!(
            map.map::<umem, _>(0x2fff.into(), 1, Some(&mut void)).next(),
            None
        );
        assert_eq!(
            map.map::<umem, _>(0x4000.into(), 1, Some(&mut void)).next(),
            None
        );
    }

    #[test]
    fn test_mapping_range_close() {
        let mut map = MemoryMap::new();
        map.push_range(0x1000.into(), 0x2000.into(), 0.into());
        map.push_range(0x2000.into(), 0x3000.into(), 0x2000.into());

        let mut void_panic = |x| panic!("Should not have mapped {:?}", x);
        let mut void = |_| true;

        assert_eq!(
            (map.map::<umem, _>(0x2000.into(), 1, Some(&mut void_panic))
                .next()
                .unwrap()
                .0)
                .0,
            Address::from(0x2000)
        );
        assert_eq!(
            (map.map::<umem, _>(0x2fff.into(), 1, Some(&mut void_panic))
                .next()
                .unwrap()
                .0)
                .0,
            Address::from(0x2fff)
        );
        assert_eq!(
            map.map::<umem, _>(0x3fff.into(), 1, Some(&mut void)).next(),
            None
        );
        assert_eq!(
            map.map::<umem, _>(0x3000.into(), 1, Some(&mut void)).next(),
            None
        );
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

    #[test]
    fn test_max_address() {
        let mut map = MemoryMap::new();
        map.push_remap(0x1000.into(), 0x1000, 0.into());
        map.push_remap(0x3000.into(), 0x1000, 0x2000.into());
        assert_eq!(map.max_address(), Address::from(0x3FFF));
    }

    #[test]
    fn test_real_size() {
        let mut map = MemoryMap::new();
        map.push_remap(0x1000.into(), 0x1000, 0.into());
        map.push_remap(0x3000.into(), 0x1000, 0x2000.into());
        map.push_remap(0x6000.into(), 0x2000, 0x3000.into());
        assert_eq!(map.real_size(), 0x4000);
    }

    #[cfg(feature = "memmapfiles")]
    #[test]
    fn test_load_toml() {
        let mappings: MemoryMapFile = ::toml::from_str(
            "
[[range]]
base=0x1000
length=0x1000

[[range]]
base=0x2000
length=0x1000
real_base=0x3000",
        )
        .unwrap();

        assert_eq!(mappings.ranges.len(), 2);
        assert_eq!(mappings.ranges[0].real_base, None);
        assert_eq!(mappings.ranges[1].real_base, Some(0x3000));
    }
}
