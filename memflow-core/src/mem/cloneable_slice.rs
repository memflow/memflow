use crate::iter::{SplitAtIndex, SplitAtIndexNoMutation};
use crate::types::TryAsMut;

#[repr(transparent)]
pub struct CloneableSliceMut<'a, T>(&'a mut [T]);

impl<'a, T> CloneableSliceMut<'a, T> {
    pub unsafe fn from_slice_mut(elem: &'a mut [T]) -> Self {
        Self { 0: elem }
    }
}

impl<'a> AsRef<[u8]> for CloneableSliceMut<'a, u8> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl<'a, T> std::ops::Deref for CloneableSliceMut<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<'a, T> std::ops::DerefMut for CloneableSliceMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

// This is inherently unsafe,
// but the rest of the library is inherently unsafe too,
// aliasing rules can be ignored here.
impl<'a, T> Clone for CloneableSliceMut<'a, T> {
    fn clone(&self) -> Self {
        let cloned_slice =
            unsafe { std::slice::from_raw_parts_mut(self.0.as_ptr() as *mut _, self.0.len()) };
        Self { 0: cloned_slice }
    }
}

impl<'a, T> SplitAtIndexNoMutation for CloneableSliceMut<'a, T> {}

impl<'a, T> SplitAtIndex for CloneableSliceMut<'a, T> {
    fn split_inclusive_at(&mut self, idx: usize) -> (Self, Option<Self>) {
        let (left, right) = self.0.split_inclusive_at(idx);
        (Self { 0: left }, right.map(|s| Self { 0: s }))
    }

    fn split_at(&mut self, idx: usize) -> (Self, Option<Self>) {
        let (left, right) = self.0.split_at(idx);
        (Self { 0: left }, right.map(|s| Self { 0: s }))
    }

    fn length(&self) -> usize {
        self.0.length()
    }

    fn size_hint(&self) -> usize {
        self.0.size_hint()
    }
}

impl<'a, T> TryAsMut<[T]> for CloneableSliceMut<'a, T> {
    fn try_as_mut(&mut self) -> Option<&mut [T]> {
        Some(self.as_mut())
    }
}
