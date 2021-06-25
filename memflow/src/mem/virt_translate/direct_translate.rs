use super::VirtualTranslate2;
use crate::architecture::VirtualTranslate3;
use crate::error::Error;
use crate::iter::SplitAtIndex;
use crate::mem::PhysicalMemory;
use crate::types::{size, Address, PhysicalAddress};
use std::prelude::v1::*;

/*
The `DirectTranslate` struct provides a default implementation for `VirtualTranslate2` for physical memory.
*/
#[derive(Debug, Default)]
pub struct DirectTranslate {
    tmp_buf: Box<[std::mem::MaybeUninit<u8>]>,
}

impl DirectTranslate {
    pub fn new() -> Self {
        Self::with_capacity(size::mb(64))
    }

    pub fn with_capacity(size: usize) -> Self {
        Self {
            tmp_buf: vec![std::mem::MaybeUninit::new(0); size].into_boxed_slice(),
        }
    }
}

impl Clone for DirectTranslate {
    fn clone(&self) -> Self {
        Self::with_capacity(self.tmp_buf.len())
    }
}

impl VirtualTranslate2 for DirectTranslate {
    fn virt_to_phys_iter<T, B, D, VI, VO, FO>(
        &mut self,
        phys_mem: &mut T,
        translator: &D,
        addrs: VI,
        out: &mut VO,
        out_fail: &mut FO,
    ) where
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        D: VirtualTranslate3,
        VI: Iterator<Item = (Address, B)>,
        VO: Extend<(PhysicalAddress, B)>,
        FO: Extend<(Error, Address, B)>,
    {
        translator.virt_to_phys_iter(phys_mem, addrs, out, out_fail, &mut self.tmp_buf)
    }
}
