use std::prelude::v1::*;

use std::marker::PhantomData;

pub struct FnExtend<T, F> {
    func: F,
    _phantom: PhantomData<T>,
}

impl<T, F: FnMut(T)> FnExtend<T, F> {
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: PhantomData::default(),
        }
    }
}

impl<T> FnExtend<T, fn(T)> {
    pub fn void() -> Self {
        Self {
            func: |_| {},
            _phantom: PhantomData::default(),
        }
    }
}

impl<T, F: FnMut(T)> Extend<T> for FnExtend<T, F> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        iter.into_iter().for_each(&mut self.func);
    }
}
