/*!
This module covers all implementations and traits related to
reading/writing [physical](phys/index.html) and [virtual](virt/index.html) memory.

The [cache](cache/index.html) module contains all caching related
implementations. The caches just wrap the physical and virtual accessors
and are themselves a memory backend.

TODO: more documentation
*/

pub mod phys_mem;
pub mod virt_mem;
//pub mod cache;
pub mod vat;
pub mod virt_from_phys;

#[cfg(any(feature = "dummy_mem", test))]
pub mod dummy;

pub use phys_mem::{PhysicalMemory, PhysicalReadIterator, PhysicalWriteIterator};
pub use virt_mem::VirtualMemory;
//pub use cache::*;
pub use vat::VAT;
pub use virt_from_phys::VirtualFromPhysical;
