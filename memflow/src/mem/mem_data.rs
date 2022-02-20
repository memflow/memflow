//! Generic address and buffer association structure.

use crate::types::{umem, Address, PageType, PhysicalAddress};
use cglue::callback::{Callbackable, OpaqueCallback};
use cglue::iter::CIterator;
use cglue::tuple::*;

use cglue::slice::*;

/// MemData type for regular memory reads.
pub type ReadDataRaw<'a> = CTup3<Address, Address, CSliceMut<'a, u8>>;
pub type ReadData<'a> = CTup2<Address, CSliceMut<'a, u8>>;

pub trait ReadRawIterator<'a>: Iterator<Item = ReadDataRaw<'a>> + 'a {}
impl<'a, T: Iterator<Item = ReadDataRaw<'a>> + 'a> ReadRawIterator<'a> for T {}

pub trait ReadIterator<'a>: Iterator<Item = ReadData<'a>> + 'a {}
impl<'a, T: Iterator<Item = ReadData<'a>> + 'a> ReadIterator<'a> for T {}

/// MemData type for regular memory writes.
pub type WriteDataRaw<'a> = CTup3<Address, Address, CSliceRef<'a, u8>>;
pub type WriteData<'a> = CTup2<Address, CSliceRef<'a, u8>>;

pub type VtopRange = CTup2<Address, umem>;

pub type MemoryRange = CTup3<Address, umem, PageType>;

pub trait WriteRawIterator<'a>: Iterator<Item = WriteDataRaw<'a>> + 'a {}
impl<'a, T: Iterator<Item = WriteDataRaw<'a>> + 'a> WriteRawIterator<'a> for T {}

pub trait WriteIterator<'a>: Iterator<Item = WriteData<'a>> + 'a {}
impl<'a, T: Iterator<Item = WriteData<'a>> + 'a> WriteIterator<'a> for T {}

/// MemData type for physical memory reads.
pub type PhysicalReadData<'a> = CTup3<PhysicalAddress, Address, CSliceMut<'a, u8>>;

pub trait PhysicalReadIterator<'a>: Iterator<Item = PhysicalReadData<'a>> + 'a {}
impl<'a, T: Iterator<Item = PhysicalReadData<'a>> + 'a> PhysicalReadIterator<'a> for T {}

/// MemData type for physical memory writes.
pub type PhysicalWriteData<'a> = CTup3<PhysicalAddress, Address, CSliceRef<'a, u8>>;

pub trait PhysicalWriteIterator<'a>: Iterator<Item = PhysicalWriteData<'a>> + 'a {}
impl<'a, T: Iterator<Item = PhysicalWriteData<'a>> + 'a> PhysicalWriteIterator<'a> for T {}

pub type ReadFailCallback<'a, 'b> = OpaqueCallback<'a, ReadDataRaw<'b>>;
pub type ReadCallback<'a, 'b> = OpaqueCallback<'a, ReadData<'b>>;

pub type WriteFailCallback<'a, 'b> = OpaqueCallback<'a, WriteDataRaw<'b>>;
pub type WriteCallback<'a, 'b> = OpaqueCallback<'a, WriteData<'b>>;

pub type MemoryRangeCallback<'a> = OpaqueCallback<'a, MemoryRange>;

/// Data needed to perform memory operations.
///
/// `inp` is an iterator containing
#[repr(C)]
#[cfg_attr(feature = "abi_stable", derive(::abi_stable::StableAbi))]
pub struct MemOps<'a: 'c, 'b, 'c, T: 'b, P: 'a> {
    pub inp: CIterator<'b, T>,
    pub out: Option<&'c mut OpaqueCallback<'a, P>>,
    pub out_fail: Option<&'c mut OpaqueCallback<'a, P>>,
}

impl<'a: 'c, 'b, 'c, T: 'b, P: 'a> MemOps<'a, 'b, 'c, T, P> {
    #[inline(always)]
    pub fn with_raw_mut<O, F: FnOnce(MemOps<T, P>) -> O>(
        iter: impl Into<CIterator<'b, T>>,
        out: Option<&'c mut OpaqueCallback<'a, P>>,
        out_fail: Option<&'c mut OpaqueCallback<'a, P>>,
        func: F,
    ) -> O {
        func(Self {
            inp: iter.into(),
            out,
            out_fail,
        })
    }

    #[inline(always)]
    pub fn with_raw<O, F: FnOnce(MemOps<T, P>) -> O>(
        mut iter: impl Iterator<Item = T>,
        out: Option<&mut OpaqueCallback<'a, P>>,
        out_fail: Option<&mut OpaqueCallback<'a, P>>,
        func: F,
    ) -> O {
        func(MemOps {
            inp: (&mut iter).into(),
            out,
            out_fail,
        })
    }
}

impl<'a: 'c, 'b, 'c, A: 'b + Into<Address> + Copy, T: 'b, P: 'a>
    MemOps<'a, 'b, 'c, CTup3<A, Address, T>, P>
{
    #[inline(always)]
    pub fn with<O, F: FnOnce(MemOps<CTup3<A, Address, T>, P>) -> O>(
        iter: impl Iterator<Item = (A, T)> + 'a,
        out: Option<&'c mut OpaqueCallback<'a, P>>,
        out_fail: Option<&'c mut OpaqueCallback<'a, P>>,
        func: F,
    ) -> O {
        let iter = iter.map(|(a, b)| CTup3(a, a.into(), b));
        Self::with_raw(iter, out, out_fail, func)
    }
}

impl<'a: 'c, 'b, 'c, T: 'b, I: Into<CIterator<'b, T>>, P: 'a> From<I> for MemOps<'a, 'b, 'c, T, P> {
    fn from(inp: I) -> Self {
        Self {
            inp: inp.into(),
            out: None,
            out_fail: None,
        }
    }
}

pub fn opt_call<T>(cb: Option<&mut impl Callbackable<T>>, data: T) -> bool {
    cb.map(|cb| cb.call(data)).unwrap_or(true)
}

pub type ReadRawMemOps<'buf, 'a, 'b, 'c> = MemOps<'a, 'b, 'c, ReadDataRaw<'buf>, ReadData<'buf>>;
pub type WriteRawMemOps<'buf, 'a, 'b, 'c> = MemOps<'a, 'b, 'c, WriteDataRaw<'buf>, WriteData<'buf>>;
pub type ReadMemOps<'buf, 'a, 'b, 'c> = MemOps<'a, 'b, 'c, ReadData<'buf>, ReadData<'buf>>;
pub type WriteMemOps<'buf, 'a, 'b, 'c> = MemOps<'a, 'b, 'c, WriteData<'buf>, WriteData<'buf>>;
pub type PhysicalReadMemOps<'buf, 'a, 'b, 'c> =
    MemOps<'a, 'b, 'c, PhysicalReadData<'buf>, ReadData<'buf>>;
pub type PhysicalWriteMemOps<'buf, 'a, 'b, 'c> =
    MemOps<'a, 'b, 'c, PhysicalWriteData<'buf>, WriteData<'buf>>;
