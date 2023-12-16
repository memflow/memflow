//! Module with basic types used in memflow.
//!
//! This module contains types for handling virtual and physical addresses.
//! It also contains types for handling pointers, pages and
//! it exposes different size helpers.

pub mod address;
pub use address::{
    clamp_to_isize, clamp_to_usize, imem, umem, Address, PrimitiveAddress, UMEM_BITS,
};

mod mem_units;
pub use mem_units::*;

pub mod page;
pub use page::{Page, PageType};

pub mod physical_address;
pub use physical_address::PhysicalAddress;

pub mod pointer;
pub use pointer::{Pointer, Pointer32, Pointer64};

pub mod byte_swap;
pub use byte_swap::ByteSwap;

pub mod cache;
pub use cache::{CacheValidator, DefaultCacheValidator};

pub mod gap_remover;
