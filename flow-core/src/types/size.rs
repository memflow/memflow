/*!
This module contains helper functions for creating various byte sizes.
All function are const and will be [optimized](https://rust.godbolt.org/z/T6LiwJ) by rustc.
*/

/// Returns a usize representing the length in bytes from the given number of kilobytes.
pub const fn kb(kb: usize) -> usize {
    kb * 1024
}

/// Returns a usize representing the length in bytes from the given number of kilobits.
pub const fn kib(kib: usize) -> usize {
    kb(kib) / 8
}

/// Returns a usize representing the length in bytes from the given number of megabytes.
pub const fn mb(mb: usize) -> usize {
    kb(mb) * 1024
}

/// Returns a usize representing the length in bytes from the given number of megabits.
pub const fn mib(mib: usize) -> usize {
    mb(mib) / 8
}

/// Returns a usize representing the length in bytes from the given number of gigabytes.
pub const fn gb(gb: usize) -> usize {
    mb(gb) * 1024
}

/// Returns a usize representing the length in bytes from the given number of gigabits.
pub const fn gib(gib: usize) -> usize {
    gb(gib) / 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        assert_eq!(kb(20), 20480usize);
        assert_eq!(kib(123), 15744usize);
        assert_eq!(mb(20), 20_971_520_usize);
        assert_eq!(mib(52), 6_815_744_usize);
        assert_eq!(gb(20), 21_474_836_480_usize);
        assert_eq!(gib(52), 6_979_321_856_usize);
    }
}
