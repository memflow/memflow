use crate::address::Length;
use crate::arch::ByteOrder;

pub fn byte_order() -> ByteOrder {
    ByteOrder::LittleEndian
}

pub fn page_size() -> Length {
    Length::from_kb(4)
}

pub fn len_addr() -> Length {
    Length::from(8)
}

pub fn len_u64() -> Length {
    Length::from(8)
}

pub fn len_u32() -> Length {
    Length::from(4)
}

pub fn len_u16() -> Length {
    Length::from(2)
}

pub fn len_u8() -> Length {
    Length::from(1)
}

pub fn len_i64() -> Length {
    Length::from(8)
}

pub fn len_i32() -> Length {
    Length::from(4)
}

pub fn len_i16() -> Length {
    Length::from(2)
}

pub fn len_i8() -> Length {
    Length::from(1)
}

pub fn len_f32() -> Length {
    Length::from(4)
}
