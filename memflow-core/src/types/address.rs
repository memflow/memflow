/*!
Abstraction over a address on the target system.
*/

use std::default::Default;
use std::fmt;
use std::ops;

/**
This type represents a address on the target system.
It internally holds a `u64` value but can also be used
when working in 32-bit environments.

This type will not handle overflow for 32-bit or 64-bit addresses / lengths.
*/
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(u64);

/// Constructs an `Address` from a `i32` value.
impl From<i32> for Address {
    fn from(item: i32) -> Self {
        Self { 0: item as u64 }
    }
}

/// Constructs an `Address` from a `u32` value.
impl From<u32> for Address {
    fn from(item: u32) -> Self {
        Self { 0: u64::from(item) }
    }
}

/// Constructs an `Address` from a `u64` value.
impl From<u64> for Address {
    fn from(item: u64) -> Self {
        Self { 0: item }
    }
}

/// Constructs an `Address` from a `usize` value.
impl From<usize> for Address {
    fn from(item: usize) -> Self {
        Self { 0: item as u64 }
    }
}

impl Address {
    /**
    A address with the value of zero.

    # Examples

    ```
    use memflow_core::types::Address;

    fn test() {
        println!("address: {}", Address::NULL);
    }
    ```
    */
    pub const NULL: Address = Address { 0: 0 };

    /**
    A address with an invalid value.

    # Examples

    ```
    use memflow_core::types::Address;

    fn test() {
        println!("address: {}", Address::INVALID);
    }
    ```
    */
    pub const INVALID: Address = Address { 0: !0 };

    /**
    Returns an address with a value of zero.

    # Examples

    ```
    use memflow_core::types::Address;

    fn test() {
        println!("address: {}", Address::null());
    }
    ```
    */
    pub const fn null() -> Self {
        Address::NULL
    }

    /**
    Checks wether the address is zero or not.

    # Examples

    ```
    use memflow_core::types::Address;

    fn test() {
        assert_eq!(Address::null().is_null(), true);
        assert_eq!(Address::from(0x1000u64).is_null(), false);
    }
    ```
    */
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    /**
    Returns an address with a invalid value.

    # Examples

    ```
    use memflow_core::types::Address;

    fn test() {
        println!("address: {}", Address::invalid());
    }
    ```
    */
    pub const fn invalid() -> Self {
        Address::INVALID
    }

    /**
    Checks wether the address is valid or not.

    # Examples

    ```
    use memflow_core::types::Address;

    fn test() {
        assert_eq!(Address::invalid().is_valid(), false);
        assert_eq!(Address::from(0x1000u64).is_valid(), true);
    }
    ```
    */
    pub const fn is_valid(self) -> bool {
        self.0 != !0
    }

    /**
    Converts the address into a `u32` value.

    # Examples

    ```
    use memflow_core::types::Address;

    fn test() {
        let addr = Address::from(0x1000u64);
        let addr_u32: u32 = addr.as_u32();
    }
    ```
    */
    pub const fn as_u32(self) -> u32 {
        self.0 as u32
    }

    /**
    Converts the address into a `u64` value.

    # Examples

    ```
    use memflow_core::types::Address;

    fn test() {
        let addr = Address::from(0x1000u64);
        let addr_u64: u64 = addr.as_u64();
    }
    ```
    */
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /**
    Converts the address into a `usize` value.

    # Examples

    ```
    use memflow_core::types::Address;

    fn test() {
        let addr = Address::from(0x1000u64);
        let addr_usize: usize = addr.as_usize();
    }
    ```
    */
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    /**
    Aligns the containing address to the given page size.
    It returns the base address of the containing page.

    # Examples

    ```
    use memflow_core::types::{Address, size};

    let addr = Address::from(0x1234);
    let aligned = addr.as_page_aligned(size::kb(4));
    assert_eq!(aligned, Address::from(0x1000));
    ```
    */
    pub const fn as_page_aligned(self, page_size: usize) -> Address {
        Address {
            0: self.0 - self.0 % (page_size as u64),
        }
    }

    /**
    Returns true or false wether the bit at the specified index is either 0 or 1.
    An index of 0 will check the least significant bit.

    # Examples

    ```
    use memflow_core::types::Address;

    let addr = Address::from(2);
    let bit = addr.bit_at(1);
    assert_eq!(bit, true);
    ```
    */
    pub const fn bit_at(self, idx: u8) -> bool {
        (self.0 & ((1 as u64) << idx)) != 0
    }
}

/**
Returns a address with a value of zero.

# Examples

```
use memflow_core::types::Address;

fn test() {
    assert_eq!(Address::default().is_null(), true);
}
```
*/
impl Default for Address {
    fn default() -> Self {
        Self::null()
    }
}

/**
Adds a `usize` to a `Address` which results in a `Address`.
# Examples
```
use memflow_core::types::Address;
fn test() {
    assert_eq!(Address::from(10) + 5usize, Address::from(15));
}
```
*/
impl ops::Add<usize> for Address {
    type Output = Self;

    fn add(self, other: usize) -> Self {
        Self {
            0: self.0 + (other as u64),
        }
    }
}

/**
Adds a `usize` to a `Address`.

# Examples

```
use memflow_core::types::Address;
fn test() {
    let mut addr = Address::from(10);
    addr += 5;
    assert_eq!(addr, Address::from(15));
}
```
*/
impl ops::AddAssign<usize> for Address {
    fn add_assign(&mut self, other: usize) {
        *self = Self {
            0: self.0 + (other as u64),
        }
    }
}

// TODO: guarantee no underlfow
/**
Subtracts a `Address` from a `Address` resulting in a `usize`.

# Examples

```
use memflow_core::types::Address;

fn test() {
    assert_eq!(Address::from(10) - 5, Address::from(5));
}
```
*/
impl ops::Sub for Address {
    type Output = usize;

    fn sub(self, other: Self) -> usize {
        (self.0 - other.0) as usize
    }
}

// TODO: guarantee no underlfow
/// Subtracts a `usize` from a `Address` resulting in a `Address`.
impl ops::Sub<usize> for Address {
    type Output = Address;

    fn sub(self, other: usize) -> Address {
        Self {
            0: self.0 - (other as u64),
        }
    }
}

/**
Subtracts a `usize` from a `Address`.

# Examples

```
use memflow_core::types::Address;

fn test() {
    let mut addr = Address::from(10);
    addr -= 5;
    assert_eq!(addr, Address::from(5));
}

```
*/
impl ops::SubAssign<usize> for Address {
    fn sub_assign(&mut self, other: usize) {
        *self = Self {
            0: self.0 - (other as u64),
        }
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}
impl fmt::UpperHex for Address {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}", self.0)
    }
}
impl fmt::LowerHex for Address {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}
impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::super::size;
    use super::*;

    #[test]
    fn test_null_valid() {
        assert_eq!(Address::null().is_null(), true);
        assert_eq!(Address::invalid().is_valid(), false);
    }

    #[test]
    fn test_from() {
        assert_eq!(Address::from(1337).as_u64(), 1337);
        assert_eq!(Address::from(4321).as_usize(), 4321);
    }

    #[test]
    fn test_alignment() {
        assert_eq!(
            Address::from(0x1234).as_page_aligned(size::kb(4)),
            Address::from(0x1000)
        );
        assert_eq!(
            Address::from(0xFFF1_2345u64).as_page_aligned(0x10000),
            Address::from(0xFFF1_0000u64)
        );
    }

    #[test]
    fn test_bits() {
        assert_eq!(Address::from(1).bit_at(0), true);
        assert_eq!(Address::from(1).bit_at(1), false);
        assert_eq!(Address::from(1).bit_at(2), false);
        assert_eq!(Address::from(1).bit_at(3), false);

        assert_eq!(Address::from(2).bit_at(0), false);
        assert_eq!(Address::from(2).bit_at(1), true);
        assert_eq!(Address::from(2).bit_at(2), false);
        assert_eq!(Address::from(2).bit_at(3), false);

        assert_eq!(Address::from(13).bit_at(0), true);
        assert_eq!(Address::from(13).bit_at(1), false);
        assert_eq!(Address::from(13).bit_at(2), true);
        assert_eq!(Address::from(13).bit_at(3), true);
    }

    #[test]
    fn test_ops() {
        assert_eq!(Address::from(10) + 5usize, Address::from(15));

        assert_eq!(Address::from(10) - Address::from(5), 5usize);
        assert_eq!(Address::from(100) - 5usize, Address::from(95));
    }
}
