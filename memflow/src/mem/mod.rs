//! This module covers all implementations and traits related to
//! reading/writing [physical](phys_mem/index.html) and [virtual](virt_mem/index.html) memory.
//!
//! The [cache](cache/index.html) module contains all caching related
//! implementations. The caches just wrap the physical and virtual accessors
//! and are themselves a memory backend.
//!
//! TODO: more documentation

pub mod mem_data;
pub mod mem_map;
pub mod memory_view;
pub mod phys_mem;
pub mod virt_mem;
pub mod virt_translate;

pub use mem_map::{MemoryMap, PhysicalMemoryMapping};
pub use phys_mem::{CachedPhysicalMemory, PhysicalMemory, PhysicalMemoryMetadata};
#[cfg(feature = "std")]
pub use phys_mem::{DelayedPhysicalMemory, PhysicalMemoryMetrics};
pub use virt_mem::VirtualDma;
pub use virt_translate::{
    CachedVirtualTranslate, DirectTranslate, VirtualTranslate, VirtualTranslate2,
    VirtualTranslate3, VtopFailureCallback, VtopOutputCallback,
};

pub use memory_view::{CachedView, MemoryView, MemoryViewBatcher, MemoryViewMetadata};

#[cfg(feature = "std")]
pub use memory_view::MemoryCursor;

pub use mem_data::*;
