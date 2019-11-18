use crate::arch::ByteOrder;
use crate::address::Length;

pub fn byte_order() -> ByteOrder {
    ByteOrder::LittleEndian
}

pub fn page_size() -> Length {
    Length::from_kb(4)
}

pub fn len_addr() -> Length {
    Length::from(4)
}

pub fn len_u64() -> Length {
    Length::from(4)
}

pub fn len_u32() -> Length {
    Length::from(4)
}

pub fn len_i64() -> Length {
    Length::from(4)
}

pub fn len_i32() -> Length {
    Length::from(4)
}

pub fn len_f32() -> Length {
    Length::from(4)
}
