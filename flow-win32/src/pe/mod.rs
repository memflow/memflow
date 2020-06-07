pub mod pe32;
pub mod pe64;

use std::cell::{RefCell, UnsafeCell};

use pelite::{Error, PeView, Result};

use flow_core::iter::PageChunksMut;
use flow_core::mem::VirtualMemory;
use flow_core::types::{Address, Length};

const PE_PAGE_SIZE: Length = Length::from_kb(4);

#[derive(Copy, Clone)]
pub enum PeFormat {
    Pe64,
    Pe32,
}

/// Wrapping Context to enable the MemoryPeView to be Copy-able
pub struct MemoryPeViewContext<'a, T: VirtualMemory + ?Sized> {
    virt_mem: RefCell<&'a mut T>,
    image_base: Address,
    image_format: PeFormat,
    image_pages: RefCell<Box<[bool]>>,
    image_cache: UnsafeCell<Box<[u8]>>,
}

impl<'a, T: VirtualMemory + ?Sized> MemoryPeViewContext<'a, T> {
    pub fn new(virt_mem: &'a mut T, image_base: Address) -> Result<Self> {
        // read the first page of the image
        let mut image_header = [0u8; 0x1000];
        virt_mem
            .virt_read_raw_into(image_base, &mut image_header)
            .map_err(|_| Error::Unmapped)?;

        let view = PeView::from_bytes(image_header.as_ref())?;

        // TODO: check if size_of_image < 0x1000 or too huge / or use SizeOfHeaders
        let size_of_image = match view.optional_header() {
            pelite::Wrap::T32(opt32) => opt32.SizeOfImage,
            pelite::Wrap::T64(opt64) => opt64.SizeOfImage,
        };

        let mut image_cache = vec![0u8; size_of_image as usize].into_boxed_slice();

        // create a map that contains all possible pages
        let mut image_pages =
            vec![false; (size_of_image / PE_PAGE_SIZE.as_u32()) as usize].into_boxed_slice();

        // copy over header page
        image_cache[..image_header.len()].copy_from_slice(&image_header);
        image_pages[0] = true;

        Ok(Self {
            virt_mem: RefCell::new(virt_mem),
            image_base,
            image_format: match view {
                pelite::Wrap::T32(_) => PeFormat::Pe32,
                pelite::Wrap::T64(_) => PeFormat::Pe64,
            },
            image_pages: RefCell::new(image_pages),
            image_cache: UnsafeCell::new(image_cache),
        })
    }

    pub fn image_format(&self) -> PeFormat {
        self.image_format
    }

    pub unsafe fn update_cache(&self, addr: Address, mut len: Length) {
        if len.is_zero() {
            // this is a string read, we just estimate the length of the string here
            len = Length::from_kb(1);
        }

        // TODO: use wraping here

        // always read up to page boundary
        let start_addr = addr.as_page_aligned(PE_PAGE_SIZE);
        let end_addr = (addr + len + PE_PAGE_SIZE).as_page_aligned(PE_PAGE_SIZE);

        let slice = &mut (*self.image_cache.get())[start_addr.as_usize()..end_addr.as_usize()];

        for (chunk_addr, chunk) in PageChunksMut::create_from(slice, start_addr, Length::from_kb(4))
        {
            // chunk_addr is already page aligned
            let page_idx = chunk_addr.as_usize() / PE_PAGE_SIZE.as_usize();
            if !self.image_pages.borrow()[page_idx] {
                self.virt_mem
                    .borrow_mut()
                    .virt_read_raw_into(self.image_base + Length::from(chunk_addr.as_u64()), chunk)
                    .ok();
                self.image_pages.borrow_mut()[page_idx] = true;
            }
        }
    }
}
