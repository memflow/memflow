// bit mask macros
pub const fn make_bit_mask(a: u32, b: u32) -> u64 {
    (0xffffffffffffffff >> (63 - b)) & !(((1 as u64) << a) - 1)
}

macro_rules! get_bit {
    ($a:expr, $b:expr) => {
        ($a & ((1 as u64) << $b)) != 0
    };
}

// page test macros
#[macro_export]
macro_rules! is_large_page {
    ($a:expr) => {
        get_bit!($a, 7)
    };
}

#[macro_export]
macro_rules! is_transition_page {
    ($a:expr) => {
        get_bit!($a, 11)
    };
}

#[macro_export]
macro_rules! is_prototype_page {
    ($a:expr) => {
        get_bit!($a, 10)
    };
}

// TODO: tests
#[macro_export]
macro_rules! check_entry {
    ($a:expr) => {
        if get_bit!($a, 0) {
            true
        } else if is_transition_page!($a) && !is_prototype_page!($a) {
            true
        } else {
            false
        }
    };
}

// TODO: write tests for these macros
// pagetable indizes
#[macro_export]
macro_rules! pml4_index_bits {
    ($a:expr) => {
        ($a & make_bit_mask(39, 47)) >> 36
    };
}

#[macro_export]
macro_rules! pdpte_index_bits {
    ($a:expr) => {
        ($a & make_bit_mask(30, 38)) >> 27
    };
}

#[macro_export]
macro_rules! pd_index_bits {
    ($a:expr) => {
        ($a & make_bit_mask(21, 29)) >> 18
    };
}

#[macro_export]
macro_rules! pt_index_bits {
    ($a:expr) => {
        ($a & make_bit_mask(12, 20)) >> 9
    };
}

#[cfg(test)]
mod tests {
    use crate::vat::x64::masks::make_bit_mask;

    #[test]
    fn test_make_bit_mask() {
        assert_eq!(make_bit_mask(0, 11), 0xfff);
        assert_eq!(make_bit_mask(12, 20), 0x1ff000);
        assert_eq!(make_bit_mask(21, 29), 0x3fe00000);
        assert_eq!(make_bit_mask(30, 38), 0x7fc0000000);
        assert_eq!(make_bit_mask(39, 47), 0xff8000000000);
        assert_eq!(make_bit_mask(12, 51), 0xffffffffff000);
    }

    #[test]
    fn test_get_bit() {
        //assert_eq!(make_bit_mask(0, 11), 0xfff);
    }

    #[test]
    fn test_is_large_page() {
        assert_eq!(is_large_page!(0x00000000000000F0), true);
        assert_eq!(is_large_page!(0x0000000000000080), true);
        assert_eq!(is_large_page!(0x0000000000000070), false);
        assert_eq!(is_large_page!(0x0000000000000020), false);
    }

    #[test]
    fn test_is_transition_page() {
        assert_eq!(is_transition_page!(0x0000000000000F00), true);
        assert_eq!(is_transition_page!(0x0000000000000800), true);
        assert_eq!(is_transition_page!(0x0000000000000700), false);
        assert_eq!(is_transition_page!(0x0000000000000200), false);
    }

    #[test]
    fn test_is_prototype_page() {
        assert_eq!(is_prototype_page!(0x0000000000000F00), true);
        assert_eq!(is_prototype_page!(0x0000000000000800), false);
        assert_eq!(is_prototype_page!(0x0000000000000700), true);
        assert_eq!(is_prototype_page!(0x0000000000000200), false);
    }
}
