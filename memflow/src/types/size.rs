///! This module contains helper functions for creating various byte sizes.
///! All function are const and will be [optimized](https://rust.godbolt.org/z/T6LiwJ) by rustc.
use super::umem;

/// Returns a umem representing the length in bytes from the given number of kilobytes.
pub const fn kb(kb: umem) -> umem {
    kb * 1024
}

/// Returns a umem representing the length in bytes from the given number of kilobits.
pub const fn kib(kib: umem) -> umem {
    kb(kib) / 8
}

/// Returns a umem representing the length in bytes from the given number of megabytes.
pub const fn mb(mb: umem) -> umem {
    kb(mb) * 1024
}

/// Returns a umem representing the length in bytes from the given number of megabits.
pub const fn mib(mib: umem) -> umem {
    mb(mib) / 8
}

/// Returns a umem representing the length in bytes from the given number of gigabytes.
pub const fn gb(gb: umem) -> umem {
    mb(gb) * 1024
}

/// Returns a umem representing the length in bytes from the given number of gigabits.
pub const fn gib(gib: umem) -> umem {
    gb(gib) / 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        assert_eq!(kb(20), 20480);
        assert_eq!(kib(123), 15744);
        assert_eq!(mb(20), 20_971_520);
        assert_eq!(mib(52), 6_815_744);
        assert_eq!(gb(20), 21_474_836_480);
        assert_eq!(gib(52), 6_979_321_856);
    }
}
