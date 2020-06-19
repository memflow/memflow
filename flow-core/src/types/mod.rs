pub mod address;
pub use address::Address;

pub mod size;

pub mod page;
pub use page::{Page, PageType};

pub mod physical_address;
pub use physical_address::PhysicalAddress;

pub mod pointer32;
pub use pointer32::Pointer32;

pub mod pointer64;
pub use pointer64::Pointer64;

use either as progress;
pub use progress::{Either as Progress, Left as ToDo, Right as Done};

/// Convenience macro which will be replaced by `Address::from`
#[macro_export]
macro_rules! addr {
    ($a:expr) => {
        Address::from($a)
    };
}

/// Convenience macro which will be replaced by `Length::from`
#[macro_export]
macro_rules! len {
    ($a:expr) => {
        Length::from($a)
    };
}

/// Convenience macro which will be replaced by `Length::from_b`
#[macro_export]
macro_rules! len_b {
    ($a:expr) => {
        Length::from_b($a)
    };
}

/// Convenience macro which will be replaced by `Length::from_kb`
#[macro_export]
macro_rules! len_kb {
    ($a:expr) => {
        Length::from_kb($a)
    };
}

/// Convenience macro which will be replaced by `Length::from_kib`
#[macro_export]
macro_rules! len_kib {
    ($a:expr) => {
        Length::from_kib($a)
    };
}

/// Convenience macro which will be replaced by `Length::from_mb`
#[macro_export]
macro_rules! len_mb {
    ($a:expr) => {
        Length::from_mb($a)
    };
}

/// Convenience macro which will be replaced by `Length::from_mib`
#[macro_export]
macro_rules! len_mib {
    ($a:expr) => {
        Length::from_mib($a)
    };
}

/// Convenience macro which will be replaced by `Length::from_gb`
#[macro_export]
macro_rules! len_gb {
    ($a:expr) => {
        Length::from_gb($a)
    };
}

/// Convenience macro which will be replaced by `Length::from_gib`
#[macro_export]
macro_rules! len_gib {
    ($a:expr) => {
        Length::from_gib($a)
    };
}

/// Convenience macro which will be replaced by `Offset::from`
#[macro_export]
macro_rules! offs {
    ($a:expr) => {
        Offset::from($a)
    };
}
