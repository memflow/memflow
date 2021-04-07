/*!
64-bit Pointer abstraction.
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
/// use memflow::types::Pointer64;
/// use memflow::mem::VirtualMemory;
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
///     pub foo_ptr: Pointer64<Foo>,
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
/// ```
///
/// ```
/// use memflow::types::Pointer64;
/// use memflow::mem::VirtualMemory;
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
///     pub foo_ptr: Pointer64<Foo>,
/// }
///
/// fn read_foo_bar(virt_mem: &mut impl VirtualMemory) {
///     let bar: Bar = virt_mem.virt_read(0x1234.into()).unwrap();
///     let foo = virt_mem.virt_read_ptr64(bar.foo_ptr).unwrap();
///     println!("value: {}", foo.some_value);
/// }
///
/// # use memflow::dummy::DummyOs;
/// # use memflow::os::Process;
/// # use memflow::types::size;
/// # read_foo_bar(DummyOs::quick_process(size::mb(2), &[]).virt_mem());
/// ```
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct Pointer64<T: ?Sized = ()> {
    pub address: u64,
    phantom_data: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Pointer64<T> {
    const PHANTOM_DATA: PhantomData<fn() -> T> = PhantomData;

    /// A pointer64 with the value of zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer64;
    ///
    /// println!("pointer64: {}", Pointer64::<()>::NULL);
    /// ```
    pub const NULL: Pointer64<T> = Pointer64 {
        address: 0,
        phantom_data: PhantomData,
    };

    /// Returns a pointer64 with a value of zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer64;
    ///
    /// println!("pointer64: {}", Pointer64::<()>::null());
    /// ```
    pub const fn null() -> Self {
        Pointer64::NULL
    }

    /// Returns `true` if the pointer64 is null.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer64;
    ///
    /// let ptr = Pointer64::<()>::from(0x1000u32);
    /// assert!(!ptr.is_null());
    /// ```
    pub const fn is_null(self) -> bool {
        self.address == 0
    }

    /// Converts the pointer64 to an Option that is None when it is null
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer64;
    ///
    /// assert_eq!(Pointer64::<()>::null().non_null(), None);
    /// assert_eq!(Pointer64::<()>::from(0x1000u64).non_null(), Some(Pointer64::from(0x1000u64)));
    /// ```
    #[inline]
    pub fn non_null(self) -> Option<Pointer64<T>> {
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
    /// use memflow::types::Pointer64;
    ///
    /// let ptr = Pointer64::<()>::from(0x1000u64);
    /// let ptr_u32: u32 = ptr.as_u32();
    /// assert_eq!(ptr_u32, 0x1000);
    /// ```
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.address as u32
    }

    /// Converts the pointer64 into a `u64` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer64;
    ///
    /// let ptr = Pointer64::<()>::from(0x1000u64);
    /// let ptr_u64: u64 = ptr.as_u64();
    /// assert_eq!(ptr_u64, 0x1000);
    /// ```
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.address as u64
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
    /// use memflow::types::Pointer64;
    ///
    /// let ptr = Pointer64::<()>::from(0x1000u64);
    /// let ptr_usize: usize = ptr.as_usize();
    /// assert_eq!(ptr_usize, 0x1000);
    /// ```
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.address as usize
    }
}

impl<T: Sized> Pointer64<T> {
    /// Calculates the offset from a pointer64
    ///
    /// `count` is in units of T; e.g., a `count` of 3 represents a pointer offset of `3 * size_of::<T>()` bytes.
    ///
    /// # Panics
    ///
    /// This function panics if `T` is a Zero-Sized Type ("ZST").
    ///
    /// # Examples:
    ///
    /// ```
    /// use memflow::types::Pointer64;
    ///
    /// let ptr = Pointer64::<u16>::from(0x1000u64);
    ///
    /// println!("{:?}", ptr.offset(3));
    /// ```
    pub fn offset(self, count: i64) -> Self {
        let pointee_size = size_of::<T>();
        assert!(0 < pointee_size && pointee_size <= i64::MAX as usize);

        (self
            .address
            .wrapping_add((pointee_size as i64 * count) as u64))
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
    /// use memflow::types::Pointer64;
    ///
    /// let ptr1 = Pointer64::<u16>::from(0x1000u64);
    /// let ptr2 = Pointer64::<u16>::from(0x1008u64);
    ///
    /// assert_eq!(ptr2.offset_from(ptr1), 4);
    /// assert_eq!(ptr1.offset_from(ptr2), -4);
    /// ```
    pub fn offset_from(self, origin: Self) -> i64 {
        let pointee_size = size_of::<T>();
        assert!(0 < pointee_size && pointee_size <= i64::MAX as usize);

        let offset = self.address.wrapping_sub(origin.address) as i64;
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
    /// use memflow::types::Pointer64;
    ///
    /// let ptr = Pointer64::<u16>::from(0x1000u64);
    ///
    /// println!("{:?}", ptr.add(3));
    /// ```
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
    /// use memflow::types::Pointer64;
    ///
    /// let ptr = Pointer64::<u16>::from(0x1000u64);
    ///
    /// println!("{:?}", ptr.sub(3));
    /// ```
    pub fn sub(self, count: u64) -> Self {
        self.offset((count as i64).wrapping_neg())
    }
}

/// This function will deref the pointer directly into a Pod type.
impl<T: Pod + ?Sized> Pointer64<T> {
    pub fn phys_read_into<U: PhysicalMemory>(self, mem: &mut U, out: &mut T) -> Result<()> {
        mem.phys_read_ptr64_into(self, out)
    }

    pub fn virt_read_into<U: VirtualMemory>(self, mem: &mut U, out: &mut T) -> PartialResult<()> {
        mem.virt_read_ptr64_into(self, out)
    }
}

/// This function will return the Object this pointer is pointing towards.
impl<T: Pod + Sized> Pointer64<T> {
    pub fn phys_read<U: PhysicalMemory>(self, mem: &mut U) -> Result<T> {
        mem.phys_read_ptr64(self)
    }

    pub fn virt_read<U: VirtualMemory>(self, mem: &mut U) -> PartialResult<T> {
        mem.virt_read_ptr64(self)
    }

    pub fn phys_write<U: PhysicalMemory>(self, mem: &mut U, data: &T) -> Result<()> {
        mem.phys_write_ptr64(self, data)
    }

    pub fn virt_write<U: VirtualMemory>(self, mem: &mut U, data: &T) -> PartialResult<()> {
        mem.virt_write_ptr64(self, data)
    }
}

impl<T> Pointer64<[T]> {
    pub const fn decay(self) -> Pointer64<T> {
        Pointer64 {
            address: self.address,
            phantom_data: Pointer64::<T>::PHANTOM_DATA,
        }
    }

    pub const fn at(self, i: usize) -> Pointer64<T> {
        let address = self.address + (i * size_of::<T>()) as u64;
        Pointer64 {
            address,
            phantom_data: Pointer64::<T>::PHANTOM_DATA,
        }
    }
}

impl<T: ?Sized> Copy for Pointer64<T> {}
impl<T: ?Sized> Clone for Pointer64<T> {
    #[inline(always)]
    fn clone(&self) -> Pointer64<T> {
        *self
    }
}
impl<T: ?Sized> Default for Pointer64<T> {
    #[inline(always)]
    fn default() -> Pointer64<T> {
        Pointer64::NULL
    }
}
impl<T: ?Sized> Eq for Pointer64<T> {}
impl<T: ?Sized> PartialEq for Pointer64<T> {
    #[inline(always)]
    fn eq(&self, rhs: &Pointer64<T>) -> bool {
        self.address == rhs.address
    }
}
impl<T: ?Sized> PartialOrd for Pointer64<T> {
    #[inline(always)]
    fn partial_cmp(&self, rhs: &Pointer64<T>) -> Option<cmp::Ordering> {
        self.address.partial_cmp(&rhs.address)
    }
}
impl<T: ?Sized> Ord for Pointer64<T> {
    #[inline(always)]
    fn cmp(&self, rhs: &Pointer64<T>) -> cmp::Ordering {
        self.address.cmp(&rhs.address)
    }
}
impl<T: ?Sized> hash::Hash for Pointer64<T> {
    #[inline(always)]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.address.hash(state)
    }
}
impl<T: ?Sized> AsRef<u64> for Pointer64<T> {
    #[inline(always)]
    fn as_ref(&self) -> &u64 {
        &self.address
    }
}
impl<T: ?Sized> AsMut<u64> for Pointer64<T> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut u64 {
        &mut self.address
    }
}

// From implementations
impl<T: ?Sized> From<u32> for Pointer64<T> {
    #[inline(always)]
    fn from(address: u32) -> Pointer64<T> {
        Pointer64 {
            address: address as u64,
            phantom_data: PhantomData,
        }
    }
}

impl<T: ?Sized> From<u64> for Pointer64<T> {
    #[inline(always)]
    fn from(address: u64) -> Pointer64<T> {
        Pointer64 {
            address,
            phantom_data: PhantomData,
        }
    }
}

impl<T: ?Sized> From<Address> for Pointer64<T> {
    #[inline(always)]
    fn from(address: Address) -> Pointer64<T> {
        Pointer64 {
            address: address.as_u64(),
            phantom_data: PhantomData,
        }
    }
}

// Into implementations
impl<T: ?Sized> From<Pointer64<T>> for Address {
    #[inline(always)]
    fn from(ptr: Pointer64<T>) -> Address {
        ptr.address.into()
    }
}

impl<T: ?Sized> From<Pointer64<T>> for u64 {
    #[inline(always)]
    fn from(ptr: Pointer64<T>) -> u64 {
        ptr.address
    }
}

/// Tries to convert a Pointer64 into a u32.
/// The function will return an `Error::Bounds` error if the input value is greater than `u32::max_value()`.
impl<T: ?Sized> TryFrom<Pointer64<T>> for u32 {
    type Error = crate::error::Error;

    fn try_from(ptr: Pointer64<T>) -> std::result::Result<u32, Self::Error> {
        if ptr.address <= (u32::max_value() as u64) {
            Ok(ptr.address as u32)
        } else {
            Err(Error(ErrorOrigin::Pointer, ErrorKind::OutOfBounds))
        }
    }
}

// Arithmetic operations
impl<T> ops::Add<usize> for Pointer64<T> {
    type Output = Pointer64<T>;
    #[inline(always)]
    fn add(self, other: usize) -> Pointer64<T> {
        let address = self.address + (other * size_of::<T>()) as u64;
        Pointer64 {
            address,
            phantom_data: self.phantom_data,
        }
    }
}
impl<T> ops::Sub<usize> for Pointer64<T> {
    type Output = Pointer64<T>;
    #[inline(always)]
    fn sub(self, other: usize) -> Pointer64<T> {
        let address = self.address - (other * size_of::<T>()) as u64;
        Pointer64 {
            address,
            phantom_data: self.phantom_data,
        }
    }
}

impl<T: ?Sized> fmt::Debug for Pointer64<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.address)
    }
}
impl<T: ?Sized> fmt::UpperHex for Pointer64<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}", self.address)
    }
}
impl<T: ?Sized> fmt::LowerHex for Pointer64<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.address)
    }
}
impl<T: ?Sized> fmt::Display for Pointer64<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.address)
    }
}

unsafe impl<T: ?Sized + 'static> Pod for Pointer64<T> {}
const _: [(); std::mem::size_of::<Pointer64<()>>()] = [(); std::mem::size_of::<u64>()];

impl<T: ?Sized + 'static> ByteSwap for Pointer64<T> {
    fn byte_swap(&mut self) {
        self.address.byte_swap();
    }
}
