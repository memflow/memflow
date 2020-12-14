/*!
64-bit Pointer abstraction.
*/

use crate::error::PartialResult;
use crate::mem::VirtualMemory;
use crate::types::{Address, ByteSwap};

use std::marker::PhantomData;
use std::mem::size_of;
use std::{cmp, fmt, hash, ops};

use dataview::Pod;

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
/// use dataview::Pod;
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
/// fn read_foo_bar<T: VirtualMemory>(virt_mem: &mut T) {
///     let bar: Bar = virt_mem.virt_read(0x1234.into()).unwrap();
///     let foo = bar.foo_ptr.deref(virt_mem).unwrap();
///     println!("value: {}", foo.some_value);
/// }
///
/// # use memflow::mem::dummy::DummyMemory;
/// # use memflow::types::size;
/// # read_foo_bar(&mut DummyMemory::new_virt(size::mb(4), size::mb(2), &[]).0);
/// ```
///
/// ```
/// use memflow::types::Pointer64;
/// use memflow::mem::VirtualMemory;
/// use dataview::Pod;
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
/// fn read_foo_bar<T: VirtualMemory>(virt_mem: &mut T) {
///     let bar: Bar = virt_mem.virt_read(0x1234.into()).unwrap();
///     let foo = virt_mem.virt_read_ptr64(bar.foo_ptr).unwrap();
///     println!("value: {}", foo.some_value);
/// }
///
/// # use memflow::mem::dummy::DummyMemory;
/// # use memflow::types::size;
/// # read_foo_bar(&mut DummyMemory::new_virt(size::mb(4), size::mb(2), &[]).0);
/// ```
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct Pointer64<T: ?Sized = ()> {
    pub address: u64,
    phantom_data: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Pointer64<T> {
    const PHANTOM_DATA: PhantomData<fn() -> T> = PhantomData;

    /// A pointer with a value of zero.
    pub const NULL: Pointer64<T> = Pointer64 {
        address: 0,
        phantom_data: PhantomData,
    };

    /// Returns a pointer with a value of zero.
    pub fn null() -> Self {
        Pointer64::NULL
    }

    /// Checks wether the containing value of this pointer is zero.
    pub fn is_null(self) -> bool {
        self.address == 0
    }

    /// Returns the underlying raw u64 value of this pointer.
    pub const fn into_raw(self) -> u64 {
        self.address
    }
}

/// This function will deref the pointer directly into a Pod type.
impl<T: Pod + ?Sized> Pointer64<T> {
    pub fn deref_into<U: VirtualMemory>(self, mem: &mut U, out: &mut T) -> PartialResult<()> {
        mem.virt_read_ptr64_into(self, out)
    }
}

/// This function will return the Object this pointer is pointing towards.
impl<T: Pod + Sized> Pointer64<T> {
    pub fn deref<U: VirtualMemory>(self, mem: &mut U) -> PartialResult<T> {
        mem.virt_read_ptr64(self)
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

impl<T: ?Sized> From<u64> for Pointer64<T> {
    #[inline(always)]
    fn from(address: u64) -> Pointer64<T> {
        Pointer64 {
            address,
            phantom_data: PhantomData,
        }
    }
}
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

impl<T: ?Sized + 'static> ByteSwap for Pointer64<T> {
    fn byte_swap(&mut self) {
        self.address.byte_swap();
    }
}
