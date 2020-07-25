// bit mask macros
pub fn make_bit_mask<T: core::convert::TryInto<u8>>(a: T, b: T) -> u64 {
    (0xffff_ffff_ffff_ffff >> (63 - b.try_into().ok().unwrap()))
        & !(((1 as u64) << a.try_into().ok().unwrap()) - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_bit_mask() {
        assert_eq!(make_bit_mask(0, 11), 0xfff);
        assert_eq!(make_bit_mask(12, 20), 0x001f_f000);
        assert_eq!(make_bit_mask(21, 29), 0x3fe0_0000);
        assert_eq!(make_bit_mask(30, 38), 0x007f_c000_0000);
        assert_eq!(make_bit_mask(39, 47), 0xff80_0000_0000);
        assert_eq!(make_bit_mask(12, 51), 0x000f_ffff_ffff_f000);
    }
}
