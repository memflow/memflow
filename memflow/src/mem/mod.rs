/*!
This module covers all implementations and traits related to
reading/writing [physical](phys_mem/index.html) and [virtual](virt_mem/index.html) memory.

The [cache](cache/index.html) module contains all caching related
implementations. The caches just wrap the physical and virtual accessors
and are themselves a memory backend.

TODO: more documentation
*/

pub mod cache;
pub mod mem_map;
pub mod phys_mem;
pub mod phys_mem_batcher;
pub mod virt_mem;
pub mod virt_mem_batcher;
pub mod virt_translate;

#[cfg(feature = "std")]
pub mod cursor;

#[doc(hidden)]
pub use cache::*;
#[doc(hidden)]
pub use mem_map::{MemoryMap, PhysicalMemoryMapping};
#[doc(hidden)]
pub use phys_mem::{
    AsPhysicalMemory, PhysicalMemory, PhysicalMemoryMetadata,
    PhysicalReadData, PhysicalReadIterator, PhysicalWriteData, PhysicalWriteIterator,
};
#[doc(hidden)]
#[cfg(feature = "plugins")]
pub use phys_mem::{
    ConnectorInstanceArcBox,
};
#[doc(hidden)]
pub use phys_mem_batcher::PhysicalMemoryBatcher;
#[doc(hidden)]
pub use virt_mem::{AsVirtualMemory, VirtualDma, VirtualMemory, VirtualReadData, VirtualWriteData};
#[doc(hidden)]
pub use virt_mem_batcher::VirtualMemoryBatcher;
#[doc(hidden)]
pub use virt_translate::{DirectTranslate, VirtualTranslate};

#[cfg(feature = "std")]
#[doc(hidden)]
pub use cursor::{PhysicalMemoryCursor, VirtualMemoryCursor};
