use core::cmp::Reverse;
use std::path::{Path, PathBuf};

use cglue::arc::CArc;
use chrono::{DateTime, Local, NaiveDateTime};
use libloading::Library;
use log::{debug, info, warn, LevelFilter};
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, ErrorKind, ErrorOrigin, Result},
    plugins::{
        plugin_analyzer::{self, PluginDescriptorInfo},
        plugin_architecture, plugin_extension, plugin_file_type, plugins_path,
        MEMFLOW_PLUGIN_VERSION,
    },
};

use super::{
    plugin_analyzer::PluginKind, BuilderEmpty, ConnectorArgs, ConnectorInputArg,
    ConnectorInstanceArcBox, LibContext, Loadable, LoadableConnector, LoadableOs, OsArgs,
    OsInputArg, OsInstanceArcBox, TargetInfo,
};

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
/// let mut inventory = Inventory::scan();
/// inventory
///   .builder()
///   .connector("qemu")
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
/// let mut inventory = Inventory::scan();
/// let os = inventory
///   .builder()
///   .connector("qemu")
///   .os("linux")
///   .connector("qemu")
///   .os("win32")
///   .build();
/// # Ok(())
/// # }
/// # test().ok();
/// ```
#[derive(Clone)]
pub struct Inventory {
    plugins: Vec<PluginEntry>,
}

#[derive(Clone)]
struct PluginEntry {
    path: PathBuf,
    instance: Option<CArc<LibContext>>,
    metadata: PluginMetadata,
}

impl PluginEntry {
    pub fn load_instance(&mut self) -> Result<&CArc<LibContext>> {
        if self.instance.is_none() {
            let library = unsafe { Library::new(&self.path) }
                .map_err(|err| {
                    debug!(
                        "found plugin {:?} but could not load it: {}",
                        self.path, err
                    );
                    Error(ErrorOrigin::Inventory, ErrorKind::UnableToLoadLibrary)
                })
                .map(LibContext::from)
                .map(CArc::from)?;
            self.instance = Some(library);
        }

        Ok(self.instance.as_ref().unwrap())
    }
}

/// Metadata attached to each file
///
/// Remarks:
///
/// This structure is synced to memflow-registry / memflowup: https://github.com/memflow/memflow-registry/blob/2ff7a449324d6399d5317abfbbf8fe3e6e972689/src/storage/mod.rs#L24
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// The sha256sum of the binary file
    pub digest: String,
    /// File signature of this binary
    pub signature: String,
    /// Timestamp at which the file was added
    pub created_at: NaiveDateTime,
    /// The plugin descriptor
    pub descriptors: Vec<PluginDescriptorInfo>,
}

impl Default for Inventory {
    fn default() -> Self {
        let mut inventory = Self::empty();

        if let Ok(plugins_path) = plugins_path() {
            inventory
                .add_dir(plugins_path)
                .expect("unable to parse plugin path");
        }

        inventory.print_plugins();

        inventory
    }
}

impl Inventory {
    /// Creates an empty inventory.
    pub fn empty() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Creates a new inventory of plugins from the default plugin installation path.
    /// The default plugin installation path is also the one used by memflowup.
    ///
    /// # Examples
    ///
    /// Creating a inventory:
    /// ```
    /// use memflow::plugins::Inventory;
    ///
    /// let mut inventory = Inventory::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new inventory of plugins from the provided path.
    /// The path has to be a valid directory or the function will fail with an `Error::IO` error.
    ///
    /// # Examples
    ///
    /// Creating a inventory:
    /// ```
    /// use memflow::plugins::Inventory;
    ///
    /// let mut inventory = Inventory::scan_path("./")
    ///     .unwrap();
    /// ```
    pub fn scan_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut dir = PathBuf::default();
        dir.push(path);

        let mut inventory = Self::empty();

        inventory.add_dir(dir)?;

        inventory.print_plugins();

        Ok(inventory)
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
    /// let mut inventory = Inventory::scan();
    /// ```
    pub fn scan() -> Self {
        let mut inventory = Self::empty();

        // add default paths
        if let Ok(plugins_path) = plugins_path() {
            inventory.add_dir(plugins_path).ok();
        }

        // add environment variable MEMFLOW_PLUGIN_PATH
        let path_var = std::env::var_os("MEMFLOW_PLUGIN_PATH");
        for path in path_var
            .as_ref()
            .map(std::env::split_paths)
            .into_iter()
            .flatten()
        {
            inventory.add_dir(path).ok();
        }

        // add $MEMFLOW_PLUGIN_PATH at compile time
        if let Some(extra_plugin_paths) = option_env!("MEMFLOW_PLUGIN_PATH") {
            for path in std::env::split_paths(extra_plugin_paths) {
                inventory.add_dir(path).ok();
            }
        }

        // add current working directory
        if let Ok(pwd) = std::env::current_dir() {
            inventory.add_dir(pwd).ok();
        }

        inventory.print_plugins();

        inventory
    }

    /// Adds cargo workspace to the inventory
    ///
    /// This function is used behind the scenes by the documentation, however, is not particularly
    /// useful for end users.
    pub fn add_cargo_workspace(mut self) -> Result<Self> {
        let paths = std::fs::read_dir("../target/").map_err(|_| ErrorKind::UnableToReadDir)?;
        for path in paths {
            match path.unwrap().file_name().to_str() {
                Some("release") | Some("debug") | None => {}
                Some(x) => {
                    self.add_dir(format!("../target/{}/release/deps", x)).ok();
                    self.add_dir(format!("../target/{}/debug/deps", x)).ok();
                }
            }
        }
        self.add_dir("../target/release/deps").ok();
        self.add_dir("../target/debug/deps").ok();
        Ok(self)
    }

    fn print_plugins(&self) {
        for plugin in self.plugins.iter() {
            for descriptor in plugin.metadata.descriptors.iter() {
                info!(
                    "Found installed {:?} Plugin: {} {} ({:?})",
                    descriptor.plugin_kind, descriptor.name, descriptor.version, plugin.path,
                );
            }
        }
    }

    /// Adds a library directory to the inventory
    ///
    /// # Safety
    ///
    /// Same as previous functions - compiler can not guarantee the safety of
    /// third party library implementations.
    pub fn add_dir<P: AsRef<Path>>(&mut self, path: P) -> Result<&Self> {
        let paths = std::fs::read_dir(path.as_ref()).map_err(|err| {
            Error(ErrorOrigin::Inventory, ErrorKind::UnableToReadDir).log_error(format!(
                "unable to read plugin directory {:?}: {}",
                path.as_ref(),
                err
            ))
        })?;
        for path in paths.filter_map(|p| p.ok()) {
            if let Some(extension) = path.path().extension() {
                if extension.to_str().unwrap_or_default() == plugin_extension() {
                    self.add_file(path.path()).ok();
                }
            }
        }

        // todo: sort by date

        Ok(self)
    }

    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> Result<&Self> {
        // TODO: check if there is a .meta file and use that created_at time instead

        let mut meta_path = path.as_ref().to_path_buf();
        meta_path.set_extension("meta");

        let created_at = if meta_path.exists() {
            let content = std::fs::read_to_string(meta_path).unwrap();
            let metadata: PluginMetadata = serde_json::from_str(&content).unwrap();
            metadata.created_at
        } else {
            warn!(
                "{:?} not found, falling back via file creation date",
                meta_path
            );
            let created_at_sys = path
                .as_ref()
                .metadata()
                .map_err(|err| {
                    Error(ErrorOrigin::Inventory, ErrorKind::UnableToReadFile).log_error(format!(
                        "unable to read plugin file {:?} metadata: {}",
                        path.as_ref(),
                        err
                    ))
                })?
                .created()
                .map_err(|err| {
                    Error(ErrorOrigin::Inventory, ErrorKind::UnableToReadFile).log_error(format!(
                        "unable to read plugin file {:?} metadata: {}",
                        path.as_ref(),
                        err
                    ))
                })?;

            // convert to chrono timestamp in utc
            let created_at_dt: DateTime<Local> = created_at_sys.into();
            created_at_dt.naive_utc()
        };

        let bytes = std::fs::read(path.as_ref()).map_err(|err| {
            Error(ErrorOrigin::Inventory, ErrorKind::UnableToReadFile).log_error(format!(
                "unable to read plugin file {:?}: {}",
                path.as_ref(),
                err
            ))
        })?;

        let descriptors = plugin_analyzer::parse_descriptors(&bytes).map_err(|err| {
            Error(ErrorOrigin::Inventory, ErrorKind::UnableToReadFile).log_error(format!(
                "unable to parse descriptors in plugin file {:?}: {}",
                path.as_ref(),
                err
            ))
        })?;

        // metadata is guaranteed to contain the same plugin_version for all connectors of a file.
        let first_descriptor = descriptors.first().ok_or_else(|| {
            Error(ErrorOrigin::Inventory, ErrorKind::InvalidExeFile).log_warn(format!(
                "no plugin descriptor found in plugin file {:?}",
                path.as_ref(),
            ))
        })?;

        // check plugin architecture
        if first_descriptor.file_type != plugin_file_type()
            || first_descriptor.architecture != plugin_architecture()
        {
            return Err(
                    Error(ErrorOrigin::Inventory, ErrorKind::InvalidArchitecture).log_warn(
                        format!(
                            "plugin with incompatible architecture found {:?} (plugin was built for {:?}:{:?})",
                            path.as_ref(),
                            first_descriptor.file_type,
                            first_descriptor.architecture,
                        ),
                    ),
                );
        }

        // check plugin version
        if first_descriptor.plugin_version != MEMFLOW_PLUGIN_VERSION {
            return Err(Error(ErrorOrigin::Inventory, ErrorKind::VersionMismatch).log_warn(format!(
                    "plugin with incompatible version found {:?} (expected version {} but plugin had version {})",
                    path.as_ref(),
                    MEMFLOW_PLUGIN_VERSION,
                    first_descriptor.plugin_version
                )));
        }

        // sort by created_at
        let search_key = created_at;

        if let Ok(pos) = self
            .plugins
            .binary_search_by_key(&Reverse(search_key), |entry| {
                Reverse(entry.metadata.created_at)
            })
            .or_else(|v| {
                for p in &self.plugins[v..] {
                    if p.path == path.as_ref() {
                        return Err(v);
                    }
                    if p.metadata.created_at != created_at {
                        break;
                    }
                }
                Ok(v)
            })
        {
            self.plugins.insert(
                pos,
                PluginEntry {
                    path: path.as_ref().to_path_buf(),
                    instance: None,
                    metadata: PluginMetadata {
                        digest: String::new(), // TODO: not needed atm
                        signature: String::new(),
                        created_at,
                        descriptors,
                    },
                },
            );
        }

        Ok(self)
    }

    // TODO: download plugins

    /// Creates a new Connector / OS builder.
    ///
    /// # Examples
    ///
    /// Create a connector:
    /// ```no_run
    /// use memflow::plugins::Inventory;
    ///
    /// let mut inventory = Inventory::scan();
    /// let os = inventory
    ///   .builder()
    ///   .connector("qemu")
    ///   .build();
    /// ```
    ///
    /// Create a Connector with arguments:
    /// ```no_run
    /// use memflow::plugins::{Inventory, Args};
    ///
    /// let mut inventory = Inventory::scan();
    /// let os = inventory
    ///   .builder()
    ///   .connector("qemu")
    ///   .args(str::parse("vm-win10").unwrap())
    ///   .build();
    /// ```
    ///
    /// Create a Connector and OS with arguments:
    /// ```no_run
    /// use memflow::plugins::{Inventory, Args};
    ///
    /// let mut inventory = Inventory::scan();
    /// let os = inventory
    ///   .builder()
    ///   .connector("qemu")
    ///   .args(str::parse("vm-win10").unwrap())
    ///   .os("win10")
    ///   .build();
    /// ```
    ///
    /// Create a OS without a connector and arguments:
    /// ```no_run
    /// use memflow::plugins::Inventory;
    ///
    /// let mut inventory = Inventory::scan();
    /// let os = inventory
    ///   .builder()
    ///   .os("native")
    ///   .build();
    /// ```
    pub fn builder(&mut self) -> BuilderEmpty {
        BuilderEmpty::new(self)
    }

    /// Instantiates a new connector instance.
    /// The instance will be initialized with the args provided to this call.
    ///
    /// In case no connector could be found this will throw an `Error::Library`.
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
    /// let mut inventory = Inventory::scan_path("./").unwrap();
    /// let connector = inventory
    ///     .instantiate_connector("coredump", None, None)
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
    pub fn instantiate_connector(
        &mut self,
        name: &str,
        input: ConnectorInputArg,
        args: Option<&ConnectorArgs>,
    ) -> Result<ConnectorInstanceArcBox<'static>> {
        self.instantiate_plugin::<LoadableConnector>(name, input, args)
    }

    #[deprecated(note = "use instantiate_connector instead")]
    pub fn create_connector(
        &mut self,
        name: &str,
        input: ConnectorInputArg,
        args: Option<&ConnectorArgs>,
    ) -> Result<ConnectorInstanceArcBox<'static>> {
        self.instantiate_connector(name, input, args)
    }

    /// Instantiates a new connector instance.
    /// The instance will be initialized with the args provided to this call.
    ///
    /// In case no connector could be found this will throw an `Error::Library`.
    ///
    /// # Safety
    ///
    /// This function assumes all libraries were loaded with appropriate safety
    /// checks in place. This function is safe, but can crash if previous checks
    /// fail.
    ///
    /// # Examples
    ///
    /// Creating a OS instance with custom arguments
    /// ```
    /// use memflow::plugins::{Inventory, ConnectorArgs};
    ///
    /// # let mut inventory = Inventory::scan().add_cargo_workspace().unwrap();
    /// let args = str::parse(":4m").unwrap();
    /// let os = inventory.instantiate_os("dummy", None, Some(&args))
    ///     .unwrap();
    /// std::mem::drop(os);
    /// ```
    pub fn instantiate_os(
        &mut self,
        name: &str,
        input: OsInputArg,
        args: Option<&OsArgs>,
    ) -> Result<OsInstanceArcBox<'static>> {
        self.instantiate_plugin::<LoadableOs>(name, input, args)
    }

    #[deprecated(note = "use instantiate_os instead")]
    pub fn create_os(
        &mut self,
        name: &str,
        input: OsInputArg,
        args: Option<&OsArgs>,
    ) -> Result<OsInstanceArcBox<'static>> {
        self.instantiate_os(name, input, args)
    }

    // TODO: name should be PluginUri with appropriate tag
    fn instantiate_plugin<T: Loadable>(
        &mut self,
        name: &str,
        input: T::InputArg,
        args: Option<&T::ArgsType>,
    ) -> Result<T::Instance> {
        // instantiate the plugin
        let (instance, loadable) = self.load_plugin::<T>(name)?;
        loadable.instantiate(instance.clone(), input, args)
    }

    fn load_plugin<T: Loadable>(&mut self, name: &str) -> Result<(CArc<LibContext>, T)> {
        // find plugin + descriptor
        let (plugin, descriptor) = self
            .plugins
            .iter_mut()
            .find_map(|plugin| {
                plugin
                    .metadata
                    .descriptors
                    .iter()
                    .filter(|descriptor| descriptor.plugin_kind == T::plugin_kind())
                    .find(|descriptor| descriptor.name == name)
                    .cloned()
                    .map(|descriptor| (plugin, descriptor))
            })
            .ok_or(Error(ErrorOrigin::Inventory, ErrorKind::PluginNotFound))?;

        // load plugin instance
        let instance = plugin.load_instance()?;

        // create `Loadable` from instance
        let loadable = T::from_instance(instance, &descriptor.export_name)?;

        Ok((instance.clone(), loadable))
    }

    /// Sets the maximum logging level in all plugins and updates the
    /// internal [`PluginLogger`] in each plugin instance.
    pub fn set_max_log_level(&self, level: LevelFilter) {
        log::set_max_level(level);
        self.update_max_log_level()
    }

    fn update_max_log_level(&self) {
        let level = log::max_level();

        self.plugins
            .iter()
            .filter_map(|s| s.instance.as_ref())
            .filter_map(|s| *s.as_ref())
            .filter_map(LibContext::try_get_logger)
            .for_each(|l| l.on_level_change(level));
    }

    /// Returns the names of all currently available connectors that can be used.
    pub fn available_connectors(&self) -> Vec<String> {
        self.plugins
            .iter()
            .flat_map(|plugin| {
                plugin
                    .metadata
                    .descriptors
                    .iter()
                    .filter(|descriptor| descriptor.plugin_kind == PluginKind::Connector)
                    .map(|descriptor| descriptor.name.clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }

    /// Returns the names of all currently available os plugins that can be used.
    pub fn available_os(&self) -> Vec<String> {
        self.plugins
            .iter()
            .flat_map(|plugin| {
                plugin
                    .metadata
                    .descriptors
                    .iter()
                    .filter(|descriptor| descriptor.plugin_kind == PluginKind::Os)
                    .map(|descriptor| descriptor.name.clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }

    /// Returns the help string of the given Connector.
    ///
    /// This function returns an error in case the Connector was not found or does not implement the help feature.
    pub fn connector_help(&mut self, name: &str) -> Result<String> {
        let (_, loadable) = self.load_plugin::<LoadableConnector>(name)?;
        loadable.help()
    }

    /// Returns the help string of the given Os Plugin.
    ///
    /// This function returns an error in case the Os Plugin was not found or does not implement the help feature.
    pub fn os_help(&mut self, name: &str) -> Result<String> {
        let (_, loadable) = self.load_plugin::<LoadableOs>(name)?;
        loadable.help()
    }

    /// Returns a list of all available targets of the connector.
    ///
    /// This function returns an error in case the connector does not implement this feature.
    pub fn connector_target_list(&mut self, name: &str) -> Result<Vec<TargetInfo>> {
        let (_, loadable) = self.load_plugin::<LoadableConnector>(name)?;
        loadable.target_list()
    }
}
