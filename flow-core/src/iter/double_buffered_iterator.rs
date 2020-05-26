use std::collections::VecDeque;

pub type VecType<T> = VecDeque<T>;

pub struct DoubleBufferedMapIterator<I, FI, FO, B, C> {
    iter: I,
    fi: FI,
    fo: FO,
    buf: VecType<B>,
    buf_out: VecType<C>,
}

impl<I, FI, FO, B, C> DoubleBufferedMapIterator<I, FI, FO, B, C> {
    pub fn new(iter: I, fi: FI, fo: FO) -> Self {
        Self {
            iter,
            fi,
            fo,
            buf: VecType::new(),
            buf_out: VecType::new(),
        }
    }
}

impl<I, FI, FO, A, B, C> Iterator for DoubleBufferedMapIterator<I, FI, FO, B, C>
where
    I: Iterator<Item = A>,
    FI: FnMut(A) -> (bool, B),
    FO: FnMut(&mut VecType<B>, &mut VecType<C>),
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
