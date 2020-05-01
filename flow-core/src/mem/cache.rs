use super::MemCache;
use crate::address::{Address, Length};
use crate::mem::PageType;
use crate::Error;
use coarsetime::{Duration, Instant};

#[derive(Clone)]
pub struct TimedCache {
    address: Box<[Address]>,
    time: Box<[Instant]>,
    cache: Box<[u8]>,
    cache_time: Duration,
    page_size: Length,
    page_mask: PageType,
}

impl TimedCache {
    pub fn new(
        cache_time_millis: u64,
        cache_size: usize,
        page_size: Length,
        page_mask: PageType,
    ) -> Self {
        Self {
            address: vec![(!0_u64).into(); cache_size].into_boxed_slice(),
            time: vec![Instant::now(); cache_size].into_boxed_slice(),
            cache: vec![0; cache_size * page_size.as_usize()].into_boxed_slice(),
            cache_time: Duration::from_millis(cache_time_millis),
            page_size,
            page_mask,
        }
    }

    pub fn none_cache() -> Self {
        Self::new(0, 0, 0.into(), PageType::NONE)
    }

    fn page_index(&self, addr: Address) -> usize {
        (addr.as_usize() / self.page_size.as_usize()) % self.address.len()
    }

    fn page_and_info_from_index(&mut self, idx: usize) -> (&mut [u8], &mut Address, &mut Instant) {
        let start = self.page_size.as_usize() * idx;
        (
            &mut self.cache[start..(start + self.page_size.as_usize())],
            &mut self.address[idx],
            &mut self.time[idx],
        )
    }

    fn page_from_index(&mut self, idx: usize) -> &mut [u8] {
        let start = self.page_size.as_usize() * idx;
        &mut self.cache[start..(start + self.page_size.as_usize())]
    }

    fn try_page_with_time(
        &mut self,
        addr: Address,
        time: Instant,
    ) -> Result<&mut [u8], (&mut [u8], &mut Address, &mut Instant)> {
        let page_index = self.page_index(addr);
        if self.address[page_index] == addr.as_page_aligned(self.page_size)
            && time.duration_since(self.time[page_index]) <= self.cache_time
        {
            Ok(self.page_from_index(page_index))
        } else {
            Err(self.page_and_info_from_index(page_index))
        }
    }
}

impl MemCache for TimedCache {
    fn cached_read<F: FnMut(Address, &mut [u8]) -> Result<(), Error>>(
        &mut self,
        start: Address,
        page_type: PageType,
        out: &mut [u8],
        mut read_fn: F,
    ) -> Result<usize, Error> {
        if (self.page_mask & page_type) == PageType::NONE {
            read_fn(start, out)?;
            Ok(out.len())
        } else {
            let mut cur_addr = start.as_u64();

            let page_mask = self.page_size.as_u64() - 1;
            let page_up = self.page_size + ((cur_addr - 1) & !page_mask) - cur_addr;
            let mut cur_page: Address = (start.as_u64() & !page_mask).into();

            let instant = Instant::now();
            let mut read = 0;

            let (start_buf, end_buf) =
                out.split_at_mut(std::cmp::min(page_up.as_usize(), out.len()));

            for b in [start_buf, end_buf].iter_mut() {
                let mut page_iter = b.chunks_mut(self.page_size.as_usize());
                while let Some(chunk) = page_iter.next() {
                    let page = match self.try_page_with_time(cur_addr.into(), instant) {
                        Err((page, address, time)) => {
                            read_fn(cur_page, page)?;
                            *address = cur_page;
                            *time = instant;
                            page
                        }
                        Ok(page) => page,
                    };
                    let copy_start = (cur_addr & page_mask) as usize;
                    chunk.copy_from_slice(&page[copy_start..(copy_start + chunk.len())]);
                    read += chunk.len();

                    cur_addr += chunk.len() as u64;
                    cur_page += self.page_size;
                }
            }

            Ok(read)
        }
    }

    fn cache_page(&mut self, addr: Address, page_type: PageType, src: &[u8]) {
        if self.page_mask & page_type != PageType::NONE {
            let page_index = self.page_index(addr);
            self.page_from_index(page_index).copy_from_slice(src);
            self.address[page_index] = addr;
            self.time[page_index] = Instant::now();
        }
    }

    fn invalidate_pages(&mut self, mut addr: Address, page_type: PageType, src: &[u8]) {
        if self.page_mask & page_type != PageType::NONE {
            addr = addr.as_page_aligned(self.page_size);
            for i in (0..src.len()).step_by(self.page_size.as_usize()) {
                let cur_addr = addr + Length::from(i);
                let page_index = self.page_index(cur_addr);
                if self.address[page_index].as_page_aligned(self.page_size) == cur_addr {
                    self.address[page_index] = Address::from(!0_u64);
                }
            }
        }
    }
}
