//! Describes a view over the modules of a process

use crate::cglue::*;
use crate::prelude::v1::{Result, *};
use std::prelude::v1::*;

#[cfg_attr(feature = "plugins", cglue_trait)]
#[cglue_forward]
#[int_result]
pub trait ModuleView: Send {
    /// Walks the process' module list and calls the provided callback for each module structure
    /// address
    ///
    /// # Arguments
    /// * `target_arch` - sets which architecture to retrieve the modules for (if emulated). Choose
    /// between `Some(ProcessInfo::sys_arch())`, and `Some(ProcessInfo::proc_arch())`. `None` for all.
    /// * `callback` - where to pass each matching module to. This is an opaque callback.
    fn module_address_list_callback(&mut self, callback: ModuleAddressCallback) -> Result<()>;

    // TODO:
}
