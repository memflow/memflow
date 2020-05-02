pub mod no_cache;
pub mod timed_cache;

pub use no_cache::*;
pub use timed_cache::*;

use crate::addr::Address;
use crate::error::Result;
use crate::mem::PageType;

pub trait PageCache {
    fn cached_read<F: FnMut(Address, &mut [u8]) -> Result<()>>(
        &mut self,
        start: Address,
        page_type: PageType,
        out: &mut [u8],
        read_fn: F,
    ) -> Result<usize>;

    fn cache_page(&mut self, addr: Address, page_type: PageType, src: &[u8]);

    fn invalidate_pages(&mut self, addr: Address, page_type: PageType, src: &[u8]);
}
