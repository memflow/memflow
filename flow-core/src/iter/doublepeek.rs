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
    pub fn new(mut iter: I) -> Self {
        Self {
            next: iter.next(),
            next2: iter.next(),
            iter,
        }
    }

    pub fn double_peek(&self) -> (&Option<I::Item>, &Option<I::Item>) {
        (&self.next, &self.next2)
    }

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
        let ret = std::mem::replace(
            &mut self.next,
            std::mem::replace(&mut self.next2, self.iter.next()),
        );
        ret
    }
}
