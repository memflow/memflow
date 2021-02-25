/*!
Module containing connector and OS layer inventory related functions.

This module contains functions to interface with dynamically loaded connectors and OS layers.

This module is gated behind `plugins` feature
*/

use std::prelude::v1::*;

pub mod args;
#[doc(hidden)]
pub use args::{ArgDescriptor, Args, ArgsValidator};

// cbindgen fails to properly parse this as return type
pub type OptionVoid = Option<&'static mut std::ffi::c_void>;

pub mod connector;
pub use connector::{
    ConnectorDescriptor, ConnectorFunctionTable, ConnectorInstance, LoadableConnector,
    OpaquePhysicalMemoryFunctionTable,
};
pub type ConnectorInputArg = <LoadableConnector as Loadable>::InputArg;

pub mod os;
pub use os::{LoadableOS, OSInstance, OpaqueOSFunctionTable};
pub type OSInputArg = <LoadableOS as Loadable>::InputArg;

pub(crate) mod util;
pub use util::create_bare;

pub mod virt_mem;
pub use virt_mem::{
    OpaqueVirtualMemoryFunctionTable, VirtualMemoryFunctionTable, VirtualMemoryInstance,
};
pub(crate) mod arc;
pub(crate) use arc::{CArc, COptArc};

use crate::error::{Result, *};
use crate::types::ReprCStr;

use log::*;
use std::ffi::c_void;
use std::fs::read_dir;
use std::mem::MaybeUninit;
use std::path::{Path, PathBuf};

use libloading::Library;

/// Exported memflow plugins version
pub const MEMFLOW_PLUGIN_VERSION: i32 = 1;

/// Utility typedef for better cbindgen
///
/// TODO: remove when fixed in cbindgen
pub type OptionMut<T> = Option<&'static mut T>;

/// Opaque version of `GenericBaseTable` for FFI purposes
pub type OpaqueBaseTable = GenericBaseTable<c_void>;
/// Opaque version of `GenericCloneTable` for FFI purposes
pub type OpaqueCloneTable = GenericCloneTable<c_void>;

impl Copy for OpaqueCloneTable {}

impl Clone for OpaqueCloneTable {
    fn clone(&self) -> Self {
        *self
    }
}

/// Generic function for cloning past FFI boundary
#[repr(C)]
pub struct GenericCloneTable<T: 'static> {
    pub clone: extern "C" fn(this: &T) -> OptionMut<T>,
}

impl<T: Clone> Default for GenericCloneTable<T> {
    fn default() -> Self {
        Self {
            clone: util::c_clone::<T>,
        }
    }
}

impl<T: Clone> GenericCloneTable<T> {
    pub fn into_opaque(self) -> OpaqueCloneTable {
        unsafe { std::mem::transmute(self) }
    }
}

impl Copy for OpaqueBaseTable {}

impl Clone for OpaqueBaseTable {
    fn clone(&self) -> Self {
        *self
    }
}

/// Base table for most objects that are cloneable and droppable.
#[repr(C)]
pub struct GenericBaseTable<T: 'static> {
    clone: GenericCloneTable<T>,
    pub drop: unsafe extern "C" fn(this: &mut T),
}

impl<T: Clone> Default for &'static GenericBaseTable<T> {
    fn default() -> Self {
        &GenericBaseTable {
            clone: GenericCloneTable {
                clone: util::c_clone::<T>,
            },
            drop: util::c_drop::<T>,
        }
    }
}

impl<T: Clone> GenericBaseTable<T> {
    pub fn as_opaque(&self) -> &OpaqueBaseTable {
        unsafe { &*(self as *const Self as *const OpaqueBaseTable) }
    }
}

/// Describes a FFI safe option
#[repr(C)]
pub enum COption<T> {
    None,
    Some(T),
}

impl<T> From<Option<T>> for COption<T> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            None => Self::None,
            Some(t) => Self::Some(t),
        }
    }
}

impl<T> From<COption<T>> for Option<T> {
    fn from(opt: COption<T>) -> Self {
        match opt {
            COption::None => None,
            COption::Some(t) => Some(t),
        }
    }
}

#[repr(C)]
pub struct PluginDescriptor<T: Loadable> {
    /// The plugin api version for when the plugin was built.
    /// This has to be set to `MEMFLOW_PLUGIN_VERSION` of memflow.
    ///
    /// If the versions mismatch the inventory will refuse to load.
    pub plugin_version: i32,

    /// The name of the plugin.
    /// This name will be used when loading a plugin from the inventory.
    ///
    /// During plugin discovery, the export suffix has to match this name being capitalized
    pub name: &'static str,

    /// The version of the connector.
    /// If multiple connectors are installed the latest is picked.
    pub version: &'static str,

    /// The description of the connector.
    pub description: &'static str,

    /// Retrieve a help string from the connector.
    //pub help: extern "C" fn(&ReprCStr) -> (),

    /// Retrieve a list of available targets for this connector
    // TODO:

    /// Create instance of the plugin
    pub create: extern "C" fn(&ReprCStr, T::CInputArg, i32, &mut MaybeUninit<T::Instance>) -> i32,
}

/// Defines a common interface for loadable plugins
pub trait Loadable: Sized {
    type Instance;
    type InputArg;
    type CInputArg;

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

    fn load(library: &CArc<Library>, export: &str) -> Result<LibInstance<Self>> {
        // find os descriptor
        let descriptor = unsafe {
            library
                .as_ref()
                .get::<*mut PluginDescriptor<Self>>(format!("{}\0", export).as_bytes())
                .map_err(|_| Error(ErrorOrigin::Inventory, ErrorKind::MemflowExportsNotFound))?
                .read()
        };

        // check version
        if descriptor.plugin_version != MEMFLOW_PLUGIN_VERSION {
            warn!(
                "{} {} has a different version. version {} required, found {}.",
                Self::plugin_type(),
                descriptor.name,
                MEMFLOW_PLUGIN_VERSION,
                descriptor.plugin_version
            );
            Err(Error(ErrorOrigin::Inventory, ErrorKind::VersionMismatch))
        } else {
            Ok(LibInstance {
                library: library.clone(),
                loader: Self::new(descriptor),
            })
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
        let library = Library::new(path.as_ref())
            .map_err(|_| Error(ErrorOrigin::Inventory, ErrorKind::UnableToLoadLibrary))
            .map(CArc::from)?;

        Ok(exports
            .into_iter()
            .filter_map(|e| Self::load(&library, &e).ok())
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
                    "adding library '{}': {:?}",
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

    /// Creates an `Instance` of the library
    ///
    /// This function assumes that `load` performed necessary safety checks
    /// for validity of the library.
    fn instantiate(
        &self,
        lib: Option<CArc<Library>>,
        input: Self::InputArg,
        args: &Args,
    ) -> Result<Self::Instance>;
}

/// The core of the plugin system
///
/// It scans system directories and collects valid memflow plugins. They can then be instantiated
/// easily. The reason the libraries are collected is to allow for reuse, and save performance
///
/// # Examples
///
/// Creating a OS instance, the fastest way:
///
/// ```
/// use memflow::plugins::Inventory;
/// # use memflow::error::Result;
/// # use memflow::plugins::OSInstance;
/// # fn test() -> Result<OSInstance> {
/// Inventory::build_os_simple("qemu-procfs", "win32")
/// # }
/// # test().ok();
/// ```
///
/// Creating 2 OS instances:
/// ```
/// use memflow::plugins::{Inventory, Args};
/// # use memflow::error::Result;
/// # fn test() -> Result<()> {
///
/// let inventory = Inventory::scan();
///
/// let windows = inventory.create_os_simple("qemu-procfs", "win32")?;
/// let system = inventory.create_os("pseudo-system", None, &Args::default())?;
/// # Ok(())
/// # }
/// # test().ok();
/// ```
pub struct Inventory {
    connectors: Vec<LibInstance<connector::LoadableConnector>>,
    os_layers: Vec<LibInstance<os::LoadableOS>>,
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
                .map(|p| std::env::split_paths(p))
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

    /// Returns the names of all currently available os_layers that can be used.
    pub fn available_os_layers(&self) -> Vec<String> {
        self.os_layers
            .iter()
            .map(|c| c.loader.ident().to_string())
            .collect::<Vec<_>>()
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
    ///     .create_connector("coredump", None, &Args::new())
    ///     .unwrap();
    /// ```
    ///
    /// Defining a dynamically loaded connector:
    /// ```
    /// use memflow::error::Result;
    /// use memflow::types::size;
    /// use memflow::dummy::DummyMemory;
    /// use memflow::plugins::Args;
    /// use memflow::derive::connector;
    /// use memflow::mem::PhysicalMemory;
    ///
    /// #[connector(name = "dummy_conn")]
    /// pub fn create_connector(_args: &Args, _log_level: log::Level) ->
    ///     Result<impl PhysicalMemory + Clone> {
    ///     Ok(DummyMemory::new(size::mb(16)))
    /// }
    /// ```
    pub fn create_connector(
        &self,
        name: &str,
        input: ConnectorInputArg,
        args: &Args,
    ) -> Result<ConnectorInstance> {
        Self::create_internal(&self.connectors, name, input, args)
    }

    fn create_internal<T: Loadable>(
        libs: &[LibInstance<T>],
        name: &str,
        input: T::InputArg,
        args: &Args,
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
            "attempting to load `{}` type plugin {}",
            T::plugin_type(),
            lib.loader.ident()
        );

        lib.loader
            .instantiate(Some(lib.library.clone()), input, args)
    }

    /// Creates an instance in the same way `instantiate` does but without any arguments provided.
    ///
    /// # Safety
    ///
    /// See the above safety section.
    /// This function essentially just wraps the above function.
    ///
    /// # Examples
    ///
    /// Creating a connector instance:
    /// ```
    /// use memflow::plugins::{Inventory, Args};
    ///
    /// # let mut inventory = Inventory::scan();
    /// # inventory.add_dir_filtered("../target/release/deps".into(), "ffi").ok();
    /// # inventory.add_dir_filtered("../target/debug/deps".into(), "ffi").ok();
    /// let connector = inventory.create_connector_default("dummy")
    ///     .unwrap();
    /// ```
    pub fn create_connector_default(&self, name: &str) -> Result<ConnectorInstance> {
        self.create_connector(name, None, &Args::default())
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
    /// use memflow::plugins::{Inventory, Args};
    ///
    /// # let mut inventory = Inventory::scan();
    /// # inventory.add_dir_filtered("../target/release/deps".into(), "ffi").ok();
    /// # inventory.add_dir_filtered("../target/debug/deps".into(), "ffi").ok();
    /// let args = Args::parse("4m").unwrap();
    /// let connector = inventory.create_os("dummy", None, &args)
    ///     .unwrap();
    /// ```
    pub fn create_os(&self, name: &str, input: OSInputArg, args: &Args) -> Result<OSInstance> {
        Self::create_internal(&self.os_layers, name, input, args)
    }

    /// Create a connector and OS in one go
    ///
    /// This will build a connector, and then feed it to `create_os` function
    pub fn create_conn_os_combo(
        &self,
        conn_name: &str,
        conn_args: &Args,
        os_name: &str,
        os_args: &Args,
    ) -> Result<OSInstance> {
        let conn = self.create_connector(conn_name, None, conn_args)?;
        self.create_os(os_name, Some(conn), os_args)
    }

    /// Simple way of creating a OS in one go
    ///
    /// This function accepts no arguments for the sake of simplicity. It is advised to use
    /// `create_conn_os_combo` if passing arguments is necessary.
    pub fn create_os_simple(&self, conn_name: &str, os_name: &str) -> Result<OSInstance> {
        let conn = self.create_connector_default(conn_name)?;
        self.create_os(os_name, Some(conn), &Args::default())
    }

    /// Create a connector and OS in one go, statically
    ///
    /// This is essentially the same as `create_conn_os_combo`, but does not require access to the
    /// inventory. Instead, it finds the libraries on its own. This is less efficient, if creating
    /// multiple connector instances
    pub fn build_conn_os_combo(
        conn_name: &str,
        conn_args: &Args,
        os_name: &str,
        os_args: &Args,
    ) -> Result<OSInstance> {
        let inv = Self::scan();
        inv.create_conn_os_combo(conn_name, conn_args, os_name, os_args)
    }

    /// Create a OS instance in the most simple way, statically
    ///
    /// This is the same as `creat_os_simple`, but does not require inventory. This is the
    /// shortest, although least flexible, way of building a OS.
    pub fn build_os_simple(conn_name: &str, os_name: &str) -> Result<OSInstance> {
        let inv = Self::scan();
        inv.create_os_simple(conn_name, os_name)
    }
}

/// Reference counted library instance
///
/// This stores the necessary reference counted library instance, in order to prevent the library
/// from unloading unexpectedly. This is the required safety guarantee.
#[repr(C)]
#[derive(Clone)]
pub struct LibInstance<T> {
    library: CArc<Library>,
    loader: T,
}
