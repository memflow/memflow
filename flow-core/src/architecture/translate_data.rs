use crate::iter::SplitAtIndex;
use crate::types::Address;
use bumpalo::collections::Vec as BumpVec;
use std::cmp::Ordering;

pub type TranslateVec<'a, T> = BumpVec<'a, (Address, BumpVec<'a, TranslateData<T>>)>;

pub struct TranslateData<T> {
    pub addr: Address,
    pub buf: T,
}

impl<T: SplitAtIndex> Ord for TranslateData<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.addr.cmp(&other.addr)
    }
}

impl<T: SplitAtIndex> Eq for TranslateData<T> {}

impl<T> PartialOrd for TranslateData<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.addr.partial_cmp(&other.addr)
    }
}

impl<T> PartialEq for TranslateData<T> {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr
    }
}

impl<T: SplitAtIndex> SplitAtIndex for TranslateData<T> {
    fn split_inclusive_at(&mut self, idx: usize) -> (Self, Option<Self>)
    where
        Self: Sized,
    {
        let addr = self.addr;
        let (bleft, bright) = self.buf.split_inclusive_at(idx);
        let bl_len = bleft.length();

        (
            TranslateData { addr, buf: bleft },
            if let Some(buf) = bright {
                Some(TranslateData {
                    buf,
                    addr: addr + bl_len,
                })
            } else {
                None
            },
        )
    }

    fn split_at(&mut self, idx: usize) -> (Self, Option<Self>)
    where
        Self: Sized,
    {
        let addr = self.addr;
        let (bleft, bright) = self.buf.split_at(idx);
        let bl_len = bleft.length();

        (
            TranslateData { addr, buf: bleft },
            if let Some(buf) = bright {
                Some(TranslateData {
                    buf,
                    addr: addr + bl_len,
                })
            } else {
                None
            },
        )
    }

    fn length(&self) -> usize {
        self.buf.length()
    }

    fn size_hint(&self) -> usize {
        self.buf.size_hint()
    }
}
