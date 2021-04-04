use std::prelude::v1::*;

pub mod direct_translate;
use crate::iter::SplitAtIndex;
pub use direct_translate::DirectTranslate;

#[cfg(test)]
mod tests;

use crate::error::{Error, Result};

use crate::iter::FnExtend;
use crate::mem::PhysicalMemory;
use crate::types::{Address, PhysicalAddress};

use crate::architecture::ScopedVirtualTranslate;

pub trait VirtualTranslate
where
    Self: Send,
{
    /// Translate a list of virtual addresses
    ///
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
    /// # use memflow::dummy::{DummyMemory, DummyOS};
    /// use memflow::mem::{VirtualTranslate, DirectTranslate};
    /// use memflow::types::size;
    /// use memflow::architecture::x86::x64;
    /// use memflow::iter::FnExtend;
    ///
    /// # const VIRT_MEM_SIZE: usize = size::mb(8);
    /// # const CHUNK_SIZE: usize = 2;
    /// #
    /// # let mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOS::new(mem);
    /// # let (dtb, virtual_base) = os.alloc_dtb(VIRT_MEM_SIZE, &[]);
    /// # let mut mem = os.into_inner();
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

    /// Translate a single virtual address
    ///
    /// This function will do a virtual to physical memory translation for the
    /// `ScopedVirtualTranslate` for single address returning either PhysicalAddress, or an error.
    ///
    /// # Examples
    /// ```
    /// # use memflow::error::Result;
    /// # use memflow::types::{PhysicalAddress, Address};
    /// # use memflow::dummy::{DummyMemory, DummyOS};
    /// # use memflow::types::size;
    /// # use memflow::architecture::ScopedVirtualTranslate;
    /// use memflow::mem::{VirtualTranslate, DirectTranslate};
    /// use memflow::architecture::x86::x64;
    ///
    /// # const VIRT_MEM_SIZE: usize = size::mb(8);
    /// # const CHUNK_SIZE: usize = 2;
    /// #
    /// # let mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOS::new(mem);
    /// # let (dtb, virtual_base) = os.alloc_dtb(VIRT_MEM_SIZE, &[]);
    /// # let mut mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let arch = x64::ARCH;
    ///
    /// let mut direct_translate = DirectTranslate::new();
    ///
    /// // Translate a mapped address
    /// let res = direct_translate.virt_to_phys(
    ///     &mut mem,
    ///     &translator,
    ///     virtual_base,
    /// );
    ///
    /// assert!(res.is_ok());
    ///
    /// // Translate unmapped address
    /// let res = direct_translate.virt_to_phys(
    ///     &mut mem,
    ///     &translator,
    ///     virtual_base - 1,
    /// );
    ///
    /// assert!(res.is_err());
    ///
    /// ```
    fn virt_to_phys<T: PhysicalMemory + ?Sized, D: ScopedVirtualTranslate>(
        &mut self,
        phys_mem: &mut T,
        translator: &D,
        vaddr: Address,
    ) -> Result<PhysicalAddress> {
        let mut output = None;
        let mut success = FnExtend::new(|elem: (PhysicalAddress, _)| {
            if output.is_none() {
                output = Some(elem.0);
            }
        });
        let mut output_err = None;
        let mut fail = FnExtend::new(|elem: (Error, _, _)| output_err = Some(elem.0));

        self.virt_to_phys_iter(
            phys_mem,
            translator,
            Some((vaddr, 1)).into_iter(),
            &mut success,
            &mut fail,
        );
        output.map(Ok).unwrap_or_else(|| Err(output_err.unwrap()))
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
