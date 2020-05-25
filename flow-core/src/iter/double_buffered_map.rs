use std::collections::VecDeque;

pub struct DoubleBufferedMapIterator<I, FI, FO, B, C> {
    iter: I,
    fi: FI,
    fo: FO,
    buf: VecDeque<B>,
    buf_out: VecDeque<C>,
}

impl<I, FI, FO, A, B, C> Iterator for DoubleBufferedMapIterator<I, FI, FO, B, C>
where
    I: Iterator<Item = A>,
    FI: FnMut(A) -> (bool, B),
    FO: FnMut(&mut VecDeque<B>, &mut VecDeque<C>),
{
    type Item = C;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        //If empty, buffer up the output deque
        if self.buf_out.is_empty() {
            while let Some(elem) = self.iter.next() {
                match (self.fi)(elem) {
                    (true, elem) => {
                        self.buf.push_back(elem);
                    }
                    (false, elem) => {
                        self.buf.push_back(elem);
                        break;
                    }
                }
            }

            (self.fo)(&mut self.buf, &mut self.buf_out);
        }

        self.buf_out.pop_front()
    }
}

pub trait DoubleBufferedMap: Iterator {
    /// Split an iterator to chunks, process them, and produce another iterator back
    ///
    /// Yield chunks that are as long as determined by the first predicate `FI: FnMut(Self::Item)
    /// -> (bool, B)`. Pass that chunk to the second predicate `FO: FnMut(&mut VecDeque<B>,
    /// &mut VecDeque<C>)` as a `&mut VecDeque<B>`, where it can be processed into the output
    /// `&mut VecDeque<C>`, which is then used to retrieve individual elements.
    ///
    /// The first predicate has a return type `(bool, B)`, where `bool == false` indicates that
    /// the element is the last element of the current chunk, and `B` is the type that element of
    /// type `A` gets mapped to.
    ///
    /// Output iterator element type is `C`, which is determined by the second predicate `FO`.
    ///
    /// Buffering and mapping (thus, both predicates) get invoked only once the output
    /// `VecDeque<C>` becomes empty.
    ///
    /// Note: For maximum flexibility, the implementation does not clear `VecDeque<B>` after it
    /// gets passed to `FO`. `FO` needs to clear the buffer on its own when iterating `Copy` types
    fn double_buffered_map<FI, FO, B, C>(
        self,
        fi: FI,
        fo: FO,
    ) -> DoubleBufferedMapIterator<Self, FI, FO, B, C>
    where
        Self: Sized,
        FI: FnMut(Self::Item) -> (bool, B),
        FO: FnMut(&mut VecDeque<B>, &mut VecDeque<C>),
    {
        DoubleBufferedMapIterator {
            iter: self,
            fi,
            fo,
            buf: VecDeque::new(),
            buf_out: VecDeque::new(),
        }
    }
}

impl<T> DoubleBufferedMap for T where T: Iterator {}
