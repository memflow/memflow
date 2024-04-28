use core::cmp::Reverse;
use std::path::{Path, PathBuf};

use cglue::arc::CArc;
use chrono::{DateTime, Local, NaiveDateTime};
use libloading::Library;
use log::{debug, info, warn};
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
    ConnectorArgs, ConnectorInputArg, ConnectorInstanceArcBox, LibContext, Loadable,
    LoadableConnector, LoadableOs, OsArgs, OsInputArg, OsInstanceArcBox,
};

pub struct Registry {
    plugins: Vec<PluginEntry>,
}

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

impl Default for Registry {
    fn default() -> Self {
        let mut registry = Self {
            plugins: Vec::new(),
        };

        registry
            .add_dir(plugins_path())
            .expect("unable to parse plugin path");

        for plugin in registry.plugins.iter() {
            for descriptor in plugin.metadata.descriptors.iter() {
                info!(
                    "Found installed {:?} Plugin: {} {} ({:?})",
                    descriptor.plugin_kind, descriptor.name, descriptor.version, plugin.path,
                );
            }
        }

        registry
    }
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

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

        // metadata is guaranteed to contain at least one descriptor and the plugin_version is identical for all connectors of a file.
        let first_descriptor = descriptors.first().unwrap();

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
        match self
            .plugins
            .binary_search_by_key(&Reverse(search_key), |entry| {
                Reverse(entry.metadata.created_at)
            }) {
            Ok(_) => unreachable!(), // element already in vector @ `pos` // TODO: check for duplicate entries
            Err(pos) => self.plugins.insert(
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
            ),
        }

        Ok(self)
    }

    // TODO: download plugins

    pub fn instantiate_connector(
        &mut self,
        name: &str,
        input: ConnectorInputArg,
        args: Option<&ConnectorArgs>,
    ) -> Result<ConnectorInstanceArcBox<'static>> {
        self.instantiate_plugin::<LoadableConnector>(name, input, args)
    }

    pub fn instantiate_os(
        &mut self,
        name: &str,
        input: OsInputArg,
        args: Option<&OsArgs>,
    ) -> Result<OsInstanceArcBox<'static>> {
        self.instantiate_plugin::<LoadableOs>(name, input, args)
    }

    // TODO: name should be PluginUri with appropriate tag
    fn instantiate_plugin<T: Loadable>(
        &mut self,
        name: &str,
        input: T::InputArg,
        args: Option<&T::ArgsType>,
    ) -> Result<T::Instance> {
        // find plugin + descriptor
        let (plugin, descriptor) = self
            .plugins
            .iter_mut()
            .find_map(|plugin| {
                plugin
                    .metadata
                    .descriptors
                    .clone()
                    .into_iter()
                    .find(|descriptor| descriptor.name == name)
                    .map(|descriptor| (plugin, descriptor))
            })
            .ok_or(Error(ErrorOrigin::Inventory, ErrorKind::PluginNotFound))?;

        // load plugin instance
        let instance = plugin.load_instance()?;

        // create `Loadable` from instance
        let loadable = T::from_instance(instance, &descriptor.export_name)?;

        // instantiate the plugin
        loadable.instantiate(instance.clone(), input, args)
    }
}
