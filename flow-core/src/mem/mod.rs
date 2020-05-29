/*!
This module covers all implementations and traits related to
reading/writing [physical](phys/index.html) and [virtual](virt/index.html) memory.

The [cache](cache/index.html) module contains all caching related
implementations. The caches just wrap the physical and virtual accessors
and are themselves a memory backend.

TODO: more documentation
*/

pub mod cache;
pub mod phys;
pub mod proc_mem_ctx;
pub mod vat;
pub mod virt;

pub use cache::*;
pub use phys::AccessPhysicalMemory;
pub use proc_mem_ctx::ProcessMemoryContext;
pub use vat::VirtualAddressTranslator;
pub use virt::AccessVirtualMemory;
