/*!
PE view.
*/

use std::prelude::v1::*;

use super::MemoryPeViewContext;

use std::cmp;

use pelite::{Error, Result};

use pelite::pe64::image::*;
use pelite::pe64::{Align, Pe, PeObject};

use memflow_core::mem::VirtualMemory;
use memflow_core::types::Address;

/// View into a mapped PE image.
pub struct MemoryPeView<'a, T: VirtualMemory> {
    context: &'a MemoryPeViewContext<'a, T>,
}

impl<'a, T: VirtualMemory> Copy for MemoryPeView<'a, T> {}
impl<'a, T: VirtualMemory> Clone for MemoryPeView<'a, T> {
    fn clone(&self) -> Self {
        Self {
            context: self.context,
        }
    }
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
    pub fn new(context: &'a MemoryPeViewContext<'a, T>) -> Result<Self> {
        Ok(Self { context })
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

unsafe impl<'a, T: VirtualMemory> Pe<'a> for MemoryPeView<'a, T> {
    /// Slices the raw image buffer at the specified offset
    fn image_slice(&self, offset: usize, size: usize) -> Option<&'a [u8]> {
        unsafe {
            self.context.update_cache(Address::from(offset), size);
            (*self.context.image_cache.get()).get(offset..offset.wrapping_add(size))
        }
    }

    /// Slices the image at the specified rva.
    ///
    /// If successful the returned slice's length will be at least the given size but often be quite larger.
    /// This allows to access the image without knowing beforehand how large the structure being accessed will be.
    ///
    /// The length is the largest consecutive number of bytes available until the end.
    /// In case the of PE files on disk, this is limited to the section's size of raw data.
    ///
    /// # Errors
    ///
    /// * [`Null`](../enum.Error.html#variant.Null):
    ///   The rva is zero.
    fn slice(&self, rva: Rva, min_size_of: usize, _align: usize) -> Result<&'a [u8]> {
        unsafe {
            // slice_section(image, rva, min_size_of, align),
            let start = rva as usize;
            if rva == 0 {
                Err(Error::Null)
            } else if start + min_size_of > (*self.context.image_cache.get()).len() {
                Err(Error::Bounds)
            } else {
                // TODO: remember cache
                self.context.update_cache(Address::from(start), min_size_of);
                Ok(&(*self.context.image_cache.get())[start..])
            }
        }
    }

    /// Gets the bytes defined by a section header in this image.
    ///
    /// # Errors
    ///
    /// * [`Null`](../enum.Error.html#variant.Null):
    ///   The virtual address or pointer to raw data is zero.
    ///
    /// * [`Bounds`](../enum.Error.html#variant.Bounds):
    ///   The data referenced by the section header is out of bounds.
    fn get_section_bytes(self, section_header: &IMAGE_SECTION_HEADER) -> Result<&'a [u8]> {
        let address = section_header.VirtualAddress;
        if address == 0 {
            return Err(Error::Null);
        }
        let start = address as usize;
        let end = address.wrapping_add(section_header.VirtualSize) as usize;

        unsafe {
            self.context.update_cache(Address::from(start), start - end);
            (*self.context.image_cache.get())
                .get(start..end)
                .ok_or(Error::Bounds)
        }
    }

    /// Reads the image at the specified va.
    ///
    /// If successful the returned slice's length will be at least the given size but often be quite larger.
    /// This allows to access the image without knowing beforehand how large the structure being accessed will be.
    ///
    /// The length is the largest consecutive number of bytes available until the end.
    /// In case the of PE files on disk, this is limited to the section's size of raw data.
    ///
    /// # Errors
    ///
    /// * [`Null`](../enum.Error.html#variant.Null):
    ///   The va is zero.
    fn read(&self, va: Va, min_size_of: usize, _align: usize) -> Result<&'a [u8]> {
        unsafe {
            // read_section(image, va, min_size_of, align),
            let (image_base, image_size) = {
                let optional_header = self.optional_header();
                (optional_header.ImageBase, optional_header.SizeOfImage)
            };
            if va == 0 {
                Err(Error::Null)
            } else if va < image_base || va - image_base > image_size as Va {
                Err(Error::Bounds)
            } else {
                let start = (va - image_base) as usize;
                //if !usize::wrapping_add(image.as_ptr() as usize, start).aligned_to(align_of) {
                //    Err(Error::Misaligned)
                //} else
                if start + min_size_of > (*self.context.image_cache.get()).len() {
                    Err(Error::Bounds)
                } else {
                    // TODO: remember cache
                    self.context.update_cache(Address::from(start), min_size_of);
                    Ok(&(*self.context.image_cache.get())[start..])
                }
            }
        }
    }
}

unsafe impl<'a, T: VirtualMemory> PeObject<'a> for MemoryPeView<'a, T> {
    fn image(&self) -> &'a [u8] {
        unsafe { &*self.context.image_cache.get() }
    }
    fn align(&self) -> Align {
        Align::Section
    }
    #[cfg(feature = "serde")]
    fn serde_name(&self) -> &'static str {
        "MemoryPeView"
    }
}
