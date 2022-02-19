//! Generic address and buffer association structure.

use crate::iter::SplitAtIndex;
use crate::types::{umem, Address, PageType, PhysicalAddress};
use cglue::callback::{Callbackable, OpaqueCallback};
use cglue::iter::CIterator;

use cglue::slice::*;

/// Generic type representing an address and associated data.
///
/// This base type is always used for initialization, but the commonly used type aliases are:
/// `ReadData`, `WriteData`, `PhysicalReadData`, and `PhysicalWriteData`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "abi_stable", derive(::abi_stable::StableAbi))]
pub struct MemData2<A, T>(pub A, pub T);

impl<A, T> From<MemData2<A, T>> for (A, T) {
    fn from(MemData2(a, b): MemData2<A, T>) -> Self {
        (a, b)
    }
}

impl<A, T> From<(A, T)> for MemData2<A, T> {
    fn from((a, b): (A, T)) -> Self {
        MemData2(a, b)
    }
}

impl<T: SplitAtIndex> SplitAtIndex for MemData2<Address, T> {
    fn split_at(self, idx: umem) -> (Option<Self>, Option<Self>) {
        let (left, right) = self.1.split_at(idx);

        if let Some(left) = left {
            let left_len = left.length();
            (
                Some(MemData2(self.0, left)),
                Some(self.0 + left_len).zip(right).map(<_>::into),
            )
        } else {
            (None, Some(self.0).zip(right).map(<_>::into))
        }
    }

    unsafe fn split_at_mut(&mut self, idx: umem) -> (Option<Self>, Option<Self>) {
        let (left, right) = self.1.split_at_mut(idx);

        if let Some(left) = left {
            let left_len = left.length();
            (
                Some(MemData2(self.0, left)),
                Some(self.0 + left_len).zip(right).map(<_>::into),
            )
        } else {
            (None, Some(self.0).zip(right).map(<_>::into))
        }
    }

    fn length(&self) -> umem {
        self.1.length()
    }

    fn size_hint(&self) -> usize {
        self.1.size_hint()
    }
}

/// Generic type representing an address, original address,and associated data.
///
/// This base type is always used for initialization, but the commonly used type aliases are:
/// `ReadDataIn`, `WriteDataIn`, `PhysicalReadDataIn`, and `PhysicalWriteDataIn`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "abi_stable", derive(::abi_stable::StableAbi))]
pub struct MemData3<A, M, T>(pub A, pub M, pub T);

impl<A, M, T> From<MemData3<A, M, T>> for (A, M, T) {
    fn from(MemData3(a, b, c): MemData3<A, M, T>) -> Self {
        (a, b, c)
    }
}

impl<A, M, T> From<(A, M, T)> for MemData3<A, M, T> {
    fn from((a, b, c): (A, M, T)) -> Self {
        MemData3(a, b, c)
    }
}

impl<T: SplitAtIndex> SplitAtIndex for MemData3<Address, Address, T> {
    fn split_at(self, idx: umem) -> (Option<Self>, Option<Self>) {
        let (left, right) = self.2.split_at(idx);

        let meta = self.1;

        if let Some(left) = left {
            let left_len = left.length();
            (
                Some(MemData3(self.0, meta, left)),
                Some(self.0 + left_len)
                    .zip(right)
                    .map(|(a, b)| (a, meta + left_len, b))
                    .map(<_>::into),
            )
        } else {
            (
                None,
                Some(self.0)
                    .zip(right)
                    .map(|(a, b)| (a, meta, b))
                    .map(<_>::into),
            )
        }
    }

    unsafe fn split_at_mut(&mut self, idx: umem) -> (Option<Self>, Option<Self>) {
        let (left, right) = self.2.split_at_mut(idx);

        let meta = self.1;

        if let Some(left) = left {
            let left_len = left.length();
            (
                Some(MemData3(self.0, meta, left)),
                Some(self.0 + left_len)
                    .zip(right)
                    .map(|(a, b)| (a, meta + left_len, b))
                    .map(<_>::into),
            )
        } else {
            (
                None,
                Some(self.0)
                    .zip(right)
                    .map(|(a, b)| (a, meta, b))
                    .map(<_>::into),
            )
        }
    }

    fn length(&self) -> umem {
        self.2.length()
    }

    fn size_hint(&self) -> usize {
        self.2.size_hint()
    }
}

/// MemData type for regular memory reads.
pub type ReadDataRaw<'a> = MemData3<Address, Address, CSliceMut<'a, u8>>;
pub type ReadData<'a> = MemData2<Address, CSliceMut<'a, u8>>;

pub trait ReadRawIterator<'a>: Iterator<Item = ReadDataRaw<'a>> + 'a {}
impl<'a, T: Iterator<Item = ReadDataRaw<'a>> + 'a> ReadRawIterator<'a> for T {}

pub trait ReadIterator<'a>: Iterator<Item = ReadData<'a>> + 'a {}
impl<'a, T: Iterator<Item = ReadData<'a>> + 'a> ReadIterator<'a> for T {}

/// MemData type for regular memory writes.
pub type WriteDataRaw<'a> = MemData3<Address, Address, CSliceRef<'a, u8>>;
pub type WriteData<'a> = MemData2<Address, CSliceRef<'a, u8>>;

pub type VtopRange = MemData2<Address, umem>;

pub type MemoryRange = MemData3<Address, umem, PageType>;

pub trait WriteRawIterator<'a>: Iterator<Item = WriteDataRaw<'a>> + 'a {}
impl<'a, T: Iterator<Item = WriteDataRaw<'a>> + 'a> WriteRawIterator<'a> for T {}

pub trait WriteIterator<'a>: Iterator<Item = WriteData<'a>> + 'a {}
impl<'a, T: Iterator<Item = WriteData<'a>> + 'a> WriteIterator<'a> for T {}

/// MemData type for physical memory reads.
pub type PhysicalReadData<'a> = MemData3<PhysicalAddress, Address, CSliceMut<'a, u8>>;

pub trait PhysicalReadIterator<'a>: Iterator<Item = PhysicalReadData<'a>> + 'a {}
impl<'a, T: Iterator<Item = PhysicalReadData<'a>> + 'a> PhysicalReadIterator<'a> for T {}

/// MemData type for physical memory writes.
pub type PhysicalWriteData<'a> = MemData3<PhysicalAddress, Address, CSliceRef<'a, u8>>;

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
    MemOps<'a, 'b, 'c, MemData3<A, Address, T>, P>
{
    #[inline(always)]
    pub fn with<O, F: FnOnce(MemOps<MemData3<A, Address, T>, P>) -> O>(
        iter: impl Iterator<Item = (A, T)> + 'a,
        out: Option<&'c mut OpaqueCallback<'a, P>>,
        out_fail: Option<&'c mut OpaqueCallback<'a, P>>,
        func: F,
    ) -> O {
        let iter = iter.map(|(a, b)| MemData3(a, a.into(), b));
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
