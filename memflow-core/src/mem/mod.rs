/*!
This module covers all implementations and traits related to
reading/writing [physical](phys/index.html) and [virtual](virt/index.html) memory.

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

#[cfg(any(feature = "dummy_mem", test))]
pub mod dummy;

#[doc(hidden)]
pub use cache::*; // TODO: specify pub declarations
#[doc(hidden)]
pub use mem_map::MemoryMap;
#[doc(hidden)]
pub use phys_mem::{
    CloneablePhysicalMemory, PhysicalMemory, PhysicalMemoryBox, PhysicalMemoryMetadata,
    PhysicalReadData, PhysicalReadIterator, PhysicalWriteData, PhysicalWriteIterator,
};
#[doc(hidden)]
pub use phys_mem_batcher::PhysicalMemoryBatcher;
#[doc(hidden)]
pub use virt_mem::{VirtualDMA, VirtualMemory, VirtualReadData, VirtualWriteData};
#[doc(hidden)]
pub use virt_mem_batcher::VirtualMemoryBatcher;
#[doc(hidden)]
pub use virt_translate::{DirectTranslate, VirtualTranslate};
