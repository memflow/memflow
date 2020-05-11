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
pub mod vat;
pub mod virt;
pub mod virt_ctx;

pub use cache::*;
pub use phys::AccessPhysicalMemory;
pub use vat::VirtualAddressTranslator;
pub use virt::AccessVirtualMemory;
pub use virt_ctx::VirtualMemoryContext;
