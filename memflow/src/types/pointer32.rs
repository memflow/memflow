/*!
32-bit Pointer abstraction.
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
/// use memflow::types::Pointer32;
/// use memflow::mem::VirtualMemory;
/// use dataview::Pod;
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
/// fn read_foo_bar<T: VirtualMemory>(virt_mem: &mut T) {
///     let bar: Bar = virt_mem.virt_read(0x1234.into()).unwrap();
///     let foo = bar.foo_ptr.deref(virt_mem).unwrap();
///     println!("value: {}", foo.some_value);
/// }
///
/// # use memflow::mem::dummy::DummyMemory;
/// # use memflow::types::size;
/// # read_foo_bar(&mut DummyMemory::new_virt(size::mb(4), size::mb(2), &[]).0);
///
/// ```
///
/// ```
/// use memflow::types::Pointer32;
/// use memflow::mem::VirtualMemory;
/// use dataview::Pod;
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
/// fn read_foo_bar<T: VirtualMemory>(virt_mem: &mut T) {
///     let bar: Bar = virt_mem.virt_read(0x1234.into()).unwrap();
///     let foo = virt_mem.virt_read_ptr32(bar.foo_ptr).unwrap();
///     println!("value: {}", foo.some_value);
/// }
///
/// # use memflow::mem::dummy::DummyMemory;
/// # use memflow::types::size;
/// # read_foo_bar(&mut DummyMemory::new_virt(size::mb(4), size::mb(2), &[]).0);
/// ```
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct Pointer32<T: ?Sized = ()> {
    pub address: u32,
    phantom_data: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Pointer32<T> {
    const PHANTOM_DATA: PhantomData<fn() -> T> = PhantomData;

    /// A pointer with a value of zero.
    pub const NULL: Pointer32<T> = Pointer32 {
        address: 0,
        phantom_data: PhantomData,
    };

    /// Returns a pointer with a value of zero.
    pub fn null() -> Self {
        Pointer32::NULL
    }

    /// Checks wether the containing value of this pointer is zero.
    pub fn is_null(self) -> bool {
        self.address == 0
    }

    /// Returns the underlying raw u32 value of this pointer.
    pub const fn into_raw(self) -> u32 {
        self.address
    }
}

/// This function will deref the pointer directly into a Pod type.
impl<T: Pod + ?Sized> Pointer32<T> {
    pub fn deref_into<U: VirtualMemory>(self, mem: &mut U, out: &mut T) -> PartialResult<()> {
        mem.virt_read_ptr32_into(self, out)
    }
}

/// This function will return the Object this pointer is pointing towards.
impl<T: Pod + Sized> Pointer32<T> {
    pub fn deref<U: VirtualMemory>(self, mem: &mut U) -> PartialResult<T> {
        mem.virt_read_ptr32(self)
    }
}

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

impl<T: ?Sized> From<u32> for Pointer32<T> {
    #[inline(always)]
    fn from(address: u32) -> Pointer32<T> {
        Pointer32 {
            address,
            phantom_data: PhantomData,
        }
    }
}
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

impl<T: ?Sized + 'static> ByteSwap for Pointer32<T> {
    fn byte_swap(&mut self) {
        self.address.byte_swap();
    }
}
