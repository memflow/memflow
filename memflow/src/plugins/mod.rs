/*!
This module contains functions related to the Inventory system for Connectors and Os-Plugins.

All functionality in this module is gated behind `plugins` feature.
*/

use crate::cglue::*;
use cglue::trait_group::c_void;
use std::prelude::v1::*;

pub mod args;
#[doc(hidden)]
pub use args::{ArgDescriptor, Args, ArgsValidator};

pub mod plugin_analyzer;

// TODO: feature gate
pub mod inventory;
pub use inventory::Inventory;

pub mod builder;
pub use builder::*;

// cbindgen fails to properly parse this as return type
pub type OptionVoid = Option<&'static mut c_void>;

pub type LibArc = CArc<c_void>;

pub mod connector;
pub use connector::{
    cglue_connectorinstance::*, ConnectorArgs, ConnectorDescriptor, ConnectorMiddlewareArgs,
    LoadableConnector,
};
pub type ConnectorInputArg = <LoadableConnector as Loadable>::InputArg;

pub mod os;
pub use os::{
    cglue_intoprocessinstance::*, cglue_osinstance::*, cglue_processinstance::*,
    IntoProcessInstanceArcBox, LoadableOs, MuOsInstanceArcBox, OsArgs, OsDescriptor,
    OsInstanceArcBox, ProcessInstanceArcBox,
};
pub type OsInputArg = <LoadableOs as Loadable>::InputArg;

pub mod logger;
pub use logger::*; // TODO: restrict

// do not expose the util module in documentation but forward all functions
pub(crate) mod util;
pub use util::*;

use crate::error::{Result, *};

use log::warn;
use std::mem::MaybeUninit;

use abi_stable::{type_layout::TypeLayout, StableAbi};
use libloading::Library;
use once_cell::sync::OnceCell;

use self::plugin_analyzer::PluginKind;

/// Exported memflow plugins version
pub const MEMFLOW_PLUGIN_VERSION: i32 = 1;

/// Help and Target callbacks
pub type HelpCallback<'a> = OpaqueCallback<'a, ReprCString>;

/// Context for a single library.
pub struct LibContext {
    lib: Library,
    logger: OnceCell<Box<PluginLogger>>,
}

impl From<Library> for LibContext {
    fn from(lib: Library) -> Self {
        Self {
            lib,
            logger: Default::default(),
        }
    }
}

impl LibContext {
    /// Get a static logger for this library context.
    ///
    /// # Safety
    ///
    /// The returned logger is not actually static. Caller must ensure the reference won't dangle
    /// after the library is unloaded. This is typically ensured by only passing this reference to
    /// the underlying library code.
    pub unsafe fn get_logger(&self) -> &'static PluginLogger {
        (&**self.logger.get_or_init(|| Box::new(PluginLogger::new())) as *const PluginLogger)
            .as_ref()
            .unwrap()
    }

    pub fn try_get_logger(&self) -> Option<&PluginLogger> {
        self.logger.get().map(|l| &**l)
    }
}

/// Target information structure
#[repr(C)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct TargetInfo {
    /// Name of the target
    pub name: ReprCString,
}

pub type TargetCallback<'a> = OpaqueCallback<'a, TargetInfo>;

#[repr(C)]
pub struct PluginDescriptor<T: Loadable> {
    /// The plugin api version for when the plugin was built.
    /// This has to be set to `MEMFLOW_PLUGIN_VERSION` of memflow.
    ///
    /// If the versions mismatch the inventory will refuse to load.
    pub plugin_version: i32,

    /// Does the plugin accept an input parameter?
    pub accept_input: bool,

    /// Layout of the input type.
    pub input_layout: &'static TypeLayout,

    /// Layout of the loaded type.
    pub output_layout: &'static TypeLayout,

    /// The name of the plugin.
    /// This name will be used when loading a plugin from the inventory.
    ///
    /// During plugin discovery, the export suffix has to match this name being capitalized
    pub name: CSliceRef<'static, u8>,

    /// The version of the connector.
    /// If multiple connectors are installed the latest is picked.
    pub version: CSliceRef<'static, u8>,

    /// The description of the connector.
    pub description: CSliceRef<'static, u8>,

    /// Retrieves a help string from the plugin (lists all available commands)
    pub help_callback: Option<extern "C" fn(callback: HelpCallback) -> ()>,

    /// Retrieves a list of available targets for the plugin
    pub target_list_callback: Option<extern "C" fn(callback: TargetCallback) -> i32>,

    /// Create instance of the plugin
    pub create: CreateFn<T>,
}

// This warning is misleading here. `Loadable::ArgsType` isn't constrained to be `#[repr(C)]` here
// but both `ConnectorArgs` and `OsArgs` that use it are marked as `#[repr(C)]`.
#[allow(improper_ctypes_definitions)]
pub type CreateFn<T> = extern "C" fn(
    Option<&<T as Loadable>::ArgsType>,
    <T as Loadable>::CInputArg,
    lib: LibArc,
    logger: Option<&'static PluginLogger>,
    &mut MaybeUninit<<T as Loadable>::Instance>,
) -> i32;

/// Defines a common interface for loadable plugins
pub trait Loadable: Sized {
    type Instance: StableAbi;
    type InputArg;
    type CInputArg: StableAbi;
    type ArgsType;

    /// Identifier string of the plugin
    fn ident(&self) -> &str;

    /// The type of plugin
    fn plugin_kind() -> PluginKind;

    /// Constant prefix for the plugin type
    fn export_prefix() -> &'static str;

    fn new(descriptor: PluginDescriptor<Self>) -> Self;

    fn from_instance(instance: &CArc<LibContext>, export_name: &str) -> Result<Self> {
        // stripping initial _ is required for MACH builds
        let export_name = export_name.strip_prefix('_').unwrap_or(export_name);

        let raw_descriptor = unsafe {
            instance
                .as_ref()
                // TODO: support loading without arc
                .ok_or(Error(ErrorOrigin::Inventory, ErrorKind::Uninitialized))?
                .lib
                .get::<*mut PluginDescriptor<Self>>(format!("{}\0", export_name).as_bytes())
                .map_err(|_| Error(ErrorOrigin::Inventory, ErrorKind::MemflowExportsNotFound))?
                .read()
        };

        // check version
        if raw_descriptor.plugin_version != MEMFLOW_PLUGIN_VERSION {
            warn!(
                "{} has a different version. version {} required, found {}.",
                export_name, MEMFLOW_PLUGIN_VERSION, raw_descriptor.plugin_version
            );
            return Err(Error(ErrorOrigin::Inventory, ErrorKind::VersionMismatch));
        }

        // check abi compatability
        if VerifyLayout::check::<Self::CInputArg>(Some(raw_descriptor.input_layout))
            .and(VerifyLayout::check::<Self::Instance>(Some(
                raw_descriptor.output_layout,
            )))
            .is_valid_strict()
        {
            Ok(Self::new(raw_descriptor))
        } else {
            // TODO: print filename
            warn!("{} has a different abi version.", export_name,);
            Err(Error(ErrorOrigin::Inventory, ErrorKind::VersionMismatch))
        }
    }

    /// Retrieves the help text for this plugin
    fn help(&self) -> Result<String>;

    /// Retrieves the list of available targets for this plugin
    fn target_list(&self) -> Result<Vec<TargetInfo>>;

    /// Creates an `Instance` of the library
    ///
    /// This function assumes that `load` performed necessary safety checks
    /// for validity of the library.
    fn instantiate(
        &self,
        library: CArc<LibContext>,
        input: Self::InputArg,
        args: Option<&Self::ArgsType>,
    ) -> Result<Self::Instance>;
}
