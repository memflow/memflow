/*!
PE view.
*/

use std::cell::RefCell;
use std::rc::Rc;
use std::{cmp, slice};

use pelite::{Error, Result};

use pelite::pe64::image::*;
use pelite::pe64::PeView;
use pelite::pe64::{Align, Pe, PeObject};

use flow_core::mem::VirtualMemory;
use flow_core::types::Address;

pub struct MemoryPeViewContext<T: VirtualMemory> {
    pub virt_mem: Rc<T>,
    pub image_base: Address,
    pub image_header: [u8; 0x1000],
}

impl<T: VirtualMemory> Copy for MemoryPeViewContext<T> {}

impl<T: VirtualMemory> Clone for MemoryPeViewContext<T> {
    fn clone(&self) -> Self {
        Self {
            virt_mem: self.virt_mem.clone(),
            image_base: self.image_base,
            image_header: self.image_header,
        }
    }
}

impl<T: VirtualMemory> MemoryPeViewContext<T> {
    pub fn new(virt_mem: T, image_base: Address) -> Result<Self> {
        // read the first page of the image
        let mut image_header = [0u8; 0x1000];
        virt_mem
            .virt_read_into(image_base, &mut image_header)
            .map_err(|_| Error::Unmapped)?;

        Ok(Self {
            virt_mem: Rc::new(virt_mem),
            image_base,
            image_header,
        })
    }
}

/// View into a mapped PE image.
#[derive(Copy, Clone)]
pub struct MemoryPeView<'a, T: VirtualMemory> {
    context: &'a MemoryPeViewContext<T>,
}

impl<'a, T: VirtualMemory> MemoryPeView<'a, T> {
    /// Constructs a view from a `VirtualMemory` reader.
    ///
    /// # Errors
    ///
    /// * [`Bounds`](../enum.Error.html#variant.Bounds):
    ///   The byte slice is too small to fit the PE headers.
    ///
    /// * [`Misaligned`](../enum.Error.html#variant.Misaligned):
    ///   The minimum alignment of 4 is not satisfied.
    ///
    /// * [`BadMagic`](../enum.Error.html#variant.BadMagic):
    ///   This is not a PE file.
    ///
    /// * [`PeMagic`](../enum.Error.html#variant.PeMagic):
    ///   Trying to parse a PE32 file with the PE32+ parser and vice versa.
    ///
    /// * [`Insanity`](../enum.Error.html#variant.Insanity):
    ///   Reasonable limits on `e_lfanew`, `SizeOfHeaders` or `NumberOfSections` are exceeded.
    pub fn new(context: &'a MemoryPeViewContext<T>) -> Result<Self> {
        // probe pe header
        let _ = PeView::from_bytes(context.image_header.as_ref())?;
        Ok(Self { context })
    }

    /// Constructs a new view from module handle.
    ///
    /// # Safety
    ///
    /// The underlying memory is borrowed and an unbounded lifetime is returned.
    /// Ensure the lifetime outlives this view instance!
    ///
    /// No sanity or safety checks are done to make sure this is really PE32(+) image.
    /// When using this with a `HMODULE` from the system the caller must be sure this is a PE32(+) image.
    #[inline]
    pub unsafe fn module(base: *const u8) -> PeView<'a> {
        let dos = &*(base as *const IMAGE_DOS_HEADER);
        let nt = &*(base.offset(dos.e_lfanew as isize) as *const IMAGE_NT_HEADERS);
        PeView {
            image: slice::from_raw_parts(base, nt.OptionalHeader.SizeOfImage as usize),
        }
    }

    /// Converts the view to file alignment.
    pub fn to_file(self) -> Vec<u8> {
        let (sizeof_headers, sizeof_image) = {
            let optional_header = self.optional_header();
            (optional_header.SizeOfHeaders, optional_header.SizeOfImage)
        };

        // Figure out the size of the file image
        let mut file_size = sizeof_headers;
        for section in self.section_headers() {
            file_size = cmp::max(
                file_size,
                u32::wrapping_add(section.PointerToRawData, section.SizeOfRawData),
            );
        }
        // Clamp to the actual image size...
        file_size = cmp::min(file_size, sizeof_image);

        // Zero fill the underlying file
        let mut vec = vec![0u8; file_size as usize];

        // Start by copying the headers
        let image = self.image();
        unsafe {
            // Validated by constructor
            let dest_headers = vec.get_unchecked_mut(..sizeof_headers as usize);
            let src_headers = image.get_unchecked(..sizeof_headers as usize);
            dest_headers.copy_from_slice(src_headers);
        }

        // Copy the section image data
        for section in self.section_headers() {
            let dest = vec.get_mut(
                section.PointerToRawData as usize
                    ..u32::wrapping_add(section.PointerToRawData, section.SizeOfRawData) as usize,
            );
            let src = image.get(
                section.VirtualAddress as usize
                    ..u32::wrapping_add(section.VirtualAddress, section.VirtualSize) as usize,
            );
            // Skip invalid sections...
            if let (Some(dest), Some(src)) = (dest, src) {
                dest.copy_from_slice(src);
            }
        }

        vec
    }
}

//----------------------------------------------------------------

unsafe impl<'a, T: VirtualMemory> Pe<'a> for MemoryPeView<'a, T> {}

unsafe impl<'a, T: VirtualMemory> PeObject<'a> for MemoryPeView<'a, T> {
    fn image(&self) -> &'a [u8] {
        &self.context.image_header
    }
    fn align(&self) -> Align {
        Align::Section
    }
    #[cfg(feature = "serde")]
    fn serde_name(&self) -> &'static str {
        "MemoryPeView"
    }
}

//----------------------------------------------------------------

#[cfg(feature = "serde")]
impl<'a> serde::Serialize for MemoryPeView<'a> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        super::pe::serialize_pe(*self, serializer)
    }
}

//----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::MemoryPeView;
    use crate::Error;

    #[test]
    fn from_byte_slice() {
        assert!(match MemoryPeView::from_bytes(&[]) {
            Err(Error::Bounds) => true,
            _ => false,
        });
    }
}
