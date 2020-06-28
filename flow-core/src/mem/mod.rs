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

pub use cache::*; // TODO: specify pub declarations
pub use mem_map::MemoryMap;
pub use phys_mem::{PhysicalMemory, PhysicalReadIterator, PhysicalWriteIterator};
pub use phys_mem_batcher::PhysicalMemoryBatcher;
pub use virt_mem::{VirtualFromPhysical, VirtualMemory};
pub use virt_mem_batcher::VirtualMemoryBatcher;
pub use virt_translate::{TranslateArch, VirtualTranslate};
