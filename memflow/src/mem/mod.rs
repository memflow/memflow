/*!
This module covers all implementations and traits related to
reading/writing [physical](phys_mem/index.html) and [virtual](virt_mem/index.html) memory.

The [cache](cache/index.html) module contains all caching related
implementations. The caches just wrap the physical and virtual accessors
and are themselves a memory backend.

TODO: more documentation
*/

pub mod cache;
pub mod mem_data;
pub mod mem_map;
pub mod memory_view;
pub mod phys_mem;
pub mod virt_mem;
pub mod virt_translate;

#[cfg(feature = "std")]
pub mod cursor;

#[doc(hidden)]
pub use cache::*;
#[doc(hidden)]
pub use mem_map::{MemoryMap, PhysicalMemoryMapping};
#[doc(hidden)]
#[cfg(feature = "plugins")]
pub use phys_mem::ConnectorInstanceArcBox;
#[doc(hidden)]
pub use phys_mem::{
    PhysicalMemory, PhysicalMemoryMetadata, PhysicalReadFailCallback, PhysicalWriteFailCallback,
};
#[doc(hidden)]
pub use virt_mem::VirtualDma;
//#[doc(hidden)]
//pub use virt_mem_batcher::VirtualMemoryBatcher;
#[doc(hidden)]
pub use virt_translate::{DirectTranslate, VirtualTranslate, VirtualTranslate2};

#[doc(hidden)]
pub use memory_view::{MemoryView, MemoryViewMetadata, ReadFailCallback, WriteFailCallback};

#[doc(hidden)]
pub use mem_data::*;

#[cfg(feature = "std")]
#[doc(hidden)]
pub use cursor::MemoryCursor;
