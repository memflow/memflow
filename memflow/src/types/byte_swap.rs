/*!
Trait for byte-swappable basic types.

The trait is used in conjunction with the `#[derive(ByteSwap)]` derive macro.
*/

use core::marker::PhantomData;

/// A trait specifying that a type/struct can be byte swapped.
///
/// This is especially useful when reading/writing from/to targets
/// with a different architecture to the one memflow is compiled with.
///
/// # Examples
///
/// ```
/// use memflow::types::ByteSwap;
/// use memflow_derive::*;
///
/// #[repr(C)]
/// #[derive(ByteSwap)]
/// pub struct Test {
///     pub type1: i32,
///     pub type2: u32,
///     pub type3: i64,
/// }
///
/// let mut test = Test {
///     type1: 10,
///     type2: 1234,
///     type3: -1234,
/// };
/// test.byte_swap();
/// ```
pub trait ByteSwap {
    fn byte_swap(&mut self);
}

// signed types
impl ByteSwap for i8 {
    fn byte_swap(&mut self) {
        // no-op
    }
}

impl ByteSwap for i16 {
    fn byte_swap(&mut self) {
        *self = Self::from_le_bytes(self.to_be_bytes());
    }
}

impl ByteSwap for i32 {
    fn byte_swap(&mut self) {
        *self = Self::from_le_bytes(self.to_be_bytes());
    }
}

impl ByteSwap for i64 {
    fn byte_swap(&mut self) {
        *self = Self::from_le_bytes(self.to_be_bytes());
    }
}

impl ByteSwap for i128 {
    fn byte_swap(&mut self) {
        *self = Self::from_le_bytes(self.to_be_bytes());
    }
}

impl ByteSwap for isize {
    fn byte_swap(&mut self) {
        *self = Self::from_le_bytes(self.to_be_bytes());
    }
}

// unsigned types
impl ByteSwap for u8 {
    fn byte_swap(&mut self) {
        // no-op
    }
}

impl ByteSwap for u16 {
    fn byte_swap(&mut self) {
        *self = Self::from_le_bytes(self.to_be_bytes());
    }
}

impl ByteSwap for u32 {
    fn byte_swap(&mut self) {
        *self = Self::from_le_bytes(self.to_be_bytes());
    }
}

impl ByteSwap for u64 {
    fn byte_swap(&mut self) {
        *self = Self::from_le_bytes(self.to_be_bytes());
    }
}

impl ByteSwap for u128 {
    fn byte_swap(&mut self) {
        *self = Self::from_le_bytes(self.to_be_bytes());
    }
}

impl ByteSwap for usize {
    fn byte_swap(&mut self) {
        *self = Self::from_le_bytes(self.to_be_bytes());
    }
}

// floating point types
impl ByteSwap for f32 {
    fn byte_swap(&mut self) {
        *self = Self::from_le_bytes(self.to_be_bytes());
    }
}

impl ByteSwap for f64 {
    fn byte_swap(&mut self) {
        *self = Self::from_le_bytes(self.to_be_bytes());
    }
}

// pointer types
impl<T: 'static> ByteSwap for *const T {
    fn byte_swap(&mut self) {
        *self = usize::from_le_bytes((*self as usize).to_be_bytes()) as *const T;
    }
}

impl<T: 'static> ByteSwap for *mut T {
    fn byte_swap(&mut self) {
        *self = usize::from_le_bytes((*self as usize).to_be_bytes()) as *mut T;
    }
}

// phantomdata type
impl<T: 'static> ByteSwap for PhantomData<T> {
    fn byte_swap(&mut self) {
        // no-op
    }
}

// slice types
impl<T: ByteSwap> ByteSwap for [T] {
    fn byte_swap(&mut self) {
        self.iter_mut().for_each(|e| e.byte_swap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_i8() {
        let mut num = 100i8;
        num.byte_swap();
        assert_eq!(num, 100i8.swap_bytes());
        num.byte_swap();
        assert_eq!(num, 100);
    }

    #[test]
    fn swap_i16() {
        let mut num = 1234i16;
        num.byte_swap();
        assert_eq!(num, 1234i16.swap_bytes());
        num.byte_swap();
        assert_eq!(num, 1234);
    }

    #[test]
    fn swap_i32() {
        let mut num = 1234i32;
        num.byte_swap();
        assert_eq!(num, 1234i32.swap_bytes());
        num.byte_swap();
        assert_eq!(num, 1234);
    }

    #[test]
    fn swap_i64() {
        let mut num = 1234i64;
        num.byte_swap();
        assert_eq!(num, 1234i64.swap_bytes());
        num.byte_swap();
        assert_eq!(num, 1234);
    }

    #[test]
    fn swap_i128() {
        let mut num = 1234i128;
        num.byte_swap();
        assert_eq!(num, 1234i128.swap_bytes());
        num.byte_swap();
        assert_eq!(num, 1234);
    }

    #[test]
    fn swap_u8() {
        let mut num = 100u8;
        num.byte_swap();
        assert_eq!(num, 100u8.swap_bytes());
        num.byte_swap();
        assert_eq!(num, 100);
    }

    #[test]
    fn swap_u16() {
        let mut num = 1234u16;
        num.byte_swap();
        assert_eq!(num, 1234u16.swap_bytes());
        num.byte_swap();
        assert_eq!(num, 1234);
    }

    #[test]
    fn swap_u32() {
        let mut num = 1234u32;
        num.byte_swap();
        assert_eq!(num, 1234u32.swap_bytes());
        num.byte_swap();
        assert_eq!(num, 1234);
    }

    #[test]
    fn swap_u64() {
        let mut num = 1234u64;
        num.byte_swap();
        assert_eq!(num, 1234u64.swap_bytes());
        num.byte_swap();
        assert_eq!(num, 1234);
    }

    #[test]
    fn swap_u128() {
        let mut num = 1234u128;
        num.byte_swap();
        assert_eq!(num, 1234u128.swap_bytes());
        num.byte_swap();
        assert_eq!(num, 1234);
    }

    #[test]
    fn swap_slice_i16() {
        let mut slice = [1234i16, 50, 64, 128, 200];
        slice.byte_swap();
        assert_eq!(slice[0], 1234i16.swap_bytes());
        slice.byte_swap();
        assert_eq!(slice[0], 1234);
    }
}
