// pointer64.rs
// this file is mostly adapted from https://github.com/CasualX/intptr

use crate::addr::Address;
use crate::error::Result;
use crate::mem::{AccessVirtualMemory, VirtualMemoryContext};

use std::marker::PhantomData;
use std::mem::size_of;
use std::{cmp, fmt, hash, ops};

use dataview::Pod;

#[repr(transparent)]
pub struct Pointer64<T: ?Sized = ()> {
    pub address: u64,
    phantom_data: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Pointer64<T> {
    const PHANTOM_DATA: PhantomData<fn() -> T> = PhantomData;

    pub const NULL: Pointer64<T> = Pointer64 {
        address: 0,
        phantom_data: PhantomData,
    };

    pub fn null() -> Self {
        Pointer64::NULL
    }

    pub fn is_null(self) -> bool {
        self.address == 0
    }

    pub const fn into_raw(self) -> u64 {
        self.address
    }
}

impl<T: Pod + ?Sized> Pointer64<T> {
    pub fn deref_into<U: AccessVirtualMemory>(
        self,
        mem: &mut VirtualMemoryContext<U>,
        out: &mut T,
    ) -> Result<()> {
        mem.virt_read_into(Address::from(self.address), out)
    }
}

impl<T: Pod + Sized> Pointer64<T> {
    pub fn deref<U: AccessVirtualMemory>(self, mem: &mut VirtualMemoryContext<U>) -> Result<T> {
        mem.virt_read(Address::from(self.address))
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
