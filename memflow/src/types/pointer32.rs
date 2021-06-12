/*!
32-bit Pointer abstraction.
*/

use crate::dataview::Pod;
use crate::error::{Error, ErrorKind, ErrorOrigin, PartialResult, Result};
use crate::mem::{PhysicalMemory, VirtualMemory};
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
/// use memflow::types::Pointer32;
/// use memflow::mem::VirtualMemory;
/// use memflow::dataview::Pod;
///
/// #[repr(C)]
/// #[derive(Clone, Debug, Pod)]
/// struct Foo {
///     pub some_value: i32,
/// }
///
/// #[repr(C)]
/// #[derive(Clone, Debug, Pod)]
/// struct Bar {
///     pub foo_ptr: Pointer32<Foo>,
/// }
///
/// fn read_foo_bar(virt_mem: &mut impl VirtualMemory) {
///     let bar: Bar = virt_mem.virt_read(0x1234.into()).unwrap();
///     let foo = bar.foo_ptr.virt_read(virt_mem).unwrap();
///     println!("value: {}", foo.some_value);
/// }
///
/// # use memflow::types::size;
/// # use memflow::dummy::DummyOs;
/// # use memflow::os::Process;
/// # read_foo_bar(DummyOs::quick_process(size::mb(2), &[]).virt_mem());
///
/// ```
///
/// ```
/// use memflow::types::Pointer32;
/// use memflow::mem::VirtualMemory;
/// use memflow::dataview::Pod;
///
/// #[repr(C)]
/// #[derive(Clone, Debug, Pod)]
/// struct Foo {
///     pub some_value: i32,
/// }
///
/// #[repr(C)]
/// #[derive(Clone, Debug, Pod)]
/// struct Bar {
///     pub foo_ptr: Pointer32<Foo>,
/// }
///
/// fn read_foo_bar(virt_mem: &mut impl VirtualMemory) {
///     let bar: Bar = virt_mem.virt_read(0x1234.into()).unwrap();
///     let foo = virt_mem.virt_read_ptr32(bar.foo_ptr).unwrap();
///     println!("value: {}", foo.some_value);
/// }
///
/// # use memflow::types::size;
/// # use memflow::dummy::DummyOs;
/// # use memflow::os::Process;
/// # read_foo_bar(DummyOs::quick_process(size::mb(2), &[]).virt_mem());
/// ```
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct Pointer32<T: ?Sized = ()> {
    pub address: u32,
    phantom_data: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Pointer32<T> {
    const PHANTOM_DATA: PhantomData<fn() -> T> = PhantomData;

    /// A pointer32 with the value of zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer32;
    ///
    /// println!("pointer32: {}", Pointer32::<()>::NULL);
    /// ```
    pub const NULL: Pointer32<T> = Pointer32 {
        address: 0,
        phantom_data: PhantomData,
    };

    /// Returns a pointer32 with a value of zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer32;
    ///
    /// println!("pointer32: {}", Pointer32::<()>::null());
    /// ```
    #[inline]
    pub const fn null() -> Self {
        Pointer32::NULL
    }

    /// Returns `true` if the pointer is null.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer32;
    ///
    /// let ptr = Pointer32::<()>::from(0x1000u32);
    /// assert!(!ptr.is_null());
    /// ```
    #[inline]
    pub const fn is_null(self) -> bool {
        self.address == 0
    }

    /// Converts the pointer32 to an Option that is None when it is null
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer32;
    ///
    /// assert_eq!(Pointer32::<()>::null().non_null(), None);
    /// assert_eq!(Pointer32::<()>::from(0x1000u32).non_null(), Some(Pointer32::from(0x1000)));
    /// ```
    #[inline]
    pub fn non_null(self) -> Option<Pointer32<T>> {
        if self.is_null() {
            None
        } else {
            Some(self)
        }
    }

    /// Converts the pointer32 into a `u32` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer32;
    ///
    /// let ptr = Pointer32::<()>::from(0x1000u32);
    /// let ptr_u32: u32 = ptr.as_u32();
    /// assert_eq!(ptr_u32, 0x1000);
    /// ```
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.address
    }

    /// Converts the pointer32 into a `u64` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer32;
    ///
    /// let ptr = Pointer32::<()>::from(0x1000u32);
    /// let ptr_u64: u64 = ptr.as_u64();
    /// assert_eq!(ptr_u64, 0x1000);
    /// ```
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.address as u64
    }

    /// Converts the pointer32 into a `usize` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer32;
    ///
    /// let ptr = Pointer32::<()>::from(0x1000u32);
    /// let ptr_usize: usize = ptr.as_usize();
    /// assert_eq!(ptr_usize, 0x1000);
    /// ```
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.address as usize
    }
}

impl<T: Sized> Pointer32<T> {
    /// Calculates the offset from a pointer
    ///
    /// `count` is in units of T; e.g., a `count` of 3 represents a pointer offset of `3 * size_of::<T>()` bytes.
    ///
    /// # Panics
    ///
    /// This function panics if `T` is a Zero-Sized Type ("ZST").
    /// This function also panics when `offset * size_of::<T>()`
    /// causes overflow of a signed 32-bit integer.
    ///
    /// # Examples:
    ///
    /// ```
    /// use memflow::types::Pointer32;
    ///
    /// let ptr = Pointer32::<u16>::from(0x1000u32);
    ///
    /// println!("{:?}", ptr.offset(3));
    /// ```
    pub fn offset(self, count: i32) -> Self {
        let pointee_size = size_of::<T>();
        assert!(0 < pointee_size && pointee_size <= i32::MAX as usize);

        (self
            .address
            .wrapping_add((pointee_size as i32 * count) as u32))
        .into()
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
    /// use memflow::types::Pointer32;
    ///
    /// let ptr1 = Pointer32::<u16>::from(0x1000u32);
    /// let ptr2 = Pointer32::<u16>::from(0x1008u32);
    ///
    /// assert_eq!(ptr2.offset_from(ptr1), 4);
    /// assert_eq!(ptr1.offset_from(ptr2), -4);
    /// ```
    pub fn offset_from(self, origin: Self) -> i32 {
        let pointee_size = size_of::<T>();
        assert!(0 < pointee_size && pointee_size <= i32::MAX as usize);

        let offset = self.address.wrapping_sub(origin.address) as i32;
        offset / pointee_size as i32
    }
}

/// Implement special phys/virt read/write for Pod types
impl<T: Pod + ?Sized> Pointer32<T> {
    pub fn phys_read_into<U: PhysicalMemory>(self, mem: &mut U, out: &mut T) -> Result<()> {
        mem.phys_read_ptr32_into(self, out)
    }

    pub fn virt_read_into<U: VirtualMemory>(self, mem: &mut U, out: &mut T) -> PartialResult<()> {
        mem.virt_read_ptr32_into(self, out)
    }
}

impl<T: Pod + Sized> Pointer32<T> {
    pub fn phys_read<U: PhysicalMemory>(self, mem: &mut U) -> Result<T> {
        mem.phys_read_ptr32(self)
    }

    pub fn virt_read<U: VirtualMemory>(self, mem: &mut U) -> PartialResult<T> {
        mem.virt_read_ptr32(self)
    }

    pub fn phys_write<U: PhysicalMemory>(self, mem: &mut U, data: &T) -> Result<()> {
        mem.phys_write_ptr32(self, data)
    }

    pub fn virt_write<U: VirtualMemory>(self, mem: &mut U, data: &T) -> PartialResult<()> {
        mem.virt_write_ptr32(self, data)
    }
}

/// Implement special phys/virt read/write for CReprStr
// TODO: fixme
/*
impl Pointer32<ReprCString> {
    pub fn phys_read<U: PhysicalMemory>(self, mem: &mut U) -> Result<ReprCString> {
        match mem.phys_read_char_string(self.address.into()) {
            Ok(s) => Ok(s.into()),
            Err(e) => Err(e),
        }
    }

    pub fn virt_read<U: VirtualMemory>(self, mem: &mut U) -> PartialResult<ReprCString> {
        match mem.virt_read_char_string(self.address.into()) {
            Ok(s) => Ok(s.into()),
            Err(PartialError::Error(e)) => Err(PartialError::Error(e)),
            Err(PartialError::PartialVirtualRead(s)) => {
                Err(PartialError::PartialVirtualRead(s.into()))
            }
            Err(PartialError::PartialVirtualWrite) => Err(PartialError::PartialVirtualWrite),
        }
    }
}
*/

impl<T> Pointer32<[T]> {
    pub const fn decay(self) -> Pointer32<T> {
        Pointer32 {
            address: self.address,
            phantom_data: Pointer32::<T>::PHANTOM_DATA,
        }
    }

    pub const fn at(self, i: usize) -> Pointer32<T> {
        let address = self.address + (i * size_of::<T>()) as u32;
        Pointer32 {
            address,
            phantom_data: Pointer32::<T>::PHANTOM_DATA,
        }
    }
}

impl<T: ?Sized> Copy for Pointer32<T> {}
impl<T: ?Sized> Clone for Pointer32<T> {
    #[inline(always)]
    fn clone(&self) -> Pointer32<T> {
        *self
    }
}
impl<T: ?Sized> Default for Pointer32<T> {
    #[inline(always)]
    fn default() -> Pointer32<T> {
        Pointer32::NULL
    }
}
impl<T: ?Sized> Eq for Pointer32<T> {}
impl<T: ?Sized> PartialEq for Pointer32<T> {
    #[inline(always)]
    fn eq(&self, rhs: &Pointer32<T>) -> bool {
        self.address == rhs.address
    }
}
impl<T: ?Sized> PartialOrd for Pointer32<T> {
    #[inline(always)]
    fn partial_cmp(&self, rhs: &Pointer32<T>) -> Option<cmp::Ordering> {
        self.address.partial_cmp(&rhs.address)
    }
}
impl<T: ?Sized> Ord for Pointer32<T> {
    #[inline(always)]
    fn cmp(&self, rhs: &Pointer32<T>) -> cmp::Ordering {
        self.address.cmp(&rhs.address)
    }
}
impl<T: ?Sized> hash::Hash for Pointer32<T> {
    #[inline(always)]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.address.hash(state)
    }
}
impl<T: ?Sized> AsRef<u32> for Pointer32<T> {
    #[inline(always)]
    fn as_ref(&self) -> &u32 {
        &self.address
    }
}
impl<T: ?Sized> AsMut<u32> for Pointer32<T> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut u32 {
        &mut self.address
    }
}

// From implementations
impl<T: ?Sized> From<u32> for Pointer32<T> {
    #[inline(always)]
    fn from(address: u32) -> Pointer32<T> {
        Pointer32 {
            address,
            phantom_data: PhantomData,
        }
    }
}

/// Tries to converts an u64 into a Pointer32.
/// The function will return an `Error::Bounds` error if the input value is greater than `u32::max_value()`.
impl<T: ?Sized> TryFrom<u64> for Pointer32<T> {
    type Error = crate::error::Error;

    fn try_from(address: u64) -> std::result::Result<Pointer32<T>, Self::Error> {
        if address <= (u32::max_value() as u64) {
            Ok(Pointer32 {
                address: address as u32,
                phantom_data: PhantomData,
            })
        } else {
            Err(Error(ErrorOrigin::Pointer, ErrorKind::OutOfBounds))
        }
    }
}

/// Tries to converts an Address into a Pointer32.
/// The function will return an Error::Bounds if the input value is greater than `u32::max_value()`.
impl<T: ?Sized> TryFrom<Address> for Pointer32<T> {
    type Error = crate::error::Error;

    fn try_from(address: Address) -> std::result::Result<Pointer32<T>, Self::Error> {
        if address.as_u64() <= (u32::max_value() as u64) {
            Ok(Pointer32 {
                address: address.as_u32(),
                phantom_data: PhantomData,
            })
        } else {
            Err(Error(ErrorOrigin::Pointer, ErrorKind::OutOfBounds))
        }
    }
}

// Into implementations
impl<T: ?Sized> From<Pointer32<T>> for Address {
    #[inline(always)]
    fn from(ptr: Pointer32<T>) -> Address {
        ptr.address.into()
    }
}

impl<T: ?Sized> From<Pointer32<T>> for u32 {
    #[inline(always)]
    fn from(ptr: Pointer32<T>) -> u32 {
        ptr.address
    }
}

impl<T: ?Sized> From<Pointer32<T>> for u64 {
    #[inline(always)]
    fn from(ptr: Pointer32<T>) -> u64 {
        ptr.address as u64
    }
}

// Arithmetic operations
impl<T> ops::Add<usize> for Pointer32<T> {
    type Output = Pointer32<T>;
    #[inline(always)]
    fn add(self, other: usize) -> Pointer32<T> {
        let address = self.address + (other * size_of::<T>()) as u32;
        Pointer32 {
            address,
            phantom_data: self.phantom_data,
        }
    }
}
impl<T> ops::Sub<usize> for Pointer32<T> {
    type Output = Pointer32<T>;
    #[inline(always)]
    fn sub(self, other: usize) -> Pointer32<T> {
        let address = self.address - (other * size_of::<T>()) as u32;
        Pointer32 {
            address,
            phantom_data: self.phantom_data,
        }
    }
}

impl<T: ?Sized> fmt::Debug for Pointer32<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.address)
    }
}
impl<T: ?Sized> fmt::UpperHex for Pointer32<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}", self.address)
    }
}
impl<T: ?Sized> fmt::LowerHex for Pointer32<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.address)
    }
}
impl<T: ?Sized> fmt::Display for Pointer32<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.address)
    }
}

unsafe impl<T: ?Sized + 'static> Pod for Pointer32<T> {}
const _: [(); std::mem::size_of::<Pointer32<()>>()] = [(); std::mem::size_of::<u32>()];

impl<T: ?Sized + 'static> ByteSwap for Pointer32<T> {
    fn byte_swap(&mut self) {
        self.address.byte_swap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset() {
        let ptr8 = Pointer32::<u8>::from(0x1000u32);
        assert_eq!(ptr8.offset(3).as_u32(), 0x1003u32);
        assert_eq!(ptr8.offset(-5).as_u32(), 0xFFBu32);

        let ptr16 = Pointer32::<u16>::from(0x1000u32);
        assert_eq!(ptr16.offset(3).as_u32(), 0x1006u32);
        assert_eq!(ptr16.offset(-5).as_u32(), 0xFF6u32);

        let ptr32 = Pointer32::<u32>::from(0x1000u32);
        assert_eq!(ptr32.offset(3).as_u32(), 0x100Cu32);
        assert_eq!(ptr32.offset(-5).as_u32(), 0xFECu32);
    }

    #[test]
    fn offset_from() {
        let ptr1 = Pointer32::<u16>::from(0x1000u32);
        let ptr2 = Pointer32::<u16>::from(0x1008u32);

        assert_eq!(ptr2.offset_from(ptr1), 4);
        assert_eq!(ptr1.offset_from(ptr2), -4);
    }
}
