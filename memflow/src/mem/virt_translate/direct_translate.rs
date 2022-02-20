use super::{VirtualTranslate2, VirtualTranslate3, VtopFailureCallback, VtopOutputCallback};
use crate::iter::SplitAtIndex;
use crate::mem::PhysicalMemory;
use crate::types::{size, Address};
use cglue::tuple::*;
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
    fn virt_to_phys_iter<T, B, D, VI>(
        &mut self,
        phys_mem: &mut T,
        translator: &D,
        addrs: VI,
        out: &mut VtopOutputCallback<B>,
        out_fail: &mut VtopFailureCallback<B>,
    ) where
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        D: VirtualTranslate3,
        VI: Iterator<Item = CTup3<Address, Address, B>>,
    {
        translator.virt_to_phys_iter(phys_mem, addrs, out, out_fail, &mut self.tmp_buf)
    }
}
