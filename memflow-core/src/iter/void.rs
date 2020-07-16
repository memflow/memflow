use std::prelude::v1::*;

use std::marker::PhantomData;

pub struct ExtendVoid<T, F> {
    func: F,
    _phantom: PhantomData<T>,
}

impl<T, F: FnMut(T)> ExtendVoid<T, F> {
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: PhantomData::default(),
        }
    }
}

impl<T> ExtendVoid<T, fn(T)> {
    pub fn void() -> Self {
        Self {
            func: |_| {},
            _phantom: PhantomData::default(),
        }
    }
}

impl<T, F: FnMut(T)> Extend<T> for ExtendVoid<T, F> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        iter.into_iter().for_each(&mut self.func);
    }
}
