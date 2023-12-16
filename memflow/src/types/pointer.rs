/*!
Pointer abstraction.
*/

use crate::cglue::ReprCString;
use crate::dataview::Pod;
use crate::error::{PartialError, PartialResult};
use crate::mem::MemoryView;
use crate::types::{imem, umem, Address, ByteSwap, PrimitiveAddress};

use std::convert::TryInto;
use std::marker::PhantomData;
use std::mem::size_of;
use std::{cmp, fmt, hash, ops};

pub type Pointer32<T> = Pointer<u32, T>;
pub type Pointer64<T> = Pointer<u64, T>;

const _: [(); std::mem::size_of::<Pointer32<()>>()] = [(); std::mem::size_of::<u32>()];
const _: [(); std::mem::size_of::<Pointer64<()>>()] = [(); std::mem::size_of::<u64>()];

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
///     pub foo_ptr: Pointer64<Foo>,
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
/// use memflow::types::Pointer64;
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
///     pub foo_ptr: Pointer64<Foo>,
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
pub struct Pointer<U: Sized, T: ?Sized = ()> {
    pub inner: U,
    phantom_data: PhantomData<fn() -> T>,
}
unsafe impl<U: Pod, T: ?Sized + 'static> Pod for Pointer<U, T> {}

impl<U: PrimitiveAddress, T: ?Sized> Pointer<U, T> {
    const PHANTOM_DATA: PhantomData<fn() -> T> = PhantomData;

    /// Returns a pointer64 with a value of zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer64;
    ///
    /// println!("pointer: {}", Pointer64::<()>::null());
    /// ```
    #[inline]
    pub fn null() -> Self {
        Pointer {
            inner: U::null(),
            phantom_data: PhantomData,
        }
    }

    /// Returns `true` if the pointer64 is null.
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
    pub fn is_null(self) -> bool {
        self.inner.is_null()
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
    pub fn non_null(self) -> Option<Pointer<U, T>> {
        if self.is_null() {
            None
        } else {
            Some(self)
        }
    }

    /// Converts the pointer into a raw `umem` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::{Pointer64, umem};
    ///
    /// let ptr = Pointer64::<()>::from(0x1000u64);
    /// let ptr_umem: umem = ptr.to_umem();
    /// assert_eq!(ptr_umem, 0x1000);
    /// ```
    #[inline]
    pub fn to_umem(self) -> umem {
        self.inner.to_umem()
    }

    // Returns the address this pointer holds.
    #[inline]
    pub fn address(&self) -> Address {
        Address::from(self.inner)
    }
}

impl<U: PrimitiveAddress, T: Sized> Pointer<U, T> {
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
    /// use memflow::types::Pointer64;
    ///
    /// let ptr = Pointer64::<u16>::from(0x1000u64);
    ///
    /// println!("{:?}", ptr.offset(3));
    /// ```
    pub fn offset(self, count: imem) -> Self {
        let pointee_size = U::from_umem(size_of::<T>() as umem);
        assert!(U::null() < pointee_size && pointee_size <= PrimitiveAddress::max());

        if count >= 0 {
            self.inner
                .wrapping_add(U::from_umem(pointee_size.to_umem() * count as umem))
                .into()
        } else {
            self.inner
                .wrapping_sub(U::from_umem(pointee_size.to_umem() * (-count) as umem))
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
    /// use memflow::types::Pointer64;
    ///
    /// let ptr1 = Pointer64::<u16>::from(0x1000u64);
    /// let ptr2 = Pointer64::<u16>::from(0x1008u64);
    ///
    /// assert_eq!(ptr2.offset_from(ptr1), 4);
    /// assert_eq!(ptr1.offset_from(ptr2), -4);
    /// ```
    pub fn offset_from(self, origin: Self) -> imem {
        let pointee_size: imem = size_of::<T>().try_into().unwrap();
        let offset = self.inner.to_imem().wrapping_sub(origin.inner.to_imem());
        offset / pointee_size as imem
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
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, count: umem) -> Self {
        self.offset(count as imem)
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
    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, count: umem) -> Self {
        self.offset((count as imem).wrapping_neg())
    }
}

/// Implement special phys/virt read/write for Pod types
impl<U: PrimitiveAddress, T: Pod + ?Sized> Pointer<U, T> {
    pub fn read_into<M: MemoryView>(self, mem: &mut M, out: &mut T) -> PartialResult<()> {
        mem.read_ptr_into(self, out)
    }
}

impl<U: PrimitiveAddress, T: Pod + Sized> Pointer<U, T> {
    pub fn read<M: MemoryView>(self, mem: &mut M) -> PartialResult<T> {
        mem.read_ptr(self)
    }

    pub fn write<M: MemoryView>(self, mem: &mut M, data: &T) -> PartialResult<()> {
        mem.write_ptr(self, data)
    }
}

/// Implement special phys/virt read/write for CReprStr
impl<U: PrimitiveAddress> Pointer<U, ReprCString> {
    pub fn read_string<M: MemoryView>(self, mem: &mut M) -> PartialResult<ReprCString> {
        match mem.read_char_string(self.inner.to_umem().into()) {
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

impl<U: PrimitiveAddress, T> Pointer<U, [T]> {
    pub fn decay(self) -> Pointer<U, T> {
        Pointer {
            inner: self.inner,
            phantom_data: Pointer::<U, T>::PHANTOM_DATA,
        }
    }

    pub fn at(self, i: umem) -> Pointer<U, T> {
        let inner = self
            .inner
            .wrapping_add(U::from_umem(size_of::<T>() as umem * i));
        Pointer {
            inner,
            phantom_data: Pointer::<U, T>::PHANTOM_DATA,
        }
    }
}

impl<U: PrimitiveAddress, T: ?Sized> Copy for Pointer<U, T> {}
impl<U: PrimitiveAddress, T: ?Sized> Clone for Pointer<U, T> {
    #[inline(always)]
    fn clone(&self) -> Pointer<U, T> {
        *self
    }
}
impl<U: PrimitiveAddress, T: ?Sized> Default for Pointer<U, T> {
    #[inline(always)]
    fn default() -> Pointer<U, T> {
        Pointer::null()
    }
}
impl<U: PrimitiveAddress, T: ?Sized> Eq for Pointer<U, T> {}
impl<U: PrimitiveAddress, T: ?Sized> PartialEq for Pointer<U, T> {
    #[inline(always)]
    fn eq(&self, rhs: &Pointer<U, T>) -> bool {
        self.inner == rhs.inner
    }
}
impl<U: PrimitiveAddress, T: ?Sized> PartialOrd for Pointer<U, T> {
    #[inline(always)]
    fn partial_cmp(&self, rhs: &Pointer<U, T>) -> Option<cmp::Ordering> {
        Some(self.cmp(rhs))
    }
}
impl<U: PrimitiveAddress, T: ?Sized> Ord for Pointer<U, T> {
    #[inline(always)]
    fn cmp(&self, rhs: &Pointer<U, T>) -> cmp::Ordering {
        self.inner.cmp(&rhs.inner)
    }
}
impl<U: PrimitiveAddress, T: ?Sized> hash::Hash for Pointer<U, T> {
    #[inline(always)]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}
impl<U: PrimitiveAddress, T: ?Sized> AsRef<U> for Pointer<U, T> {
    #[inline(always)]
    fn as_ref(&self) -> &U {
        &self.inner
    }
}
impl<U: PrimitiveAddress, T: ?Sized> AsMut<U> for Pointer<U, T> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut U {
        &mut self.inner
    }
}

// From implementations
impl<U: PrimitiveAddress, T: ?Sized> From<U> for Pointer<U, T> {
    #[inline(always)]
    fn from(address: U) -> Pointer<U, T> {
        Pointer {
            inner: address,
            phantom_data: PhantomData,
        }
    }
}

impl<T: ?Sized> From<Address> for Pointer64<T> {
    #[inline(always)]
    fn from(address: Address) -> Pointer64<T> {
        Pointer {
            inner: address.to_umem() as u64,
            phantom_data: PhantomData,
        }
    }
}

// Into implementations
impl<U: Into<Address>, T: ?Sized> From<Pointer<U, T>> for umem {
    #[inline(always)]
    fn from(ptr: Pointer<U, T>) -> umem {
        let address: Address = ptr.inner.into();
        address.to_umem()
    }
}

// Arithmetic operations
impl<U: PrimitiveAddress, T> ops::Add<umem> for Pointer<U, T> {
    type Output = Pointer<U, T>;
    #[inline(always)]
    fn add(self, other: umem) -> Pointer<U, T> {
        let address = self.inner + U::from_umem(size_of::<T>() as umem * other);
        Pointer {
            inner: address,
            phantom_data: self.phantom_data,
        }
    }
}
impl<U: PrimitiveAddress, T> ops::Sub<umem> for Pointer<U, T> {
    type Output = Pointer<U, T>;
    #[inline(always)]
    fn sub(self, other: umem) -> Pointer<U, T> {
        let address = self.inner - U::from_umem(size_of::<T>() as umem * other);
        Pointer {
            inner: address,
            phantom_data: self.phantom_data,
        }
    }
}

#[cfg(feature = "64_bit_mem")]
impl<U: PrimitiveAddress, T> ops::Add<usize> for Pointer<U, T> {
    type Output = Pointer<U, T>;
    #[inline(always)]
    fn add(self, other: usize) -> Pointer<U, T> {
        self + other as umem
    }
}
#[cfg(feature = "64_bit_mem")]
impl<U: PrimitiveAddress, T> ops::Sub<usize> for Pointer<U, T> {
    type Output = Pointer<U, T>;
    #[inline(always)]
    fn sub(self, other: usize) -> Pointer<U, T> {
        self - other as umem
    }
}

impl<U: PrimitiveAddress, T: ?Sized> fmt::Debug for Pointer<U, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.inner)
    }
}
impl<U: PrimitiveAddress, T: ?Sized> fmt::UpperHex for Pointer<U, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}", self.inner)
    }
}
impl<U: PrimitiveAddress, T: ?Sized> fmt::LowerHex for Pointer<U, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.inner)
    }
}
impl<U: PrimitiveAddress, T: ?Sized> fmt::Display for Pointer<U, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.inner)
    }
}

impl<U: PrimitiveAddress, T: ?Sized + 'static> ByteSwap for Pointer<U, T> {
    fn byte_swap(&mut self) {
        self.inner.byte_swap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset32() {
        let ptr8 = Pointer32::<u8>::from(0x1000u32);
        assert_eq!(ptr8.offset(3).to_umem(), 0x1003);
        assert_eq!(ptr8.offset(-5).to_umem(), 0xFFB);

        let ptr16 = Pointer32::<u16>::from(0x1000u32);
        assert_eq!(ptr16.offset(3).to_umem(), 0x1006);
        assert_eq!(ptr16.offset(-5).to_umem(), 0xFF6);

        let ptr32 = Pointer32::<u32>::from(0x1000u32);
        assert_eq!(ptr32.offset(3).to_umem(), 0x100C);
        assert_eq!(ptr32.offset(-5).to_umem(), 0xFEC);
    }

    #[test]
    fn offset64() {
        let ptr8 = Pointer64::<u8>::from(0x1000u64);
        assert_eq!(ptr8.offset(3).to_umem(), 0x1003);
        assert_eq!(ptr8.offset(-5).to_umem(), 0xFFB);

        let ptr16 = Pointer64::<u16>::from(0x1000u64);
        assert_eq!(ptr16.offset(3).to_umem(), 0x1006);
        assert_eq!(ptr16.offset(-5).to_umem(), 0xFF6);

        let ptr32 = Pointer64::<u32>::from(0x1000u64);
        assert_eq!(ptr32.offset(3).to_umem(), 0x100C);
        assert_eq!(ptr32.offset(-5).to_umem(), 0xFEC);

        let ptr64 = Pointer64::<u64>::from(0x1000u64);
        assert_eq!(ptr64.offset(3).to_umem(), 0x1018);
        assert_eq!(ptr64.offset(-5).to_umem(), 0xFD8);
    }

    #[test]
    fn offset_from() {
        let ptr1 = Pointer64::<u16>::from(0x1000u64);
        let ptr2 = Pointer64::<u16>::from(0x1008u64);

        assert_eq!(ptr2.offset_from(ptr1), 4);
        assert_eq!(ptr1.offset_from(ptr2), -4);
    }
}
