use std::prelude::v1::*;

pub mod direct_translate;
use crate::iter::SplitAtIndex;
pub use direct_translate::DirectTranslate;

#[cfg(test)]
mod tests;

use crate::error::{Error, Result};

use crate::mem::PhysicalMemory;
use crate::types::{Address, PhysicalAddress};

use crate::architecture::ScopedVirtualTranslate;

pub trait VirtualTranslate
where
    Self: Send,
{
    /// This function will do a virtual to physical memory translation for the
    /// `ScopedVirtualTranslate` over multiple elements.
    ///
    /// In most cases, you will want to use the `VirtualDMA`, but this trait is provided if needed
    /// to implement some more advanced filtering.
    ///
    /// # Examples
    ///
    /// ```
    /// # use memflow::error::Result;
    /// # use memflow::types::{PhysicalAddress, Address};
    /// # use memflow::mem::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::architecture::x86::x64;
    /// use memflow::iter::FnExtend;
    /// use memflow::mem::{VirtualTranslate, DirectTranslate};
    ///
    /// # const VIRT_MEM_SIZE: usize = size::mb(8);
    /// # const CHUNK_SIZE: usize = 2;
    /// #
    /// # let mut mem = DummyMemory::new(size::mb(16));
    /// # let (dtb, virtual_base) = mem.alloc_dtb(VIRT_MEM_SIZE, &[]);
    /// # let translator = x64::new_translator(dtb);
    /// let arch = x64::ARCH;
    ///
    /// let mut buffer = vec![0; VIRT_MEM_SIZE * CHUNK_SIZE / arch.page_size()];
    /// let buffer_length = buffer.len();
    ///
    /// // In this example, 8 megabytes starting from `virtual_base` are mapped in.
    /// // We translate 2 bytes chunks over the page boundaries. These bytes will be
    /// // split off into 2 separate translated chunks.
    /// let addresses = buffer
    ///     .chunks_mut(CHUNK_SIZE)
    ///     .enumerate()
    ///     .map(|(i, buf)| (virtual_base + ((i + 1) * size::kb(4) - 1), buf));
    ///
    /// let mut translated_data = vec![];
    /// let mut failed_translations = FnExtend::void();
    ///
    /// let mut direct_translate = DirectTranslate::new();
    ///
    /// direct_translate.virt_to_phys_iter(
    ///     &mut mem,
    ///     &translator,
    ///     addresses,
    ///     &mut translated_data,
    ///     &mut failed_translations,
    /// );
    ///
    ///
    /// // We tried to translate one byte out of the mapped memory, it had to fail
    /// assert_eq!(translated_data.len(), buffer_length - 1);
    ///
    /// # Ok::<(), memflow::error::Error>(())
    /// ```
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
        D: ScopedVirtualTranslate,
        VI: Iterator<Item = (Address, B)>,
        VO: Extend<(PhysicalAddress, B)>,
        FO: Extend<(Error, Address, B)>;

    // helpers
    fn virt_to_phys<T: PhysicalMemory + ?Sized, D: ScopedVirtualTranslate>(
        &mut self,
        phys_mem: &mut T,
        translator: &D,
        vaddr: Address,
    ) -> Result<PhysicalAddress> {
        let mut vec = vec![]; //Vec::new_in(&arena);
        let mut vec_fail = vec![]; //BumpVec::new_in(&arena);
        self.virt_to_phys_iter(
            phys_mem,
            translator,
            Some((vaddr, 1)).into_iter(),
            &mut vec,
            &mut vec_fail,
        );
        if let Some(ret) = vec.pop() {
            Ok(ret.0)
        } else {
            Err(vec_fail.pop().unwrap().0)
        }
    }
}

// forward impls
impl<'a, T, P> VirtualTranslate for P
where
    T: VirtualTranslate + ?Sized,
    P: std::ops::DerefMut<Target = T> + Send,
{
    #[inline]
    fn virt_to_phys_iter<U, B, D, VI, VO, FO>(
        &mut self,
        phys_mem: &mut U,
        translator: &D,
        addrs: VI,
        out: &mut VO,
        out_fail: &mut FO,
    ) where
        U: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        D: ScopedVirtualTranslate,
        VI: Iterator<Item = (Address, B)>,
        VO: Extend<(PhysicalAddress, B)>,
        FO: Extend<(Error, Address, B)>,
    {
        (**self).virt_to_phys_iter(phys_mem, translator, addrs, out, out_fail)
    }
}
