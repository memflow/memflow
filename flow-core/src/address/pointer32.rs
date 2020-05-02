// pointer32.rs
// this file is mostly adapted from https://github.com/CasualX/intptr

use crate::addr::Address;
use crate::error::Result;
use crate::mem::{AccessVirtualMemory, VirtualMemoryContext};

use std::marker::PhantomData;
use std::mem::size_of;
use std::{cmp, fmt, hash, ops};

use dataview::Pod;

#[repr(transparent)]
pub struct Pointer32<T: ?Sized = ()> {
    pub address: u32,
    phantom_data: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Pointer32<T> {
    const PHANTOM_DATA: PhantomData<fn() -> T> = PhantomData;

    pub const NULL: Pointer32<T> = Pointer32 {
        address: 0,
        phantom_data: PhantomData,
    };

    pub fn null() -> Self {
        Pointer32::NULL
    }

    pub fn is_null(self) -> bool {
        self.address == 0
    }

    pub const fn into_raw(self) -> u32 {
        self.address
    }
}

impl<T: Pod + ?Sized> Pointer32<T> {
    pub fn deref_into<U: AccessVirtualMemory>(
        self,
        mem: &mut VirtualMemoryContext<U>,
        out: &mut T,
    ) -> Result<()> {
        mem.virt_read_into(Address::from(self.address), out)
    }
}

impl<T: Pod + Sized> Pointer32<T> {
    pub fn deref<U: AccessVirtualMemory>(self, mem: &mut VirtualMemoryContext<U>) -> Result<T> {
        mem.virt_read(Address::from(self.address))
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
