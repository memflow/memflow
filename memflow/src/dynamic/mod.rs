/*!
Module containing connector and OS layer inventory related functions.

This module contains functions to interface with dynamically loaded connectors and OS layers.

This module is gated behind `dynamic` feature
*/

pub mod args;
#[doc(hidden)]
pub use args::Args;

pub mod connector;
pub use connector::{
    ConnectorBaseTable, ConnectorDescriptor, ConnectorFunctionTable, ConnectorInstance,
    ConnectorInventory, PhysicalMemoryFunctionTable, MEMFLOW_CONNECTOR_VERSION,
};

use crate::error::*;

use log::*;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use libloading::Library;

pub trait Loadable: Sized {
    type Instance;

    fn exists(&self, instances: &[LibInstance<Self>]) -> bool {
        instances
            .iter()
            .find(|i| i.loader.ident() == self.ident())
            .is_some()
    }

    fn ident(&self) -> &str;

    /// # Safety
    ///
    /// Loading third party libraries is inherently unsafe and the compiler
    /// cannot guarantee that the implementation of the library
    /// matches the one specified here. This is especially true if
    /// the loaded library implements the necessary interface manually.
    ///
    /// It is adviced to use a proc macro for defining a connector.
    unsafe fn load(library: Library) -> Result<LibInstance<Self>>;

    unsafe fn load_path(path: impl AsRef<Path>) -> Result<LibInstance<Self>> {
        let library =
            Library::new(path.as_ref()).map_err(|_| Error::Connector("unable to load library"))?;

        Self::load(library)
    }

    fn instantiate(&self, lib: Arc<Library>, args: &Args) -> Result<Self::Instance>;
}

pub struct LibInventory<T> {
    libs: Vec<LibInstance<T>>,
}

impl<F, T: Loadable<Instance = F>> LibInventory<T> {
    /// Creates a new inventory of connectors from the provided path.
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
    /// use memflow::dynamic::ConnectorInventory;
    ///
    /// let inventory = unsafe {
    ///     ConnectorInventory::scan_path("./")
    /// }.unwrap();
    /// ```
    pub unsafe fn scan_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut dir = PathBuf::default();
        dir.push(path);

        let mut ret = Self { libs: vec![] };
        ret.add_dir(dir)?;
        Ok(ret)
    }

    #[doc(hidden)]
    #[deprecated]
    pub unsafe fn with_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::scan_path(path)
    }

    /// Creates a new inventory of connectors by searching various paths.
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
    /// use memflow::dynamic::ConnectorInventory;
    ///
    /// let inventory = unsafe {
    ///     ConnectorInventory::scan()
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

        let mut ret = Self { libs: vec![] };

        for mut path in path_iter {
            path.push("memflow");
            ret.add_dir(path).ok();
        }

        if let Ok(pwd) = std::env::current_dir() {
            ret.add_dir(pwd).ok();
        }

        ret
    }

    #[doc(hidden)]
    #[deprecated]
    pub unsafe fn try_new() -> Result<Self> {
        Ok(Self::scan())
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

        info!(
            "scanning {:?} for libraries of type {}",
            dir,
            std::any::type_name::<T>()
        );

        for entry in read_dir(dir).map_err(|_| Error::IO("unable to read directory"))? {
            let entry = entry.map_err(|_| Error::IO("unable to read directory entry"))?;
            if let Ok(lib) = T::load_path(entry.path()) {
                if !lib.loader.exists(&self.libs) {
                    info!(
                        "adding library '{}': {:?}",
                        lib.loader.ident(),
                        entry.path()
                    );
                    self.libs.push(lib);
                } else {
                    debug!(
                        "skipping library '{}' because it was added already: {:?}",
                        lib.loader.ident(),
                        entry.path()
                    );
                }
            }
        }

        Ok(self)
    }

    /// Returns the names of all currently available libs that can be used
    /// when calling `create_connector` or `create_connector_default`.
    pub fn available_libs(&self) -> Vec<String> {
        self.libs
            .iter()
            .map(|c| c.loader.ident().to_string())
            .collect::<Vec<_>>()
    }

    /// Tries to create a new connector instance for the connector with the given name.
    /// The connector will be initialized with the args provided to this call.
    ///
    /// In case no connector could be found this will throw an `Error::Connector`.
    ///
    /// # Safety
    ///
    /// Loading third party libraries is inherently unsafe and the compiler
    /// cannot guarantee that the implementation of the library
    /// matches the one specified here. This is especially true if
    /// the loaded library implements the necessary interface manually.
    ///
    /// It is adviced to use a proc macro for defining a connector.
    ///
    /// # Examples
    ///
    /// Creating a connector instance:
    /// ```no_run
    /// use memflow::dynamic::{ConnectorInventory, Args};
    ///
    /// let inventory = unsafe {
    ///     ConnectorInventory::scan_path("./")
    /// }.unwrap();
    /// let connector = unsafe {
    ///     inventory.create_connector("coredump", &Args::new())
    /// }.unwrap();
    /// ```
    ///
    /// Defining a dynamically loaded connector:
    /// ```
    /// use memflow::error::Result;
    /// use memflow::types::size;
    /// use memflow::mem::dummy::DummyMemory;
    /// use memflow::dynamic::Args;
    /// use memflow::derive::connector;
    ///
    /// #[connector(name = "dummy")]
    /// pub fn create_connector(_args: &Args, _log_level: log::Level) -> Result<DummyMemory> {
    ///     Ok(DummyMemory::new(size::mb(16)))
    /// }
    /// ```
    pub unsafe fn create_connector(&self, name: &str, args: &Args) -> Result<F> {
        let lib = self
            .libs
            .iter()
            .find(|c| c.loader.ident() == name)
            .ok_or_else(|| {
                error!(
                    "unable to find connector with name '{}'. available connectors are: {}",
                    name,
                    self.libs
                        .iter()
                        .map(|c| c.loader.ident().clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                Error::Connector("connector not found")
            })?;
        lib.loader.instantiate(lib.library.clone(), args)
    }

    /// Creates a connector in the same way `create_connector` does but without any arguments provided.
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
    /// use memflow::dynamic::{ConnectorInventory, Args};
    ///
    /// let inventory = unsafe {
    ///     ConnectorInventory::scan_path("./")
    /// }.unwrap();
    /// let connector = unsafe {
    ///     inventory.create_connector_default("coredump")
    /// }.unwrap();
    /// ```
    pub unsafe fn create_connector_default(&self, name: &str) -> Result<F> {
        self.create_connector(name, &Args::default())
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct LibInstance<T> {
    library: Arc<Library>,
    loader: T,
}
