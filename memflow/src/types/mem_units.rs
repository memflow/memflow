///! This module contains helper functions for creating various byte sizes.
///! All function are const and will be [optimized](https://rust.godbolt.org/z/T6LiwJ) by rustc.
use super::umem;

#[macro_export]
macro_rules! impl_unit_helper {
    ($mod_name:ident, $type_name:ident) => {
        pub mod $mod_name {

            pub use super::*;

            /// Returns a umem representing the length in bytes from the given number of kilobytes.
            pub const fn kb(kb: $type_name) -> $type_name {
                kb * 1024
            }

            /// Returns a $type_name representing the length in bytes from the given number of kilobits.
            pub const fn kib(kib: $type_name) -> $type_name {
                kb(kib) / 8
            }

            /// Returns a $type_name representing the length in bytes from the given number of megabytes.
            pub const fn mb(mb: $type_name) -> $type_name {
                kb(mb) * 1024
            }

            /// Returns a $type_name representing the length in bytes from the given number of megabits.
            pub const fn mib(mib: $type_name) -> $type_name {
                mb(mib) / 8
            }

            /// Returns a $type_name representing the length in bytes from the given number of gigabytes.
            pub const fn gb(gb: $type_name) -> $type_name {
                mb(gb) * 1024
            }

            /// Returns a $type_name representing the length in bytes from the given number of gigabits.
            pub const fn gib(gib: $type_name) -> $type_name {
                gb(gib) / 8
            }
        }
    };
}

impl_unit_helper!(size, usize);
impl_unit_helper!(mem, umem);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        assert_eq!(size::kb(20), 20480);
        assert_eq!(size::kib(123), 15744);
        assert_eq!(size::mb(20), 20_971_520);
        assert_eq!(size::mib(52), 6_815_744);
        assert_eq!(size::gb(20), 21_474_836_480);
        assert_eq!(size::gib(52), 6_979_321_856);
    }
}
