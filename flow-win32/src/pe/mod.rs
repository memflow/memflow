pub mod pe32;
pub mod pe64;

use std::cell::{RefCell, UnsafeCell};

use pelite::{Error, PeView, Result};

use flow_core::mem::VirtualMemory;
use flow_core::types::{Address, Length};

enum PeFormat {
    PE32,
    PE64,
}

/// Wrapping Context to enable the MemoryPeView to be Copy-able
pub struct MemoryPeViewContext<T: VirtualMemory> {
    virt_mem: RefCell<T>,
    image_base: Address,
    format: PeFormat,
    image_cache: UnsafeCell<Vec<u8>>,
}

impl<T: VirtualMemory> MemoryPeViewContext<T> {
    pub fn new(mut virt_mem: T, image_base: Address) -> Result<Self> {
        // read the first page of the image
        let mut image_header = [0u8; 0x1000];
        virt_mem
            .virt_read_into(image_base, &mut image_header)
            .map_err(|_| Error::Unmapped)?;

        let view = PeView::from_bytes(image_header.as_ref())?;

        let size_of_image = match view.optional_header() {
            pelite::Wrap::T32(opt32) => opt32.SizeOfImage,
            pelite::Wrap::T64(opt64) => opt64.SizeOfImage,
        };
        let mut image_cache = vec![0u8; size_of_image as usize];
        image_cache[..image_header.len()].copy_from_slice(&image_header);

        Ok(Self {
            virt_mem: RefCell::new(virt_mem),
            image_base,
            format: match view {
                pelite::Wrap::T32(_) => PeFormat::PE32,
                pelite::Wrap::T64(_) => PeFormat::PE64,
            },
            image_cache: UnsafeCell::new(image_cache),
        })
    }

    pub unsafe fn update_cache(&self, addr: Address, mut len: Length) {
        if len.is_zero() {
            // this is a string read, we just estimate the length of the string here
            len = Length::from_kb(1);
        }

        self.virt_mem
            .borrow_mut()
            .virt_read_raw_into(
                self.image_base + Length::from(addr.as_u64()),
                &mut (*self.image_cache.get())[addr.as_usize()..(addr + len).as_usize()],
            )
            .ok();
    }
}

//----------------------------------------------------------------

/// Wraps 32-bit and 64-bit variants.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize), serde(untagged))]
pub enum Wrap<T32, T64> {
    T32(T32),
    T64(T64),
}

impl<Iter32: Iterator, Iter64: Iterator> Iterator for Wrap<Iter32, Iter64> {
    type Item = Wrap<Iter32::Item, Iter64::Item>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Wrap::T32(iter32) => iter32.next().map(Wrap::T32),
            Wrap::T64(iter64) => iter64.next().map(Wrap::T64),
        }
    }
}

impl<T32, T64> Wrap<Result<T32>, Result<T64>> {
    /// Transposes a wrap of results in a result of a wrap.
    #[inline]
    pub fn transpose(self) -> Result<Wrap<T32, T64>> {
        match self {
            Wrap::T32(Ok(ok)) => Ok(Wrap::T32(ok)),
            Wrap::T32(Err(err)) => Err(err),
            Wrap::T64(Ok(ok)) => Ok(Wrap::T64(ok)),
            Wrap::T64(Err(err)) => Err(err),
        }
    }
}
impl<T32, T64> Wrap<Option<T32>, Option<T64>> {
    /// Transposes a wrap of options in an option of a wrap.
    #[inline]
    pub fn transpose(self) -> Option<Wrap<T32, T64>> {
        match self {
            Wrap::T32(Some(some)) => Some(Wrap::T32(some)),
            Wrap::T32(None) => None,
            Wrap::T64(Some(some)) => Some(Wrap::T64(some)),
            Wrap::T64(None) => None,
        }
    }
}
impl<T> Wrap<T, T> {
    /// Unwraps the wrapped value of equal types.
    #[inline]
    pub fn into(self) -> T {
        match self {
            Wrap::T32(val) => val,
            Wrap::T64(val) => val,
        }
    }
}

/// Format agnostic Lazy PE view.
pub type MemoryPeView<'a, T> =
    Wrap<pe32::MemoryPeView<'a, T>, pe64::MemoryPeView<'a, T>>;

impl<'a, T: VirtualMemory> MemoryPeView<'a, T> {
    pub fn new(context: &'a MemoryPeViewContext<T>) -> Result<MemoryPeView<'a, T>> {
        match context.format {
            PeFormat::PE64 => Ok(Wrap::T64(pe64::MemoryPeView::new(context)?)),
            PeFormat::PE32 => Ok(Wrap::T32(pe32::MemoryPeView::new(context)?)),
        }
    }
}
