/*!
Module containing connector and OS layer inventory related functions.

This module contains functions to interface with dynamically loaded connectors and OS layers.

This module is gated behind `plugins` feature
*/

use std::prelude::v1::*;

pub mod args;
#[doc(hidden)]
pub use args::Args;

// cbindgen fails to properly parse this as return type
pub type OptionVoid = Option<&'static mut std::ffi::c_void>;

pub mod connector;
pub use connector::{
    ConnectorDescriptor, ConnectorFunctionTable, ConnectorInstance, LoadableConnector,
    OpaquePhysicalMemoryFunctionTable,
};
pub type ConnectorInputArg = <LoadableConnector as Loadable>::InputArg;

pub mod os;
pub use os::{KernelInstance, LoadableOS, OpaqueKernelFunctionTable};
pub type OSInputArg = <LoadableOS as Loadable>::InputArg;

pub(crate) mod util;
pub mod virt_mem;
pub use virt_mem::{
    OpaqueVirtualMemoryFunctionTable, VirtualMemoryFunctionTable, VirtualMemoryInstance,
};
pub(crate) mod arc;
pub(crate) use arc::{CArc, COptArc};

use crate::error::{Result, *};

use log::*;
use std::ffi::c_void;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

use libloading::Library;

/// Exported memflow plugins version
pub const MEMFLOW_PLUGIN_VERSION: i32 = 8;

pub type OptionMut<T> = Option<&'static mut T>;

pub type OpaqueBaseTable = GenericBaseTable<c_void>;

impl Copy for OpaqueBaseTable {}

impl Clone for OpaqueBaseTable {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct GenericBaseTable<T: 'static> {
    pub clone: extern "C" fn(this: &T) -> OptionMut<T>,
    pub drop: unsafe extern "C" fn(this: &mut T),
}

impl<T: Clone> Default for GenericBaseTable<T> {
    fn default() -> Self {
        Self {
            clone: util::c_clone::<T>,
            drop: util::c_drop::<T>,
        }
    }
}

impl<T: Clone> GenericBaseTable<T> {
    pub fn into_opaque(self) -> OpaqueBaseTable {
        unsafe { std::mem::transmute(self) }
    }
}

/// Defines a common interface for loadable plugins
pub trait Loadable: Sized {
    type Instance;
    type InputArg;

    fn exists(&self, instances: &[LibInstance<Self>]) -> bool {
        instances.iter().any(|i| i.loader.ident() == self.ident())
    }

    fn ident(&self) -> &str;

    /// # Safety
    ///
    /// Loading third party libraries is inherently unsafe and the compiler
    /// cannot guarantee that the implementation of the library
    /// matches the one specified here. This is especially true if
    /// the loaded library implements the necessary interface manually.
    ///
    /// It is adviced to use a provided proc macro to define a valid library.
    unsafe fn load(library: &CArc<Library>, path: impl AsRef<Path>) -> Result<LibInstance<Self>>;

    /// # Safety
    ///
    /// Loading third party libraries is inherently unsafe and the compiler
    /// cannot guarantee that the implementation of the library matches the one
    /// specified here.
    unsafe fn load_into(
        lib: &CArc<Library>,
        path: impl AsRef<Path>,
        out: &mut Vec<LibInstance<Self>>,
    ) {
        if let Ok(lib) = Self::load(lib, &path) {
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
            }
        }
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

pub struct Inventory {
    connectors: Vec<LibInstance<connector::LoadableConnector>>,
    os_layers: Vec<LibInstance<os::LoadableOS>>,
}

impl Inventory {
    /// Creates a new inventory of plugins from the provided path.
    /// The path has to be a valid directory or the function will fail with an `Error::IO` error.
    ///
    /// # Safety
    ///
    /// Loading third party libraries is inherently unsafe and the compiler
    /// cannot guarantee that the implementation of the library
    /// matches the one specified here. This is especially true if
    /// the loaded library implements the necessary interface manually.
    ///
    /// # Examples
    ///
    /// Creating a inventory:
    /// ```
    /// use memflow::plugins::Inventory;
    ///
    /// let inventory = unsafe {
    ///     Inventory::scan_path("./")
    /// }.unwrap();
    /// ```
    pub unsafe fn scan_path<P: AsRef<Path>>(path: P) -> Result<Self> {
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
    /// # Safety
    ///
    /// Loading third party libraries is inherently unsafe and the compiler
    /// cannot guarantee that the implementation of the library
    /// matches the one specified here. This is especially true if
    /// the loaded library implements the necessary interface manually.
    ///
    /// # Examples
    ///
    /// Creating an inventory:
    /// ```
    /// use memflow::plugins::Inventory;
    ///
    /// let inventory = unsafe {
    ///     Inventory::scan()
    /// };
    /// ```
    pub unsafe fn scan() -> Self {
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

        let path_iter = path_iter.chain(
            dirs::home_dir()
                .map(|dir| dir.join(".local").join("lib"))
                .into_iter(),
        );

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
    /// # Safety
    ///
    /// Same as previous functions - compiler can not guarantee the safety of
    /// third party library implementations.
    pub unsafe fn add_dir(&mut self, dir: PathBuf) -> Result<&mut Self> {
        if !dir.is_dir() {
            return Err(Error::IO("invalid path argument"));
        }

        info!("scanning {:?} for libraries", dir,);

        for entry in read_dir(dir).map_err(|_| Error::IO("unable to read directory"))? {
            let entry = entry.map_err(|_| Error::IO("unable to read directory entry"))?;
            if let Ok(lib) = Library::new(entry.path())
                .map_err(|_| Error::Connector("unable to load library"))
                .map(CArc::from)
            {
                Loadable::load_into(&lib, entry.path(), &mut self.connectors);
                Loadable::load_into(&lib, entry.path(), &mut self.os_layers);
            }
        }

        Ok(self)
    }

    /// Returns the names of all currently available connectors that can be used
    /// when calling `instantiate` or `create_connector_default`.
    pub fn available_connectors(&self) -> Vec<String> {
        self.connectors
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
    /// let inventory = unsafe {
    ///     Inventory::scan_path("./")
    /// }.unwrap();
    /// let connector = unsafe {
    ///     inventory.create_connector("coredump", None, &Args::new())
    /// }.unwrap();
    /// ```
    ///
    /// Defining a dynamically loaded connector:
    /// ```
    /// use memflow::error::Result;
    /// use memflow::types::size;
    /// use memflow::mem::dummy::DummyMemory;
    /// use memflow::plugins::Args;
    /// use memflow::derive::connector;
    /// use memflow::mem::PhysicalMemory;
    ///
    /// #[connector(name = "dummy")]
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
                    std::any::type_name::<T>(),
                    libs.iter()
                        .map(|c| c.loader.ident().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                Error::Connector("plugin not found")
            })?;
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
    /// ```no_run
    /// use memflow::plugins::{Inventory, Args};
    ///
    /// let inventory = unsafe {
    ///     Inventory::scan_path("./")
    /// }.unwrap();
    /// let connector = unsafe {
    ///     inventory.create_connector_default("coredump")
    /// }.unwrap();
    /// ```
    pub fn create_connector_default(&self, name: &str) -> Result<ConnectorInstance> {
        self.create_connector(name, None, &Args::default())
    }

    pub fn create_os(&self, name: &str, input: OSInputArg, args: &Args) -> Result<KernelInstance> {
        Self::create_internal(&self.os_layers, name, input, args)
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct LibInstance<T> {
    library: CArc<Library>,
    loader: T,
}
