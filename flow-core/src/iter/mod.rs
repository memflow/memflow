pub mod page_chunks;
use crate::types::Address;
pub use page_chunks::*;

mod double_buffered_iterator;
use double_buffered_iterator::*;

mod doublepeek;
use doublepeek::*;

pub trait FlowIters: Iterator {
    /// Split an iterator to chunks, process them, and produce another iterator back
    ///
    /// Yield chunks that are as long as determined by the first predicate `FI: FnMut(Self::Item)
    /// -> (bool, B)`. Pass that chunk to the second predicate `FO: FnMut(&mut VecType<B>,
    /// &mut VecType<C>)` as a `&mut VecType<B>`, where it can be processed into the output
    /// `&mut VecType<C>`, which is then used to retrieve individual elements.
    ///
    /// The first predicate has a return type `(bool, B)`, where `bool == false` indicates that
    /// the element is the last element of the current chunk, and `B` is the type that element of
    /// type `A` gets mapped to.
    ///
    /// Output iterator element type is `C`, which is determined by the second predicate `FO`.
    ///
    /// Buffering and mapping (thus, both predicates) get invoked only once the output
    /// `VecType<C>` becomes empty.
    ///
    /// Note: For maximum flexibility, the implementation does not clear `VecType<B>` after it
    /// gets passed to `FO`. `FO` needs to clear the buffer on its own when iterating `Copy` types
    fn double_buffered_map<FI, FO, B, C>(
        self,
        fi: FI,
        fo: FO,
    ) -> DoubleBufferedMapIterator<Self, FI, FO, B, C>
    where
        Self: Sized,
        FI: FnMut(Self::Item) -> (bool, B),
        FO: FnMut(&mut VecType<B>, &mut VecType<C>),
    {
        DoubleBufferedMapIterator::new(self, fi, fo)
    }

    fn double_peekable(self) -> DoublePeekingIterator<Self>
    where
        Self: Sized,
    {
        DoublePeekingIterator::<Self>::new(self)
    }
}

impl<T> FlowIters for T where T: Iterator {}

type TrueFunc<T> = fn(Address, &T, Option<&T>) -> bool;

pub trait PageChunks {
    fn page_chunks(
        self,
        start_address: Address,
        page_size: usize,
    ) -> PageChunkIterator<Self, TrueFunc<Self>>
    where
        Self: SplitAtIndex + Sized,
    {
        PageChunkIterator::new(self, start_address, page_size, |_, _, _| true)
    }

    fn page_chunks_by<F: FnMut(Address, &Self, Option<&Self>) -> bool>(
        self,
        start_address: Address,
        page_size: usize,
        split_fn: F,
    ) -> PageChunkIterator<Self, F>
    where
        Self: SplitAtIndex + Sized,
    {
        PageChunkIterator::new(self, start_address, page_size, split_fn)
    }
}

impl<T> PageChunks for T where T: SplitAtIndex {}

#[cfg(test)]
mod tests {
    use crate::PageChunks;

    const PAGE_SIZE: usize = 97;
    const OFF: usize = 26;
    const ADDEND: usize = 17;

    #[test]
    fn pc_check_all_aligned_zero() {
        let arr = [0_u8; 0x1000];

        for (addr, _chunk) in arr.page_chunks(0.into(), PAGE_SIZE) {
            assert_eq!(addr.as_page_aligned(PAGE_SIZE), addr);
        }
    }

    #[test]
    fn pc_check_all_chunks_equal() {
        let arr = [0_u8; 100 * PAGE_SIZE];

        for (_addr, chunk) in arr.page_chunks(0.into(), PAGE_SIZE) {
            println!("{:x} {:x}", _addr, chunk.len());
            assert_eq!(chunk.len(), PAGE_SIZE);
        }
    }

    #[test]
    fn pc_check_all_chunks_equal_first_not() {
        const OFF: usize = 26;
        let arr = [0_u8; 100 * PAGE_SIZE + (PAGE_SIZE - OFF)];

        let mut page_iter = arr.page_chunks(OFF.into(), PAGE_SIZE);

        {
            let (addr, chunk) = page_iter.next().unwrap();
            assert_eq!(addr, OFF.into());
            assert_eq!(chunk.len(), PAGE_SIZE - OFF);
        }

        for (_addr, chunk) in page_iter {
            assert_eq!(chunk.len(), PAGE_SIZE);
        }
    }

    #[test]
    fn pc_check_everything() {
        const TOTAL_LEN: usize = 100 * PAGE_SIZE + ADDEND - OFF;
        let arr = [0_u8; TOTAL_LEN];

        let mut cur_len = 0;
        let mut prev_len = 0;

        let mut page_iter = arr.page_chunks(OFF.into(), PAGE_SIZE);

        {
            let (addr, chunk) = page_iter.next().unwrap();
            assert_eq!(addr, OFF.into());
            assert_eq!(chunk.len(), PAGE_SIZE - OFF);
            cur_len += chunk.len();
        }

        for (_addr, chunk) in page_iter {
            if chunk.len() != ADDEND {
                assert_eq!(chunk.len(), PAGE_SIZE);
            }
            prev_len = chunk.len();
            cur_len += prev_len;
        }

        assert_eq!(prev_len, ADDEND);
        assert_eq!(cur_len, TOTAL_LEN);
    }

    #[test]
    fn pc_check_size_hint() {
        const PAGE_COUNT: usize = 5;
        let arr = [0_u8; PAGE_SIZE * PAGE_COUNT];
        assert_eq!(
            arr.page_chunks(0.into(), PAGE_SIZE).size_hint().0,
            PAGE_COUNT
        );
        assert_eq!(
            arr.page_chunks(1.into(), PAGE_SIZE).size_hint().0,
            PAGE_COUNT + 1
        );
        assert_eq!(
            arr.page_chunks((PAGE_SIZE - 1).into(), PAGE_SIZE)
                .size_hint()
                .0,
            PAGE_COUNT + 1
        );
        assert_eq!(
            arr.page_chunks(PAGE_SIZE.into(), PAGE_SIZE).size_hint().0,
            PAGE_COUNT
        );
    }
}
