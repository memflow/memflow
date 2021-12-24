/*!
This module contains functions related to the Inventory system for Connectors and Os-Plugins.

All functionality in this module is gated behind `plugins` feature.
*/

use crate::cglue::*;
use std::ffi::c_void;
use std::prelude::v1::*;

pub mod args;
#[doc(hidden)]
pub use args::{ArgDescriptor, Args, ArgsValidator};

// cbindgen fails to properly parse this as return type
pub type OptionVoid = Option<&'static mut std::ffi::c_void>;

pub mod connector;
pub use connector::{
    cglue_connectorinstance::*, ConnectorArgs, ConnectorDescriptor, LoadableConnector,
    PageCacheParams,
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

pub(crate) mod util;
pub use util::create_bare;

use crate::error::{Result, *};

use log::*;
use std::fs::read_dir;
use std::mem::MaybeUninit;
use std::path::{Path, PathBuf};

use abi_stable::{type_layout::TypeLayout, StableAbi};
use libloading::Library;
use once_cell::sync::OnceCell;

/// Exported memflow plugins version
pub const MEMFLOW_PLUGIN_VERSION: i32 = -8;

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

pub type CreateFn<T> = extern "C" fn(
    Option<&<T as Loadable>::ArgsType>,
    <T as Loadable>::CInputArg,
    lib: CArc<c_void>,
    logger: Option<&'static PluginLogger>,
    &mut MaybeUninit<<T as Loadable>::Instance>,
) -> i32;

/// Defines a common interface for loadable plugins
pub trait Loadable: Sized {
    type Instance: StableAbi;
    type InputArg;
    type CInputArg: StableAbi;
    type ArgsType;

    /// Checks if plugin with the same `ident` already exists in input list
    fn exists(&self, instances: &[LibInstance<Self>]) -> bool {
        instances.iter().any(|i| i.loader.ident() == self.ident())
    }

    /// Identifier string of the plugin
    fn ident(&self) -> &str;

    fn plugin_type() -> &'static str;

    /// Constant prefix for the plugin type
    fn export_prefix() -> &'static str;

    fn new(descriptor: PluginDescriptor<Self>) -> Self;

    fn load(
        path: impl AsRef<Path>,
        library: &CArc<LibContext>,
        export: &str,
    ) -> Result<LibInstance<Self>> {
        // find os descriptor
        let descriptor = unsafe {
            library
                .as_ref()
                // TODO: support loading without arc
                .ok_or(Error(ErrorOrigin::Inventory, ErrorKind::Uninitialized))?
                .lib
                .get::<*mut PluginDescriptor<Self>>(format!("{}\0", export).as_bytes())
                .map_err(|_| Error(ErrorOrigin::Inventory, ErrorKind::MemflowExportsNotFound))?
                .read()
        };

        // check version
        if descriptor.plugin_version != MEMFLOW_PLUGIN_VERSION {
            warn!(
                "{} has a different version. version {} required, found {}.",
                export, MEMFLOW_PLUGIN_VERSION, descriptor.plugin_version
            );
            Err(Error(ErrorOrigin::Inventory, ErrorKind::VersionMismatch))
        } else if VerifyLayout::check::<Self::CInputArg>(Some(descriptor.input_layout))
            .and(VerifyLayout::check::<Self::Instance>(Some(
                descriptor.output_layout,
            )))
            .is_valid_strict()
        {
            Ok(LibInstance {
                path: path.as_ref().to_path_buf(),
                library: library.clone(),
                loader: Self::new(descriptor),
            })
        } else {
            warn!("{} has invalid ABI.", export);
            Err(Error(ErrorOrigin::Inventory, ErrorKind::InvalidAbi))
        }
    }

    /// Try to load a plugin library
    ///
    /// This function will access `library` and try to find corresponding entry for the plugin. If
    /// a valid plugins are found, `Ok(LibInstance<Self>)` is returned. Otherwise, `Err(Error)` is
    /// returned, with appropriate error.
    ///
    /// # Safety
    ///
    /// Loading third party libraries is inherently unsafe and the compiler
    /// cannot guarantee that the implementation of the library
    /// matches the one specified here. This is especially true if
    /// the loaded library implements the necessary interface manually.
    ///
    /// It is adviced to use a provided proc macro to define a valid library.
    fn load_all(path: impl AsRef<Path>) -> Result<Vec<LibInstance<Self>>> {
        let exports = util::find_export_by_prefix(path.as_ref(), Self::export_prefix())?;
        if exports.is_empty() {
            return Err(Error(
                ErrorOrigin::Inventory,
                ErrorKind::MemflowExportsNotFound,
            ));
        }

        // load library
        let library = unsafe { Library::new(path.as_ref()) }
            .map_err(|err| {
                debug!(
                    "found {:?} in library '{:?}' but could not load it: {}",
                    exports,
                    path.as_ref(),
                    err
                );
                Error(ErrorOrigin::Inventory, ErrorKind::UnableToLoadLibrary)
            })
            .map(LibContext::from)
            .map(CArc::from)?;

        Ok(exports
            .into_iter()
            .filter_map(|e| Self::load(path.as_ref(), &library, &e).ok())
            .collect())
    }

    /// Helper function to load a plugin into a list of library instances
    ///
    /// This function will try finding appropriate plugin entry, and add it into the list if there
    /// isn't a duplicate entry.
    ///
    /// # Safety
    ///
    /// Loading third party libraries is inherently unsafe and the compiler
    /// cannot guarantee that the implementation of the library matches the one
    /// specified here.
    fn load_append(path: impl AsRef<Path>, out: &mut Vec<LibInstance<Self>>) -> Result<()> {
        let libs = Self::load_all(path.as_ref())?;
        for lib in libs.into_iter() {
            if !lib.loader.exists(out) {
                info!(
                    "adding plugin '{}/{}': {:?}",
                    Self::plugin_type(),
                    lib.loader.ident(),
                    path.as_ref()
                );
                out.push(lib);
            } else {
                debug!(
                    "skipping library '{}' because it was added already: {:?}",
                    lib.loader.ident(),
                    path.as_ref()
                );
                return Err(Error(ErrorOrigin::Inventory, ErrorKind::AlreadyExists));
            }
        }

        Ok(())
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

/// The core of the plugin system
///
/// It scans system directories and collects valid memflow plugins. They can then be instantiated
/// easily. The reason the libraries are collected is to allow for reuse, and save performance
///
/// # Examples
///
/// Creating a OS instance, the recommended way:
///
/// ```no_run
/// use memflow::plugins::Inventory;
/// # use memflow::plugins::OsInstanceArcBox;
/// # use memflow::error::Result;
/// # fn test() -> Result<OsInstanceArcBox<'static>> {
/// let inventory = Inventory::scan();
/// inventory
///   .builder()
///   .connector("qemu_procfs")
///   .os("win32")
///   .build()
/// # }
/// # test().ok();
/// ```
///
/// Nesting connectors and os plugins:
/// ```no_run
/// use memflow::plugins::{Inventory, Args};
/// # use memflow::error::Result;
/// # fn test() -> Result<()> {
/// let inventory = Inventory::scan();
/// let os = inventory
///   .builder()
///   .connector("qemu_procfs")
///   .os("linux")
///   .connector("qemu_procfs")
///   .os("win32")
///   .build();
/// # Ok(())
/// # }
/// # test().ok();
/// ```
pub struct Inventory {
    connectors: Vec<LibInstance<connector::LoadableConnector>>,
    os_layers: Vec<LibInstance<os::LoadableOs>>,
}

impl Inventory {
    /// Creates a new inventory of plugins from the provided path.
    /// The path has to be a valid directory or the function will fail with an `Error::IO` error.
    ///
    /// # Examples
    ///
    /// Creating a inventory:
    /// ```
    /// use memflow::plugins::Inventory;
    ///
    /// let inventory = Inventory::scan_path("./")
    ///     .unwrap();
    /// ```
    pub fn scan_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut dir = PathBuf::default();
        dir.push(path);

        let mut ret = Self {
            connectors: vec![],
            os_layers: vec![],
        };
        ret.add_dir(dir)?;
        Ok(ret)
    }

    /// Creates a new inventory of plugins by searching various paths.
    ///
    /// It will query PATH, and an additional set of of directories (standard unix ones, if unix,
    /// and "HOME/.local/lib" on all OSes) for "memflow" directory, and if there is one, then
    /// search for libraries in there.
    ///
    /// # Examples
    ///
    /// Creating an inventory:
    /// ```
    /// use memflow::plugins::Inventory;
    ///
    /// let inventory = Inventory::scan();
    /// ```
    pub fn scan() -> Self {
        #[cfg(unix)]
        let extra_paths: Vec<&str> = vec![
            "/opt",
            "/lib",
            "/usr/lib/",
            "/usr/local/lib",
            "/lib32",
            "/lib64",
            "/usr/lib32",
            "/usr/lib64",
            "/usr/local/lib32",
            "/usr/local/lib64",
        ];
        #[cfg(not(unix))]
        let extra_paths: Vec<&str> = vec![];

        let path_iter = extra_paths.into_iter().map(PathBuf::from);

        let path_var = std::env::var_os("PATH");
        let path_iter = path_iter.chain(
            path_var
                .as_ref()
                .map(std::env::split_paths)
                .into_iter()
                .flatten(),
        );

        #[cfg(unix)]
        let path_iter = path_iter.chain(
            dirs::home_dir()
                .map(|dir| dir.join(".local").join("lib"))
                .into_iter(),
        );

        #[cfg(not(unix))]
        let path_iter = path_iter.chain(dirs::document_dir().into_iter());

        let mut ret = Self {
            connectors: vec![],
            os_layers: vec![],
        };

        for mut path in path_iter {
            path.push("memflow");
            ret.add_dir(path).ok();
        }

        if let Ok(pwd) = std::env::current_dir() {
            ret.add_dir(pwd).ok();
        }

        ret
    }

    /// Adds a library directory to the inventory
    ///
    /// This function applies additional filter to only scan potentially wanted files
    ///
    /// # Safety
    ///
    /// Same as previous functions - compiler can not guarantee the safety of
    /// third party library implementations.
    pub fn add_dir_filtered(&mut self, dir: PathBuf, filter: &str) -> Result<&mut Self> {
        if !dir.is_dir() {
            return Err(Error(ErrorOrigin::Inventory, ErrorKind::InvalidPath));
        }

        info!("scanning {:?} for libraries", dir,);

        for entry in
            read_dir(dir).map_err(|_| Error(ErrorOrigin::Inventory, ErrorKind::UnableToReadDir))?
        {
            let entry = entry
                .map_err(|_| Error(ErrorOrigin::Inventory, ErrorKind::UnableToReadDirEntry))?;
            if let Some(true) = entry.file_name().to_str().map(|n| n.contains(filter)) {
                self.load(entry.path());
            }
        }

        Ok(self)
    }

    /// Adds a library directory to the inventory
    ///
    /// # Safety
    ///
    /// Same as previous functions - compiler can not guarantee the safety of
    /// third party library implementations.
    pub fn add_dir(&mut self, dir: PathBuf) -> Result<&mut Self> {
        self.add_dir_filtered(dir, "")
    }

    /// Adds a single library to the inventory
    ///
    /// # Safety
    ///
    /// Same as previous functions - compiler can not guarantee the safety of
    /// third party library implementations.
    pub fn load(&mut self, path: PathBuf) -> &mut Self {
        Loadable::load_append(&path, &mut self.connectors).ok();
        Loadable::load_append(&path, &mut self.os_layers).ok();
        self
    }

    /// Returns the names of all currently available connectors that can be used.
    pub fn available_connectors(&self) -> Vec<String> {
        self.connectors
            .iter()
            .map(|c| c.loader.ident().to_string())
            .collect::<Vec<_>>()
    }

    /// Returns the names of all currently available os plugins that can be used.
    pub fn available_os(&self) -> Vec<String> {
        self.os_layers
            .iter()
            .map(|c| c.loader.ident().to_string())
            .collect::<Vec<_>>()
    }

    /// Returns the help string of the given Connector.
    ///
    /// This function returns an error in case the Connector was not found or does not implement the help feature.
    pub fn connector_help(&self, name: &str) -> Result<String> {
        Self::help_internal(&self.connectors, name)
    }

    /// Returns the help string of the given Os Plugin.
    ///
    /// This function returns an error in case the Os Plugin was not found or does not implement the help feature.
    pub fn os_help(&self, name: &str) -> Result<String> {
        Self::help_internal(&self.os_layers, name)
    }

    fn help_internal<T: Loadable>(libs: &[LibInstance<T>], name: &str) -> Result<String> {
        let lib = libs
            .iter()
            .find(|c| c.loader.ident() == name)
            .ok_or_else(|| {
                error!(
                    "unable to find plugin with name '{}'. available `{}` plugins are: {}",
                    name,
                    T::plugin_type(),
                    libs.iter()
                        .map(|c| c.loader.ident().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                Error(ErrorOrigin::Inventory, ErrorKind::PluginNotFound)
            })?;

        lib.loader.help()
    }

    /// Returns a list of all available targets of the connector.
    ///
    /// This function returns an error in case the connector does not implement this feature.
    pub fn connector_target_list(&self, name: &str) -> Result<Vec<TargetInfo>> {
        let lib = self
            .connectors
            .iter()
            .find(|c| c.loader.ident() == name)
            .ok_or_else(|| {
                error!(
                    "unable to find connector with name '{}'. available connectors are: {}",
                    name,
                    self.available_connectors().join(", "),
                );
                Error(ErrorOrigin::Inventory, ErrorKind::PluginNotFound)
            })?;

        lib.loader.target_list()
    }

    /// Creates a new Connector / OS builder.
    ///
    /// # Examples
    ///
    /// Create a connector:
    /// ```no_run
    /// use memflow::plugins::Inventory;
    ///
    /// let inventory = Inventory::scan();
    /// let os = inventory
    ///   .builder()
    ///   .connector("qemu_procfs")
    ///   .build();
    /// ```
    ///
    /// Create a Connector with arguments:
    /// ```no_run
    /// use memflow::plugins::{Inventory, Args};
    ///
    /// let inventory = Inventory::scan();
    /// let os = inventory
    ///   .builder()
    ///   .connector("qemu_procfs")
    ///   .args(str::parse("vm-win10").unwrap())
    ///   .build();
    /// ```
    ///
    /// Create a Connector and OS with arguments:
    /// ```no_run
    /// use memflow::plugins::{Inventory, Args};
    ///
    /// let inventory = Inventory::scan();
    /// let os = inventory
    ///   .builder()
    ///   .connector("qemu_procfs")
    ///   .args(str::parse("vm-win10").unwrap())
    ///   .os("win10")
    ///   .build();
    /// ```
    ///
    /// Create a OS without a connector and arguments:
    /// ```no_run
    /// use memflow::plugins::Inventory;
    ///
    /// let inventory = Inventory::scan();
    /// let os = inventory
    ///   .builder()
    ///   .os("native")
    ///   .build();
    /// ```
    pub fn builder(&self) -> BuilderEmpty {
        BuilderEmpty { inventory: self }
    }

    /// Tries to create a new instance for the library with the given name.
    /// The instance will be initialized with the args provided to this call.
    ///
    /// In case no library could be found this will throw an `Error::Library`.
    ///
    /// # Safety
    ///
    /// This function assumes all libraries were loaded with appropriate safety
    /// checks in place. This function is safe, but can crash if previous checks
    /// fail.
    ///
    /// # Examples
    ///
    /// Creating a connector instance:
    /// ```no_run
    /// use memflow::plugins::{Inventory, Args};
    ///
    /// let inventory = Inventory::scan_path("./").unwrap();
    /// let connector = inventory
    ///     .create_connector("coredump", None, None)
    ///     .unwrap();
    /// ```
    ///
    /// Defining a dynamically loaded connector:
    /// ```
    /// use memflow::error::Result;
    /// use memflow::types::size;
    /// use memflow::dummy::DummyMemory;
    /// use memflow::plugins::ConnectorArgs;
    /// use memflow::derive::connector;
    /// use memflow::mem::phys_mem::*;
    ///
    /// #[connector(name = "dummy_conn")]
    /// pub fn create_connector(_args: &ConnectorArgs) -> Result<DummyMemory> {
    ///     Ok(DummyMemory::new(size::mb(16)))
    /// }
    /// ```
    pub fn create_connector(
        &self,
        name: &str,
        input: ConnectorInputArg,
        args: Option<&ConnectorArgs>,
    ) -> Result<ConnectorInstanceArcBox<'static>> {
        Self::create_internal(&self.connectors, name, input, args)
    }

    /// Create OS instance
    ///
    /// This is the primary way of building a OS instance.
    ///
    /// # Arguments
    ///
    /// * `name` - name of the target OS
    /// * `input` - connector to be passed to the OS
    /// * `args` - arguments to be passed to the OS
    ///
    /// # Examples
    ///
    /// Creating a OS instance with custom arguments
    /// ```
    /// use memflow::plugins::{Inventory, ConnectorArgs};
    ///
    /// # let mut inventory = Inventory::scan();
    /// # let paths = std::fs::read_dir("../target/").unwrap();
    /// # for path in paths {
    /// #     match path.unwrap().file_name().to_str() {
    /// #         Some("release") | Some("debug") | None => {},
    /// #         Some(x) => {
    /// #             inventory.add_dir_filtered(format!("../target/{}/release/deps", x).into(), "ffi").ok();
    /// #             inventory.add_dir_filtered(format!("../target/{}/debug/deps", x).into(), "ffi").ok();
    /// #         }
    /// #     }
    /// # }
    /// # inventory.add_dir_filtered("../target/release/deps".into(), "ffi").ok();
    /// # inventory.add_dir_filtered("../target/debug/deps".into(), "ffi").ok();
    /// let args = str::parse(":4m").unwrap();
    /// let os = inventory.create_os("dummy", None, Some(&args))
    ///     .unwrap();
    /// std::mem::drop(os);
    /// ```
    pub fn create_os(
        &self,
        name: &str,
        input: OsInputArg,
        args: Option<&OsArgs>,
    ) -> Result<OsInstanceArcBox<'static>> {
        Self::create_internal(&self.os_layers, name, input, args)
    }

    fn create_internal<T: Loadable>(
        libs: &[LibInstance<T>],
        name: &str,
        input: T::InputArg,
        args: Option<&T::ArgsType>,
    ) -> Result<T::Instance> {
        let lib = libs
            .iter()
            .find(|c| c.loader.ident() == name)
            .ok_or_else(|| {
                error!(
                    "unable to find plugin with name '{}'. available `{}` plugins are: {}",
                    name,
                    T::plugin_type(),
                    libs.iter()
                        .map(|c| c.loader.ident().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                Error(ErrorOrigin::Inventory, ErrorKind::PluginNotFound)
            })?;

        info!(
            "attempting to load `{}` type plugin `{}` from `{}`",
            T::plugin_type(),
            lib.loader.ident(),
            lib.path.to_string_lossy(),
        );

        lib.loader.instantiate(lib.library.clone(), input, args)
    }

    /// Sets the maximum logging level in all plugins and updates the
    /// internal [`PluginLogger`] in each plugin instance.
    pub fn set_max_log_level(&self, level: LevelFilter) {
        log::set_max_level(level);
        self.update_max_log_level()
    }

    fn update_max_log_level(&self) {
        let level = log::max_level();

        self.connectors
            .iter()
            .map(|c| c.library.as_ref())
            .chain(self.os_layers.iter().map(|o| o.library.as_ref()))
            .filter_map(|l| *l)
            .filter_map(LibContext::try_get_logger)
            .for_each(|l| l.on_level_change(level));
    }
}

pub enum BuildStep<'a> {
    Connector {
        name: &'a str,
        args: Option<ConnectorArgs>,
    },
    Os {
        name: &'a str,
        args: Option<OsArgs>,
    },
}

impl<'a> BuildStep<'a> {
    /// Parse input string and construct steps for building a connector.
    ///
    /// Name and arguments are separated by `:`, for example:
    ///
    /// `kvm:5`, or `qemu:win10:memmap=map`.
    pub fn new_connector(input: &'a str) -> Result<Self> {
        let (name, args) = input.split_once(":").unwrap_or((input, ""));

        Ok(Self::Connector {
            name,
            args: if args.is_empty() {
                None
            } else {
                Some(str::parse(args)?)
            },
        })
    }

    /// Parse input string and construct steps for building an OS.
    ///
    /// Name and arguments are separated by `:`, for example:
    ///
    /// `win32`, or `win32::dtb=0xdeadbeef`.
    pub fn new_os(input: &'a str) -> Result<Self> {
        let (name, args) = input.split_once(":").unwrap_or((input, ""));

        Ok(Self::Os {
            name,
            args: if args.is_empty() {
                None
            } else {
                Some(str::parse(args)?)
            },
        })
    }

    /// Validate whether the next build step is compatible with the current one.
    pub fn validate_next(&self, next: &Self) -> bool {
        !matches!(
            (self, next),
            (BuildStep::Connector { .. }, BuildStep::Connector { .. })
                | (BuildStep::Os { .. }, BuildStep::Os { .. })
        )
    }
}

fn builder_from_args<'a>(
    connectors: impl Iterator<Item = (usize, &'a str)>,
    os_layers: impl Iterator<Item = (usize, &'a str)>,
) -> Result<Vec<BuildStep<'a>>> {
    let mut layers = connectors
        .map(|(i, a)| BuildStep::new_connector(a).map(|a| (i, a)))
        .chain(os_layers.map(|(i, a)| BuildStep::new_os(a).map(|a| (i, a))))
        .collect::<Result<Vec<_>>>()?;

    layers.sort_by(|(a, _), (b, _)| a.cmp(b));

    if layers.windows(2).any(|w| !w[0].1.validate_next(&w[1].1)) {
        return Err(Error(ErrorOrigin::Other, ErrorKind::ArgValidation));
    }

    Ok(layers.into_iter().map(|(_, s)| s).collect())
}

/// Precompiled connector chain.
///
/// Use this with [`Inventory::builder`](Inventory::builder).
pub struct ConnectorChain<'a>(Vec<BuildStep<'a>>);

impl<'a> ConnectorChain<'a> {
    /// Build a new connector chain.
    ///
    /// Arguments are iterators of command line arguments with their position and value. The
    /// position will be used to sort them and validate whether they are in correct order.
    pub fn new(
        connectors: impl Iterator<Item = (usize, &'a str)>,
        os_layers: impl Iterator<Item = (usize, &'a str)>,
    ) -> Result<Self> {
        let steps = builder_from_args(connectors, os_layers)?;

        if !matches!(steps.last(), Some(BuildStep::Connector { .. })) {
            return Err(Error(ErrorOrigin::Other, ErrorKind::ArgValidation));
        }

        Ok(Self(steps))
    }
}

/// Precompiled os chain.
///
/// Use this with [`Inventory::builder`](Inventory::builder).
pub struct OsChain<'a>(Vec<BuildStep<'a>>);

impl<'a> OsChain<'a> {
    /// Build a new OS chain.
    ///
    /// Arguments are iterators of command line arguments with their position and value. The
    /// position will be used to sort them and validate whether they are in correct order.
    pub fn new(
        connectors: impl Iterator<Item = (usize, &'a str)>,
        os_layers: impl Iterator<Item = (usize, &'a str)>,
    ) -> Result<Self> {
        let steps = builder_from_args(connectors, os_layers)?;

        if !matches!(steps.last(), Some(BuildStep::Os { .. })) {
            return Err(Error(ErrorOrigin::Other, ErrorKind::ArgValidation));
        }

        Ok(Self(steps))
    }
}

/// BuilderEmpty is the starting builder that allows to either call `connector`, or `os`.
pub struct BuilderEmpty<'a> {
    inventory: &'a Inventory,
}

impl<'a> BuilderEmpty<'a> {
    /// Adds a Connector instance to the build chain
    ///
    /// # Arguments
    ///
    /// * `name` - name of the connector
    pub fn connector(self, name: &'a str) -> OsBuilder<'a> {
        OsBuilder {
            inventory: self.inventory,
            steps: vec![BuildStep::Connector { name, args: None }],
        }
    }

    /// Adds an OS instance to the build chain
    ///
    /// # Arguments
    ///
    /// * `name` - name of the target OS
    pub fn os(self, name: &'a str) -> ConnectorBuilder<'a> {
        ConnectorBuilder {
            inventory: self.inventory,
            steps: vec![BuildStep::Os { name, args: None }],
        }
    }

    /// Chains multiple pre-validated steps, resulting in an Os ready-to-build.
    ///
    /// # Arguments
    ///
    /// * `chain` - steps to initialize the builder with.
    pub fn os_chain(self, chain: OsChain<'a>) -> ConnectorBuilder<'a> {
        ConnectorBuilder {
            inventory: self.inventory,
            steps: chain.0,
        }
    }

    /// Chains multiple pre-validated steps, resulting in a connector ready-to-build.
    ///
    /// # Arguments
    ///
    /// * `chain` - steps to initialize the builder with.
    pub fn connector_chain(self, chain: ConnectorChain<'a>) -> OsBuilder<'a> {
        OsBuilder {
            inventory: self.inventory,
            steps: chain.0,
        }
    }
}

/// ConnectorBuilder creates a new connector instance with the previous os step as an input.
pub struct ConnectorBuilder<'a> {
    inventory: &'a Inventory,
    steps: Vec<BuildStep<'a>>,
}

impl<'a> ConnectorBuilder<'a> {
    /// Adds a Connector instance to the build chain
    ///
    /// # Arguments
    ///
    /// * `name` - name of the connector
    pub fn connector(self, name: &'a str) -> OsBuilder<'a> {
        let mut steps = self.steps;
        steps.push(BuildStep::Connector { name, args: None });
        OsBuilder {
            inventory: self.inventory,
            steps,
        }
    }

    /// Appends arguments to the previously added OS.
    ///
    /// # Arguments
    ///
    /// * `os_args` - the arguments to be passed to the previously added OS
    pub fn args(mut self, os_args: OsArgs) -> ConnectorBuilder<'a> {
        if let Some(BuildStep::Os { name: _, args }) = self.steps.iter_mut().last() {
            *args = Some(os_args);
        }
        self
    }

    /// Builds the final chain of Connectors and OS and returns the last OS.
    ///
    /// Each created connector / os instance is fed into the next os / connector instance as an argument.
    /// If any build step fails the function returns an error.
    pub fn build(self) -> Result<OsInstanceArcBox<'static>> {
        let mut connector: Option<ConnectorInstanceArcBox<'static>> = None;
        let mut os: Option<OsInstanceArcBox<'static>> = None;
        for step in self.steps.iter() {
            match step {
                BuildStep::Connector { name, args } => {
                    connector = Some(self.inventory.create_connector(name, os, args.as_ref())?);
                    os = None;
                }
                BuildStep::Os { name, args } => {
                    os = Some(self.inventory.create_os(name, connector, args.as_ref())?);
                    connector = None;
                }
            };
        }
        os.ok_or(Error(ErrorOrigin::Inventory, ErrorKind::Configuration))
    }
}

/// OsBuilder creates a new os instance with the previous connector step as an input
pub struct OsBuilder<'a> {
    inventory: &'a Inventory,
    steps: Vec<BuildStep<'a>>,
}

impl<'a> OsBuilder<'a> {
    /// Adds an OS instance to the build chain
    ///
    /// # Arguments
    ///
    /// * `name` - name of the target OS
    pub fn os(self, name: &'a str) -> ConnectorBuilder<'a> {
        let mut steps = self.steps;
        steps.push(BuildStep::Os { name, args: None });
        ConnectorBuilder {
            inventory: self.inventory,
            steps,
        }
    }

    /// Appends arguments to the previously added Connector.
    ///
    /// # Arguments
    ///
    /// * `conn_args` - the arguments to be passed to the previously added Connector
    pub fn args(mut self, conn_args: ConnectorArgs) -> OsBuilder<'a> {
        if let Some(BuildStep::Connector { name: _, args }) = self.steps.iter_mut().last() {
            *args = Some(conn_args);
        }
        self
    }

    /// Builds the final chain of Connectors and OS and returns the last Connector.
    ///
    /// Each created connector / os instance is fed into the next os / connector instance as an argument.
    /// If any build step fails the function returns an error.
    pub fn build(self) -> Result<ConnectorInstanceArcBox<'static>> {
        let mut connector: Option<ConnectorInstanceArcBox<'static>> = None;
        let mut os: Option<OsInstanceArcBox<'static>> = None;
        for step in self.steps.iter() {
            match step {
                BuildStep::Connector { name, args } => {
                    connector = Some(self.inventory.create_connector(name, os, args.as_ref())?);
                    os = None;
                }
                BuildStep::Os { name, args } => {
                    os = Some(self.inventory.create_os(name, connector, args.as_ref())?);
                    connector = None;
                }
            };
        }
        connector.ok_or(Error(ErrorOrigin::Inventory, ErrorKind::Configuration))
    }
}

/// Reference counted library instance
///
/// This stores the necessary reference counted library instance, in order to prevent the library
/// from unloading unexpectedly. This is the required safety guarantee.
#[repr(C)]
#[derive(Clone)]
pub struct LibInstance<T> {
    path: PathBuf,
    library: CArc<LibContext>,
    loader: T,
}
