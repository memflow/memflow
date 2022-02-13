//! Generic address and buffer association structure.

use crate::iter::SplitAtIndex;
use crate::types::{umem, Address, PhysicalAddress};
use cglue::callback::OpaqueCallback;

use cglue::slice::*;

/// Generic type representing an address and associated data.
///
/// This base type is always used for initialization, but the commonly used type aliases are:
/// `ReadData`, `WriteData`, `PhysicalReadData`, and `PhysicalWriteData`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "abi_stable", derive(::abi_stable::StableAbi))]
pub struct MemData<A, T>(pub A, pub T);

impl<A, T> From<MemData<A, T>> for (A, T) {
    fn from(MemData(a, b): MemData<A, T>) -> Self {
        (a, b)
    }
}

impl<A, T> From<(A, T)> for MemData<A, T> {
    fn from((a, b): (A, T)) -> Self {
        MemData(a, b)
    }
}

impl<T: SplitAtIndex> SplitAtIndex for MemData<Address, T> {
    fn split_at(self, idx: umem) -> (Option<Self>, Option<Self>) {
        let (left, right) = self.1.split_at(idx);

        if let Some(left) = left {
            let left_len = left.length();
            (
                Some(MemData(self.0, left)),
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
                Some(MemData(self.0, left)),
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

/// MemData type for regular memory reads.
pub type ReadData<'a> = MemData<Address, CSliceMut<'a, u8>>;

pub trait ReadIterator<'a>: Iterator<Item = ReadData<'a>> + 'a {}
impl<'a, T: Iterator<Item = ReadData<'a>> + 'a> ReadIterator<'a> for T {}

/// MemData type for regular memory writes.
pub type WriteData<'a> = MemData<Address, CSliceRef<'a, u8>>;

pub type MemoryRange = MemData<Address, umem>;

pub trait WriteIterator<'a>: Iterator<Item = WriteData<'a>> + 'a {}
impl<'a, T: Iterator<Item = WriteData<'a>> + 'a> WriteIterator<'a> for T {}

/// MemData type for physical memory reads.
pub type PhysicalReadData<'a> = MemData<PhysicalAddress, CSliceMut<'a, u8>>;

pub trait PhysicalReadIterator<'a>: Iterator<Item = PhysicalReadData<'a>> + 'a {}
impl<'a, T: Iterator<Item = PhysicalReadData<'a>> + 'a> PhysicalReadIterator<'a> for T {}

/// MemData type for physical memory writes.
pub type PhysicalWriteData<'a> = MemData<PhysicalAddress, CSliceRef<'a, u8>>;

pub trait PhysicalWriteIterator<'a>: Iterator<Item = PhysicalWriteData<'a>> + 'a {}
impl<'a, T: Iterator<Item = PhysicalWriteData<'a>> + 'a> PhysicalWriteIterator<'a> for T {}

pub type ReadFailCallback<'a, 'b> = OpaqueCallback<'a, ReadData<'b>>;

pub type WriteFailCallback<'a, 'b> = OpaqueCallback<'a, WriteData<'b>>;

pub type MemoryRangeCallback<'a> = OpaqueCallback<'a, MemoryRange>;
