use crate::address::Address;
use crate::mem::{PageCache, PageType};
use crate::Error;

pub const NO_CACHE: NoCache = NoCache {};

#[derive(Clone)]
pub struct NoCache {}

impl NoCache {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for NoCache {
    fn default() -> Self {
        Self {}
    }
}

impl PageCache for NoCache {
    fn cached_read<F: FnMut(Address, &mut [u8]) -> Result<(), Error>>(
        &mut self,
        start: Address,
        _page_type: PageType,
        out: &mut [u8],
        mut read_fn: F,
    ) -> Result<usize, Error> {
        read_fn(start, out)?;
        Ok(out.len())
    }

    fn cache_page(&mut self, _addr: Address, _page_type: PageType, _src: &[u8]) {}

    fn invalidate_pages(&mut self, mut _addr: Address, _page_type: PageType, _src: &[u8]) {}
}
