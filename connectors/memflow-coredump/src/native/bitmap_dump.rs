use std::fs::File;
use std::io::Read;
use std::mem::MaybeUninit;

use dataview::Pod;
use log::{debug, info};
use memflow_core::*;

const BMP_SIGNATURE: u32 = 0x504D4446; // 'PMDF'
const VALID_DUMP: u32 = 0x504D5544; // 'PMUD'

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BmpHeader {
    pub signature: u32,
    pub valid_dump: u32,
    // There is some padding over here
    pub _pad: [u8; 0x20 - 0x8],

    pub first_page: u64,
    pub total_present_pages: u64,
    pub pages: u64,
}

#[allow(clippy::uninit_assumed_init)]
impl BmpHeader {
    pub fn uninit() -> Self {
        unsafe { MaybeUninit::uninit().assume_init() }
    }
}

unsafe impl Pod for BmpHeader {}

pub fn parse_bitmap_dump(file: &mut File) -> Result<MemoryMap<(Address, usize)>> {
    let mut header = BmpHeader::uninit();

    file.read_exact(header.as_bytes_mut())
        .map_err(|_| Error::Connector("unable to read bitmap header"))?;

    if header.signature != BMP_SIGNATURE {
        return Err(Error::Connector("BitMap header signature is not valid"));
    }

    if header.valid_dump != VALID_DUMP {
        return Err(Error::Connector("BitMap header dump flag is not valid"));
    }

    info!(
        "BitMap dump - first_page: {:x} present_pages: {:x} page_offset: {:x}",
        header.first_page, header.total_present_pages, header.pages,
    );

    type BType = u128;
    const BITS_PER_VAL: usize = std::mem::size_of::<BType>() * 8;

    let mut bitmap: Vec<BType> = vec![];
    // Add an additional elem so that the final reg gets pushed
    bitmap.resize(header.pages as usize / BITS_PER_VAL + 1, 0);

    file.read_exact(&mut bitmap.as_bytes_mut()[..((header.pages as usize + 7) / 8)])
        .map_err(|_| Error::Connector("unable to read the bitmap"))?;

    let mut reg_accum_bit = 0usize;
    let mut reg_start_bit = 0usize;
    let mut accum_bits = 0usize;

    let mut cur_bit = 0usize;

    let mut mem_map = MemoryMap::new();

    let real_base: Address = header.first_page.into();

    for b in bitmap.into_iter() {
        if b != BType::MAX {
            let mut i = 0usize;

            // Not exactly sure if this would behave well on big endian arch, we may need to .to_le()
            let mut temp = b;

            while i < BITS_PER_VAL {
                let ones = std::cmp::min((!temp).trailing_zeros() as usize, BITS_PER_VAL - i);
                cur_bit += ones;
                i += ones;
                temp = temp.wrapping_shr(ones as u32);
                accum_bits += ones;

                let zeros = std::cmp::min(temp.trailing_zeros() as usize, BITS_PER_VAL - i);

                if zeros > 0 {
                    i += zeros;
                    cur_bit += zeros;
                    temp = temp.wrapping_shr(zeros as u32);

                    if reg_accum_bit != accum_bits {
                        let base = reg_start_bit as usize * 0x1000usize;
                        let remap_base = real_base + reg_accum_bit as usize * 0x1000usize;
                        let size = (accum_bits - reg_accum_bit) as usize * 0x1000usize;

                        debug!(
                            "adding memory mapping: base={:x} size={:x} real_base={:x}",
                            base, size, remap_base
                        );

                        mem_map.push_remap(base.into(), size, remap_base);
                        reg_accum_bit = accum_bits;
                    }

                    reg_start_bit = cur_bit;
                }
            }
        } else {
            cur_bit += BITS_PER_VAL;
            accum_bits += BITS_PER_VAL;
        }
    }

    info!(
        "total bits {:x} map size: {:x}",
        accum_bits,
        mem_map.iter().count()
    );

    Ok(mem_map)
}
