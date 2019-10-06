use log::{info, trace, warn};

use std::ops::{Index, RangeFrom};

use address::{Address, Length};
use mem::{PhysicalRead, VirtualRead};
use scroll::ctx::MeasureWith;

use crate::dtb::DTB;

// TODO: move this in a seperate crate as a elf/pe/macho helper for pa/va

// TODO: we need both a physical and virtual reader, our use case is va though
pub struct VirtualScrollReader<'a, T: PhysicalRead + VirtualRead> {
    mem: &'a mut T,
    dtb: DTB,
    base: Address,
}

impl<'a, T: PhysicalRead + VirtualRead> VirtualScrollReader<'a, T> {
    pub fn new(mem: &'a mut T, dtb: DTB, base: Address) -> Self {
        VirtualScrollReader{
            mem: mem,
            dtb: dtb,
            base: base,
        }
    }
}

/*
impl<'a, T: PhysicalRead + VirtualRead> Index<usize> for VirtualScrollReader<'a, T> {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        info!("VirtualScrollReader(): reading byte at {:x}", idx);
        let buf = self.mem.virt_read(self.dtb.arch, self.dtb.dtb, self.base + Length::from(idx), Length::from_b(1)).unwrap();
        &buf[0]
    }
}

impl<'a, T: PhysicalRead + VirtualRead> Index<RangeFrom<usize>> for VirtualScrollReader<'a, T> {
    type Output = [u8];

    fn index(&self, range: RangeFrom<usize>) -> &[u8] {
        info!("VirtualScrollReader(): reading range from {:x}", range.start);
        let buf = self.mem.virt_read(self.dtb.arch, self.dtb.dtb, self.base + Length::from(range.start), Length::from_mb(2)).unwrap();
        &buf
    }
}

impl<'a, T: PhysicalRead + VirtualRead, Ctx> MeasureWith<Ctx> for VirtualScrollReader<'a, T> {
    type Units = usize;

    #[inline]
    fn measure_with(&self, _ctx: &Ctx) -> Self::Units {
        // TODO: return a somewhat senseful length here based on ram limits?
        //println!("measuring results in len {}", self.buf.len());
        Length::from_gb(16).as_usize()
    }
}
*/