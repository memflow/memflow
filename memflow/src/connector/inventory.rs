/*!
Connector inventory interface.
*/

use crate::error::{Error, Result};
use crate::mem::{CloneablePhysicalMemory, PhysicalMemoryBox};

use super::ConnectorArgs;

use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use log::{debug, error, info, warn};

use libloading::Library;

/// Exported memflow connector version
pub const MEMFLOW_CONNECTOR_VERSION: i32 = 5;

/// Type of a single connector instance
pub type ConnectorType = PhysicalMemoryBox;

/// Describes a connector
pub struct ConnectorDescriptor {
    /// The connector inventory api version for when the connector was built.
    /// This has to be set to `MEMFLOW_CONNECTOR_VERSION` of memflow.
    ///
    /// If the versions mismatch the inventory will refuse to load.
    pub connector_version: i32,

    /// The name of the connector.
    /// This name will be used when loading a connector from a connector inventory.
    pub name: &'static str,

    /// The factory function for the connector.
    /// Calling this function will produce new connector instances.
    pub factory: extern "C" fn(args: &ConnectorArgs) -> Result<ConnectorType>,
}

/// Holds an inventory of available connectors.
pub struct ConnectorInventory {
    connectors: Vec<Connector>,
}

impl ConnectorInventory {
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
    /// use memflow::connector::ConnectorInventory;
    ///
    /// let inventory = unsafe {
    ///     ConnectorInventory::scan_path("./")
    /// }.unwrap();
    /// ```
    pub unsafe fn scan_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut dir = PathBuf::default();
        dir.push(path);

        let mut ret = Self { connectors: vec![] };
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
    /// use memflow::connector::ConnectorInventory;
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

        #[cfg(unix)]
        let path_iter = path_iter.chain(
            dirs::home_dir()
                .map(|dir| dir.join(".local").join("lib"))
                .into_iter(),
        );

        #[cfg(not(unix))]
        let path_iter = path_iter.chain(dirs::document_dir().into_iter());

        let mut ret = Self { connectors: vec![] };

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

        info!("scanning {:?} for connectors", dir);

        for entry in read_dir(dir).map_err(|_| Error::IO("unable to read directory"))? {
            let entry = entry.map_err(|_| Error::IO("unable to read directory entry"))?;
            if let Ok(connector) = Connector::try_with(entry.path()) {
                if self
                    .connectors
                    .iter()
                    .find(|c| connector.name == c.name)
                    .is_none()
                {
                    info!("adding connector '{}': {:?}", connector.name, entry.path());
                    self.connectors.push(connector);
                } else {
                    debug!(
                        "skipping connector '{}' because it was added already: {:?}",
                        connector.name,
                        entry.path()
                    );
                }
            }
        }

        Ok(self)
    }

    /// Returns the names of all currently available connectors that can be used
    /// when calling `create_connector` or `create_connector_default`.
    pub fn available_connectors(&self) -> Vec<String> {
        self.connectors
            .iter()
            .map(|c| c.name.clone())
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
    /// use memflow::connector::{ConnectorInventory, ConnectorArgs};
    ///
    /// let inventory = unsafe {
    ///     ConnectorInventory::scan_path("./")
    /// }.unwrap();
    /// let connector = unsafe {
    ///     inventory.create_connector("coredump", &ConnectorArgs::new())
    /// }.unwrap();
    /// ```
    ///
    /// Defining a dynamically loaded connector:
    /// ```
    /// use memflow::error::Result;
    /// use memflow::types::size;
    /// use memflow::mem::dummy::DummyMemory;
    /// use memflow::connector::ConnectorArgs;
    /// use memflow_derive::connector;
    ///
    /// #[connector(name = "dummy")]
    /// pub fn create_connector(_args: &ConnectorArgs) -> Result<DummyMemory> {
    ///     Ok(DummyMemory::new(size::mb(16)))
    /// }
    /// ```
    pub unsafe fn create_connector(
        &self,
        name: &str,
        args: &ConnectorArgs,
    ) -> Result<ConnectorInstance> {
        let connector = self
            .connectors
            .iter()
            .find(|c| c.name == name)
            .ok_or_else(|| {
                error!(
                    "unable to find connector with name '{}'. available connectors are: {}",
                    name,
                    self.connectors
                        .iter()
                        .map(|c| c.name.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                Error::Connector("connector not found")
            })?;
        connector.create(args)
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
    /// use memflow::connector::{ConnectorInventory, ConnectorArgs};
    ///
    /// let inventory = unsafe {
    ///     ConnectorInventory::scan_path("./")
    /// }.unwrap();
    /// let connector = unsafe {
    ///     inventory.create_connector_default("coredump")
    /// }.unwrap();
    /// ```
    pub unsafe fn create_connector_default(&self, name: &str) -> Result<ConnectorInstance> {
        self.create_connector(name, &ConnectorArgs::default())
    }
}

/// Stores a connector library instance.
///
/// # Examples
///
/// Creating a connector instance:
/// ```no_run
/// use memflow::connector::{Connector, ConnectorArgs};
///
/// let connector_lib = unsafe {
///     Connector::try_with("./connector.so")
/// }.unwrap();
///
/// let connector = unsafe {
///     connector_lib.create(&ConnectorArgs::new())
/// }.unwrap();
/// ```
#[derive(Clone)]
pub struct Connector {
    _library: Arc<Library>,
    name: String,
    factory: extern "C" fn(args: &ConnectorArgs) -> Result<ConnectorType>,
}

impl Connector {
    /// Tries to initialize a connector from a `Path`.
    /// The path must point to a valid dynamic library that implements
    /// the memflow inventory interface.
    ///
    /// If the connector does not contain the necessary exports or the version does
    /// not match the current api version this function will return an `Error::Connector`.
    ///
    /// # Safety
    ///
    /// Loading third party libraries is inherently unsafe and the compiler
    /// cannot guarantee that the implementation of the library
    /// matches the one specified here. This is especially true if
    /// the loaded library implements the necessary interface manually.
    pub unsafe fn try_with<P: AsRef<Path>>(path: P) -> Result<Self> {
        let library =
            Library::new(path.as_ref()).map_err(|_| Error::Connector("unable to load library"))?;

        let desc = library
            .get::<*mut ConnectorDescriptor>(b"MEMFLOW_CONNECTOR\0")
            .map_err(|_| Error::Connector("connector descriptor not found"))?
            .read();

        if desc.connector_version != MEMFLOW_CONNECTOR_VERSION {
            warn!(
                "connector {:?} has a different version. version {} required, found {}.",
                path.as_ref(),
                MEMFLOW_CONNECTOR_VERSION,
                desc.connector_version
            );
            return Err(Error::Connector("connector version mismatch"));
        }

        Ok(Self {
            _library: Arc::new(library),
            name: desc.name.to_string(),
            factory: desc.factory,
        })
    }

    /// Creates a new connector instance from this library.
    /// The connector is initialized with the arguments provided to this function.
    ///
    /// # Safety
    ///
    /// Loading third party libraries is inherently unsafe and the compiler
    /// cannot guarantee that the implementation of the library
    /// matches the one specified here. This is especially true if
    /// the loaded library implements the necessary interface manually.
    ///
    /// It is adviced to use a proc macro for defining a connector.
    pub unsafe fn create(&self, args: &ConnectorArgs) -> Result<ConnectorInstance> {
        let connector_res = (self.factory)(args);

        if let Err(err) = connector_res {
            debug!("{}", err)
        }

        // We do not want to return error with data from the shared library
        // that may get unloaded before it gets displayed
        let instance = connector_res?;

        Ok(ConnectorInstance {
            _library: self._library.clone(),
            instance,
        })
    }
}

/// Describes initialized connector instance
///
/// This structure is returned by `Connector`. It is needed to maintain reference
/// counts to the loaded connector library.
#[derive(Clone)]
pub struct ConnectorInstance {
    instance: ConnectorType,

    /// Internal library arc.
    ///
    /// This will keep the library loaded in memory as long as the connector instance is alive.
    /// This has to be the last member of the struct so the library will be unloaded _after_
    /// the instance is destroyed.
    ///
    /// If the library is unloaded prior to the instance this will lead to a SIGSEGV.
    _library: Arc<Library>,
}

impl std::ops::Deref for ConnectorInstance {
    type Target = dyn CloneablePhysicalMemory;

    fn deref(&self) -> &Self::Target {
        &*self.instance
    }
}

impl std::ops::DerefMut for ConnectorInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.instance
    }
}
