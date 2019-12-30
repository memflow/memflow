// TODO: rename addr to something useful or rename this crate?
pub mod addr;
pub mod length;
pub mod offset;

// forward declares
pub use addr::*;
pub use length::*;
pub use offset::*;

// macro declares
#[macro_export]
macro_rules! addr {
    ($a:expr) => {
        Address::from($a)
    };
}

#[macro_export]
macro_rules! len {
    ($a:expr) => {
        Length::from($a)
    };
}

#[macro_export]
macro_rules! len_b {
    ($a:expr) => {
        Length::from_b($a)
    };
}

#[macro_export]
macro_rules! len_kb {
    ($a:expr) => {
        Length::from_kb($a)
    };
}

#[macro_export]
macro_rules! len_kib {
    ($a:expr) => {
        Length::from_kib($a)
    };
}

#[macro_export]
macro_rules! len_mb {
    ($a:expr) => {
        Length::from_mb($a)
    };
}

#[macro_export]
macro_rules! len_mib {
    ($a:expr) => {
        Length::from_mib($a)
    };
}

#[macro_export]
macro_rules! len_gb {
    ($a:expr) => {
        Length::from_gb($a)
    };
}

#[macro_export]
macro_rules! len_gib {
    ($a:expr) => {
        Length::from_gib($a)
    };
}

#[macro_export]
macro_rules! offs {
    ($a:expr) => {
        Offset::from($a)
    };
}
