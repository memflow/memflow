pub struct DoublePeekingIterator<I>
where
    I: Iterator,
{
    iter: I,
    next: Option<I::Item>,
    next2: Option<I::Item>,
}

impl<I> DoublePeekingIterator<I>
where
    I: Iterator,
{
    /// Construct a double peeking iterator
    ///
    /// It will consume the next 2 elements upon call
    pub fn new(mut iter: I) -> Self {
        Self {
            next: iter.next(),
            next2: iter.next(),
            iter,
        }
    }

    /// Peek 2 elements without moving the iterator's head
    pub fn double_peek(&self) -> (&Option<I::Item>, &Option<I::Item>) {
        (&self.next, &self.next2)
    }

    /// Check if there isn't an element after the next one
    ///
    /// This will check if the second next element is none.
    /// It will still return true if next element is None,
    /// and it may return false on unfused iterators that happen
    /// to have None elements in the middle.
    pub fn is_next_last(&self) -> bool {
        self.next2.is_none()
    }
}

impl<I> Iterator for DoublePeekingIterator<I>
where
    I: Iterator,
{
    type Item = I::Item;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        std::mem::replace(
            &mut self.next,
            std::mem::replace(&mut self.next2, self.iter.next()),
        )
    }
}
