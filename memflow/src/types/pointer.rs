/*!
Pointer abstraction.
*/

use crate::cglue::ReprCString;
use crate::dataview::Pod;
use crate::error::{Error, ErrorKind, ErrorOrigin, PartialError, PartialResult};
use crate::mem::MemoryView;
use crate::types::{Address, ByteSwap};

use std::convert::TryFrom;
use std::marker::PhantomData;
use std::mem::size_of;
use std::{cmp, fmt, hash, ops};

/// This type can be used in structs that are being read from the target memory.
/// It holds a phantom type that can be used to describe the proper type of the pointer
/// and to read it in a more convenient way.
///
/// This module is a direct adaption of [CasualX's great IntPtr crate](https://github.com/CasualX/intptr).
///
/// Generally the generic Type should implement the Pod trait to be read into easily.
/// See [here](https://docs.rs/dataview/0.1.1/dataview/) for more information on the Pod trait.
///
/// # Examples
///
/// ```
/// use memflow::types::Pointer;
/// use memflow::mem::MemoryView;
/// use memflow::dataview::Pod;
///
/// #[repr(C)]
/// #[derive(Clone, Debug, Pod)]
/// struct Foo {
///     pub some_value: i64,
/// }
///
/// #[repr(C)]
/// #[derive(Clone, Debug, Pod)]
/// struct Bar {
///     pub foo_ptr: Pointer<Foo>,
/// }
///
/// fn read_foo_bar(mem: &mut impl MemoryView) {
///     let bar: Bar = mem.read(0x1234.into()).unwrap();
///     let foo = bar.foo_ptr.read(mem).unwrap();
///     println!("value: {}", foo.some_value);
/// }
///
/// # use memflow::types::size;
/// # use memflow::dummy::DummyOs;
/// # use memflow::os::Process;
/// # read_foo_bar(&mut DummyOs::quick_process(size::mb(2), &[]));
/// ```
///
/// ```
/// use memflow::types::Pointer;
/// use memflow::mem::MemoryView;
/// use memflow::dataview::Pod;
///
/// #[repr(C)]
/// #[derive(Clone, Debug, Pod)]
/// struct Foo {
///     pub some_value: i64,
/// }
///
/// #[repr(C)]
/// #[derive(Clone, Debug, Pod)]
/// struct Bar {
///     pub foo_ptr: Pointer<Foo>,
/// }
///
/// fn read_foo_bar(mem: &mut impl MemoryView) {
///     let bar: Bar = mem.read(0x1234.into()).unwrap();
///     let foo = mem.read_ptr(bar.foo_ptr).unwrap();
///     println!("value: {}", foo.some_value);
/// }
///
/// # use memflow::dummy::DummyOs;
/// # use memflow::os::Process;
/// # use memflow::types::size;
/// # read_foo_bar(&mut DummyOs::quick_process(size::mb(2), &[]));
/// ```
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct Pointer<T: ?Sized = ()> {
    pub address: Address,
    phantom_data: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Pointer<T> {
    const PHANTOM_DATA: PhantomData<fn() -> T> = PhantomData;

    /// A pointer64 with the value of zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer;
    ///
    /// println!("pointer: {}", Pointer::<()>::NULL);
    /// ```
    pub const NULL: Pointer<T> = Pointer {
        address: Address::NULL,
        phantom_data: PhantomData,
    };

    /// Returns a pointer64 with a value of zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer;
    ///
    /// println!("pointer: {}", Pointer::<()>::null());
    /// ```
    pub const fn null() -> Self {
        Pointer::NULL
    }

    /// Returns `true` if the pointer64 is null.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer;
    ///
    /// let ptr = Pointer::<()>::from(0x1000u32);
    /// assert!(!ptr.is_null());
    /// ```
    pub const fn is_null(self) -> bool {
        self.address.is_null()
    }

    /// Converts the pointer64 to an Option that is None when it is null
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer;
    ///
    /// assert_eq!(Pointer::<()>::null().non_null(), None);
    /// assert_eq!(Pointer::<()>::from(0x1000u64).non_null(), Some(Pointer::from(0x1000u64)));
    /// ```
    #[inline]
    pub fn non_null(self) -> Option<Pointer<T>> {
        if self.is_null() {
            None
        } else {
            Some(self)
        }
    }

    /// Converts the pointer64 into a `u32` value.
    ///
    /// # Remarks:
    ///
    /// This function internally uses `as u32` which can cause a wrap-around
    /// in case the internal 64-bit value does not fit the 32-bit `u32`.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer;
    ///
    /// let ptr = Pointer::<()>::from(0x1000u64);
    /// let ptr_u32: u32 = ptr.as_u32();
    /// assert_eq!(ptr_u32, 0x1000);
    /// ```
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.address.as_u64() as u32
    }

    /// Converts the pointer64 into a `u64` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer;
    ///
    /// let ptr = Pointer::<()>::from(0x1000u64);
    /// let ptr_u64: u64 = ptr.as_u64();
    /// assert_eq!(ptr_u64, 0x1000);
    /// ```
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.address.as_u64()
    }

    /// Converts the pointer64 into a `usize` value.
    ///
    /// # Remarks:
    ///
    /// When compiling for a 32-bit architecture the size of `usize`
    /// is only 32-bit. Since this function internally uses `as usize` it can cause a wrap-around
    /// in case the internal 64-bit value does not fit in the 32-bit `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer;
    ///
    /// let ptr = Pointer::<()>::from(0x1000u64);
    /// let ptr_usize: usize = ptr.as_usize();
    /// assert_eq!(ptr_usize, 0x1000);
    /// ```
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.address.as_u64() as usize
    }

    pub const fn address(&self, arch_bits: u8, little_endian: bool) -> Address {
        let addr = self.address.as_u64();
        let addr = addr & Address::bit_mask_u8(0..(arch_bits - 1)).as_u64();
        // TODO: this swapping is probably wrong,
        // and it will probably need to be swapped automatically when reads are performed.
        let addr = if cfg!(target_endian = "little") != little_endian {
            addr.swap_bytes()
        } else {
            addr
        };
        Address::from_u64(addr)
    }
}

impl<T: Sized> Pointer<T> {
    /// Calculates the offset from a pointer64
    ///
    /// `count` is in units of T; e.g., a `count` of 3 represents a pointer offset of `3 * size_of::<T>()` bytes.
    ///
    /// # Panics
    ///
    /// This function panics if `T` is a Zero-Sized Type ("ZST").
    /// This function also panics when `offset * size_of::<T>()`
    /// causes overflow of a signed 64-bit integer.
    ///
    /// # Examples:
    ///
    /// ```
    /// use memflow::types::Pointer;
    ///
    /// let ptr = Pointer::<u16>::from(0x1000u64);
    ///
    /// println!("{:?}", ptr.offset(3));
    /// ```
    pub fn offset(self, count: i64) -> Self {
        let pointee_size = size_of::<T>();
        assert!(0 < pointee_size && pointee_size <= i64::MAX as usize);

        if count >= 0 {
            self.address
                .wrapping_add(pointee_size * count as usize)
                .into()
        } else {
            self.address
                .wrapping_sub(pointee_size * (-count) as usize)
                .into()
        }
    }

    /// Calculates the distance between two pointers. The returned value is in
    /// units of T: the distance in bytes is divided by `mem::size_of::<T>()`.
    ///
    /// This function is the inverse of [`offset`].
    ///
    /// [`offset`]: #method.offset
    ///
    /// # Panics
    ///
    /// This function panics if `T` is a Zero-Sized Type ("ZST").
    ///
    /// # Examples:
    ///
    /// ```
    /// use memflow::types::Pointer;
    ///
    /// let ptr1 = Pointer::<u16>::from(0x1000u64);
    /// let ptr2 = Pointer::<u16>::from(0x1008u64);
    ///
    /// assert_eq!(ptr2.offset_from(ptr1), 4);
    /// assert_eq!(ptr1.offset_from(ptr2), -4);
    /// ```
    pub fn offset_from(self, origin: Self) -> i64 {
        let pointee_size = size_of::<T>();
        assert!(0 < pointee_size && pointee_size <= i64::MAX as usize);

        let offset = self
            .address
            .wrapping_sub(origin.address.as_u64() as usize)
            .as_u64() as i64;
        offset / pointee_size as i64
    }

    /// Calculates the offset from a pointer (convenience for `.offset(count as i64)`).
    ///
    /// `count` is in units of T; e.g., a `count` of 3 represents a pointer
    /// offset of `3 * size_of::<T>()` bytes.
    ///
    /// # Panics
    ///
    /// This function panics if `T` is a Zero-Sized Type ("ZST").
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use memflow::types::Pointer;
    ///
    /// let ptr = Pointer::<u16>::from(0x1000u64);
    ///
    /// println!("{:?}", ptr.add(3));
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, count: u64) -> Self {
        self.offset(count as i64)
    }

    /// Calculates the offset from a pointer (convenience for
    /// `.offset((count as isize).wrapping_neg())`).
    ///
    /// `count` is in units of T; e.g., a `count` of 3 represents a pointer
    /// offset of `3 * size_of::<T>()` bytes.
    ///
    /// # Panics
    ///
    /// This function panics if `T` is a Zero-Sized Type ("ZST").
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use memflow::types::Pointer;
    ///
    /// let ptr = Pointer::<u16>::from(0x1000u64);
    ///
    /// println!("{:?}", ptr.sub(3));
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, count: u64) -> Self {
        self.offset((count as i64).wrapping_neg())
    }
}

/// Implement special phys/virt read/write for Pod types
impl<T: Pod + ?Sized> Pointer<T> {
    pub fn read_into<U: MemoryView>(self, mem: &mut U, out: &mut T) -> PartialResult<()> {
        mem.read_ptr_into(self, out)
    }
}

impl<T: Pod + Sized> Pointer<T> {
    pub fn read<U: MemoryView>(self, mem: &mut U) -> PartialResult<T> {
        mem.read_ptr(self)
    }

    pub fn write<U: MemoryView>(self, mem: &mut U, data: &T) -> PartialResult<()> {
        mem.write_ptr(self, data)
    }
}

/// Implement special phys/virt read/write for CReprStr
impl Pointer<ReprCString> {
    pub fn read_string<U: MemoryView>(self, mem: &mut U) -> PartialResult<ReprCString> {
        match mem.read_char_string(self.address.into()) {
            Ok(s) => Ok(s.into()),
            Err(PartialError::Error(e)) => Err(PartialError::Error(e)),
            Err(PartialError::PartialVirtualRead(s)) => {
                Err(PartialError::PartialVirtualRead(s.into()))
            }
            Err(PartialError::PartialVirtualWrite(s)) => {
                Err(PartialError::PartialVirtualWrite(s.into()))
            }
        }
    }
}

impl<T> Pointer<[T]> {
    pub const fn decay(self) -> Pointer<T> {
        Pointer {
            address: self.address,
            phantom_data: Pointer::<T>::PHANTOM_DATA,
        }
    }

    pub const fn at(self, i: usize) -> Pointer<T> {
        let address = self.address.wrapping_add(i * size_of::<T>());
        Pointer {
            address,
            phantom_data: Pointer::<T>::PHANTOM_DATA,
        }
    }
}

impl<T: ?Sized> Copy for Pointer<T> {}
impl<T: ?Sized> Clone for Pointer<T> {
    #[inline(always)]
    fn clone(&self) -> Pointer<T> {
        *self
    }
}
impl<T: ?Sized> Default for Pointer<T> {
    #[inline(always)]
    fn default() -> Pointer<T> {
        Pointer::NULL
    }
}
impl<T: ?Sized> Eq for Pointer<T> {}
impl<T: ?Sized> PartialEq for Pointer<T> {
    #[inline(always)]
    fn eq(&self, rhs: &Pointer<T>) -> bool {
        self.address == rhs.address
    }
}
impl<T: ?Sized> PartialOrd for Pointer<T> {
    #[inline(always)]
    fn partial_cmp(&self, rhs: &Pointer<T>) -> Option<cmp::Ordering> {
        self.address.partial_cmp(&rhs.address)
    }
}
impl<T: ?Sized> Ord for Pointer<T> {
    #[inline(always)]
    fn cmp(&self, rhs: &Pointer<T>) -> cmp::Ordering {
        self.address.cmp(&rhs.address)
    }
}
impl<T: ?Sized> hash::Hash for Pointer<T> {
    #[inline(always)]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.address.as_u64().hash(state)
    }
}
impl<T: ?Sized> AsRef<Address> for Pointer<T> {
    #[inline(always)]
    fn as_ref(&self) -> &Address {
        &self.address
    }
}
impl<T: ?Sized> AsMut<Address> for Pointer<T> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut Address {
        &mut self.address
    }
}

// From implementations
impl<T: ?Sized> From<u32> for Pointer<T> {
    #[inline(always)]
    fn from(address: u32) -> Pointer<T> {
        Pointer {
            address: Address::from(address),
            phantom_data: PhantomData,
        }
    }
}

impl<T: ?Sized> From<u64> for Pointer<T> {
    #[inline(always)]
    fn from(address: u64) -> Pointer<T> {
        Pointer {
            address: Address::from(address),
            phantom_data: PhantomData,
        }
    }
}

impl<T: ?Sized> From<Address> for Pointer<T> {
    #[inline(always)]
    fn from(address: Address) -> Pointer<T> {
        Pointer {
            address,
            phantom_data: PhantomData,
        }
    }
}

// Into implementations
impl<T: ?Sized> From<Pointer<T>> for Address {
    #[inline(always)]
    fn from(ptr: Pointer<T>) -> Address {
        ptr.address
    }
}

impl<T: ?Sized> From<Pointer<T>> for u64 {
    #[inline(always)]
    fn from(ptr: Pointer<T>) -> u64 {
        ptr.address.as_u64()
    }
}

/// Tries to convert a Pointer into a u32.
/// The function will return an `Error::Bounds` error if the input value is greater than `u32::max_value()`.
impl<T: ?Sized> TryFrom<Pointer<T>> for u32 {
    type Error = crate::error::Error;

    fn try_from(ptr: Pointer<T>) -> std::result::Result<u32, Self::Error> {
        if ptr.address <= Address::from(u32::max_value()) {
            Ok(ptr.address.as_u64() as u32)
        } else {
            Err(Error(ErrorOrigin::Pointer, ErrorKind::OutOfBounds))
        }
    }
}

// Arithmetic operations
impl<T> ops::Add<usize> for Pointer<T> {
    type Output = Pointer<T>;
    #[inline(always)]
    fn add(self, other: usize) -> Pointer<T> {
        let address = self.address + (other * size_of::<T>());
        Pointer {
            address,
            phantom_data: self.phantom_data,
        }
    }
}
impl<T> ops::Sub<usize> for Pointer<T> {
    type Output = Pointer<T>;
    #[inline(always)]
    fn sub(self, other: usize) -> Pointer<T> {
        let address = self.address - (other * size_of::<T>());
        Pointer {
            address,
            phantom_data: self.phantom_data,
        }
    }
}

impl<T: ?Sized> fmt::Debug for Pointer<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.address)
    }
}
impl<T: ?Sized> fmt::UpperHex for Pointer<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}", self.address)
    }
}
impl<T: ?Sized> fmt::LowerHex for Pointer<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.address)
    }
}
impl<T: ?Sized> fmt::Display for Pointer<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.address)
    }
}

unsafe impl<T: ?Sized + 'static> Pod for Pointer<T> {}
const _: [(); std::mem::size_of::<Pointer<()>>()] = [(); std::mem::size_of::<u64>()];

impl<T: ?Sized + 'static> ByteSwap for Pointer<T> {
    fn byte_swap(&mut self) {
        self.address.byte_swap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset() {
        let ptr8 = Pointer::<u8>::from(0x1000u64);
        assert_eq!(ptr8.offset(3).as_u64(), 0x1003u64);
        assert_eq!(ptr8.offset(-5).as_u64(), 0xFFBu64);

        let ptr16 = Pointer::<u16>::from(0x1000u64);
        assert_eq!(ptr16.offset(3).as_u64(), 0x1006u64);
        assert_eq!(ptr16.offset(-5).as_u64(), 0xFF6u64);

        let ptr32 = Pointer::<u32>::from(0x1000u64);
        assert_eq!(ptr32.offset(3).as_u64(), 0x100Cu64);
        assert_eq!(ptr32.offset(-5).as_u64(), 0xFECu64);

        let ptr64 = Pointer::<u64>::from(0x1000u64);
        assert_eq!(ptr64.offset(3).as_u64(), 0x1018u64);
        assert_eq!(ptr64.offset(-5).as_u64(), 0xFD8u64);
    }

    #[test]
    fn offset_from() {
        let ptr1 = Pointer::<u16>::from(0x1000u64);
        let ptr2 = Pointer::<u16>::from(0x1008u64);

        assert_eq!(ptr2.offset_from(ptr1), 4);
        assert_eq!(ptr1.offset_from(ptr2), -4);
    }
}
