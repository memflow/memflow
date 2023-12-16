//!
//! # memflow
//!
//! Machine introspection made easy
//!
//! ## Introduction
//!
//! memflow is a library that enables introspection of various machines (hardware, virtual machines,
//! memory dumps) in a generic fashion. There are 2 primary types of objects in memflow - _Connectors_
//! and _OS layers_. Connector provides raw access to physical memory of a machine. Meanwhile, OS
//! layer builds a higher level abstraction over running operating system, providing access to running
//! processes, input events, etc. These objects are incredibly flexible as they can be chained together
//! to gain access to a process running multiple levels of virtualization deep (see figure below).
//!
//! ```text
//! +-----------+        +-----------+
//! | native OS |        | leechcore |
//! +-+---------+        +-+---------+
//!   |                    |
//!   |  +-----------+     |  +----------+
//!   +->|  QEMU VM  |     +->| Win32 OS |
//!      +-+---------+        +-+--------+
//!        |                    |
//!        |  +----------+      |  +-----------+
//!        +->| Win32 OS |      +->| lsass.exe |
//!           +-+--------+         +-----------+
//!             |
//!             |  +-----------+
//!             +->|  Hyper-V  |
//!                +-+---------+
//!                  |
//!                  |  +----------+
//!                  +->| Linux OS |
//!                     +-+--------+
//!                       |
//!                       |  +-----------+
//!                       +->| SSHD Proc |
//!                          +-----------+
//!
//! (Example chains of access. For illustrative purposes only - Hyper-V Connector and Linux OS are not yet available)
//! ```
//!
//! As a library user, you do not have to worry about delicacies of chaining - everything is provided,
//! batteries included. See one of our [examples](memflow/examples/process_list.rs) on how simple it is to
//! build a chain (excluding parsing). All Connectors and OS layers are dynamically loadable with common
//! interface binding them.
//!
//! All of this flexibility is provided with very robust and efficient backend - memory interface is
//! batchable and divisible, which gets taken advantage of by our throughput optimized virtual address
//! translation pipeline that is able to walk the entire process virtual address space in under a second.
//! Connectors and OS layers can be composed with the vast library of generic caching mechanisms, utility
//! functions and data structures.
//!
//! The memflow ecosystem is not bound to just Rust - Connector and OS layer functions are linked together
//! using C ABI, thus users can write code that interfaces with them in other languages, such as C, C++, Zig,
//! etc. In addition, these plugins can too be implemented in foreign languages - everything is open.
//!
//! Overall, memflow is the most robust, efficient and flexible solution out there for machine introspection.
//!
//! # Structure
//!
//! memflow is separated into modules that are concerned with different parts of the ecosystem.
//! [mem](crate::mem) module is concerned with memory interfacing, [os](crate::os) module is
//! conerned with OS abstractions, [architecture](crate::architecture) module defines
//! specification of a computer architecture, as well as several built-in architectures,
//! [types](crate::types) concerns itself with data types used throughout memflow, while
//! [plugins](crate::plugins) module defines the dynamically loadable plugin types.
//!
//! ## Getting started
//!
//! To quickly get started with the memflow library, simply include its prelude:
//!
//! ```
//! use memflow::prelude::v1::*;
//! ```
//!
//! Afterwards, you will want to build a memflow object using the plugin inventory:
//!
//! ```
//! # use memflow::prelude::v1::*;
//! # fn main() -> Result<()> {
//! let inventory = Inventory::scan();
//! # let inventory = inventory.with_workspace()?;
//!
//! let conn = inventory.create_connector("dummy", None, None)?;
//! # Ok(())
//! # }
//!
//! ```
//!
//! ## Traits
//!
//! While Connectors and OS layers are the primary user facing objects, functionality of these
//! objects is provided through a set of traits.
//!
//! ### Core traits
//!
//! [MemoryView](crate::mem::memory_view::MemoryView) is the primary trait of issuing read and
//! write operations. Required functions are a bit intimidating, because memflow wants IO to be
//! batchable, which enables impressive performance, however, there are several helpers available
//! for performing simple read and write operations.
//!
//! [Os](crate::os::root::Os) and [OsInner](crate::os::root::OsInner) are the traits that deal with
//! higher level OS abstractions. They enable access to [Processes](crate::os::process::Process)
//! together with their MemoryViews. The reason for OsInner existance is lack of GATs, however,
//! this complexity should be removed as soon as the feature is
//! [stabilized](https://github.com/rust-lang/rust/pull/96709).
//!
//! [PhysicalMemory](crate::mem::phys_mem::PhysicalMemory) trait is implemented by connectors. It
//! embeds special metadata which is used by our memory caches, however is not much different from
//! MemoryView. Users performing physical reads may use the
//! [phys_view](crate::mem::phys_mem::PhysicalMemory::phys_view) function to access a view to this
//! physical address space and gain access to the helper methods.
//!
//! [VirtualTranslate](crate::mem::virt_translate::VirtualTranslate) trait is optionally provided
//! by processes in order to translate virtual addresses into physical ones. This is a lower level
//! trait.
//!
//! ### Class diagrams
//!
//! ```text
//! +----------------------------+    +----------------------------+
//! |                            |    |                            |
//! |          Connector         |    |         OS Layer           |
//! |                            |    |                            |
//! | +------------------------+ |    | +------------------------+ |
//! | |     PhysicalMemory     | |    | |        OsInner         | |
//! | +------------------------+ |    | +------------------------+ |
//! |                            |    |                            |
//! | +------------------------+ |    | +------------------------+ |
//! | |         Clone          | |    | |         Clone          | |
//! | +------------------------+ |    | +------------------------+ |
//! |                            |    |                            |
//! |         Optional:          |    |         Optional:          |
//! |                            |    |                            |
//! | +------------------------+ |    | +------------------------+ |
//! | |   ConnectorCpuState    | |    | |       MemoryView       | |
//! | +------------------------+ |    | +------------------------+ |
//! |                            |    |                            |
//! +----------------------------+    | +------------------------+ |
//!                                   | |    VirtualTranslate    | |
//!                                   | +------------------------+ |
//!                                   |                            |
//!                                   | +------------------------+ |
//!                                   | |     PhysicalMemory     | |
//!                                   | +------------------------+ |
//!                                   |                            |
//!                                   | +------------------------+ |
//!                                   | |       OsKeyboard       | |
//! +----------------------------+    | +------------------------+ |
//! |                            |    +----------------------------+
//! |     IntoProcessInstance    |
//! |                            |
//! | +------------------------+ |    +----------------------------+
//! | |        Process         | |    |                            |
//! | +------------------------+ |    |       ProcessInstance      |
//! |                            |    |                            |
//! | +------------------------+ |    | +------------------------+ |
//! | |       MemoryView       | |    | |        Process         | |
//! | +------------------------+ |    | +------------------------+ |
//! |                            |    |                            |
//! | +------------------------+ |    | +------------------------+ |
//! | |         Clone          | |    | |       MemoryView       | |
//! | +------------------------+ |    | +------------------------+ |
//! |                            |    |                            |
//! |         Optional:          |    |         Optional:          |
//! |                            |    |                            |
//! | +------------------------+ |    | +------------------------+ |
//! | |    VirtualTranslate    | |    | |    VirtualTranslate    | |
//! | +------------------------+ |    | +------------------------+ |
//! |                            |    |                            |
//! +----------------------------+    +----------------------------+
//! ```
//!
//! # Philosophy
//!
//! The core idea of memflow is to generalize where possible, specialize when needed.
//!
//! Best practice of writing memflow functions is to write them generically - use `impl Trait`
//! notation to define the type of object needed. This will allow for your code to work on both
//! dynamically loaded plugins, as well as custom, statically linked, and potentially more
//! efficient memory/OS objects.
//!
//! For instance, if you want to perform a memory read, define the function as follows:
//!
//! ```
//! use memflow::prelude::v1::*;
//! # use memflow::dummy::{DummyMemory, DummyOs};
//!
//! // Define the function with `impl Trait` notation
//! fn special_read(mem: &mut impl MemoryView) -> Result<u64> {
//!     mem.read(Address::from(0x42)).data()
//! }
//!
//! // Use it with plugin object
//! let mut inventory = Inventory::scan();
//! # let mut inventory = inventory.with_workspace().unwrap();
//! let args = str::parse(":4m").unwrap();
//! let conn = inventory.create_connector("dummy", None, Some(&args))
//!     .unwrap();
//!
//! assert!(special_read(&mut conn.into_phys_view()).is_ok());
//!
//! // Use it with statically built connector
//! let mut mem = DummyMemory::new(size::mb(4));
//!
//! assert!(special_read(&mut mem.phys_view()).is_ok());
//!
//! // Use it with statically built process
//! let mut proc = DummyOs::quick_process(size::mb(4), &[]);
//!
//! assert!(special_read(&mut proc).is_ok());
//! ```

//#![warn(missing_docs)]

// due to the fact that umem equals u64 when compiling with a x86_64 target clippy issues false-positives on these conversions.
// targets other than x86_64 still might require those.
#![allow(clippy::unnecessary_cast)]
// this issue is triggered due to an issue in bitmask 1.x
// since upgrading to 2.x broke code generation via cglue-bindgen / cbindgen
// we are allowing this lint temporarily
#![allow(clippy::bad_bit_mask)]
// no-std-compat
#![cfg_attr(not(feature = "std"), no_std)]
extern crate no_std_compat as std;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate smallvec;

pub mod error;

#[macro_use]
pub mod types;

pub mod architecture;

pub mod mem;

pub mod connector;

#[cfg(feature = "plugins")]
pub mod plugins;

pub mod os;

pub mod iter;

// forward declare
#[doc(hidden)]
pub mod derive {
    pub use ::memflow_derive::*;
}

#[doc(hidden)]
pub mod cglue {
    pub use ::cglue::prelude::v1::*;
}

#[doc(hidden)]
#[cfg(feature = "abi_stable")]
pub mod abi_stable {
    pub use ::abi_stable::*;
}

#[doc(hidden)]
pub mod dataview {
    pub use ::dataview::*;
    pub use ::memflow_derive::Pod;
}

#[doc(hidden)]
#[cfg(any(feature = "dummy_mem", test))]
pub mod dummy;

// TODO: modules should be cleanly seperated here and only necessary types should be exported
#[doc(hidden)]
#[allow(ambiguous_glob_reexports)]
pub mod prelude {
    pub mod v1 {
        pub use crate::architecture::*;
        pub use crate::cglue::*;
        pub use crate::connector::*;
        pub use crate::dataview::*;
        pub use crate::derive::*;
        pub use crate::error::*;
        pub use crate::iter::*;
        pub use crate::mem::*;
        pub use crate::os::*;
        #[cfg(feature = "plugins")]
        pub use crate::plugins::os::*;
        #[cfg(feature = "plugins")]
        pub use crate::plugins::*;
        pub use crate::types::*;
    }
    pub use v1::*;
}
