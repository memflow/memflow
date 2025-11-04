use std::mem::MaybeUninit;
use std::path::PathBuf;

use crate::cglue::result::into_int_out_result;
use crate::error::Error;

use super::{
    plugin_analyzer::{PluginArchitecture, PluginFileType},
    LibArc, PluginLogger,
};
use super::{ErrorKind, ErrorOrigin};

/// Returns the plugin extension appropriate for the current os
pub fn plugin_extension() -> &'static str {
    #[cfg(target_os = "windows")]
    return "dll";
    #[cfg(target_os = "linux")]
    return "so";
    #[cfg(target_os = "macos")]
    return "dylib";
}

/// Returns the plugin file_type appropriate for the current os
pub fn plugin_file_type() -> PluginFileType {
    #[cfg(target_os = "windows")]
    return PluginFileType::Pe;
    #[cfg(target_os = "linux")]
    return PluginFileType::Elf;
    #[cfg(target_os = "macos")]
    return PluginFileType::Mach;
}

/// Returns the plugin architecture appropriate for the current os
pub fn plugin_architecture() -> PluginArchitecture {
    #[cfg(target_arch = "x86_64")]
    return PluginArchitecture::X86_64;
    #[cfg(target_arch = "x86")]
    return PluginArchitecture::X86;
    #[cfg(target_arch = "aarch64")]
    return PluginArchitecture::Arm64;
    #[cfg(target_arch = "arm")]
    return PluginArchitecture::Arm;
}

/// Returns the path in which memflow plugins are stored.
///
/// On unix this is returns ~/.local/lib/memflow
/// On windows this returns C:\Users\[Username]\Documents\memflow
pub fn plugins_path() -> crate::error::Result<PathBuf> {
    let path = if cfg!(unix) {
        dirs::home_dir()
            .unwrap()
            .join(".local")
            .join("lib")
            .join("memflow")
    } else {
        dirs::document_dir().unwrap().join("memflow")
    };

    // ensure plugins path exists
    if !path.exists() {
        std::fs::create_dir_all(&path).map_err(|_| {
            Error(ErrorOrigin::Inventory, ErrorKind::UnableToCreateDirectory)
                .log_error("unable to create plugins directory")
        })?;
    }

    Ok(path)
}

/// Wrapper for instantiating object.
///
/// This function will initialize the [`PluginLogger`],
/// parse args into `Args`, and call the create_fn
///
/// This function is used by the connector proc macro
pub fn wrap<A: Default, T>(
    args: Option<&A>,
    lib: LibArc,
    logger: Option<&'static PluginLogger>,
    out: &mut MaybeUninit<T>,
    create_fn: impl FnOnce(&A, LibArc) -> Result<T, Error>,
) -> i32 {
    if let Some(logger) = logger {
        logger.init().ok();
    }

    let default: A = Default::default();
    let args = args.unwrap_or(&default);

    into_int_out_result(create_fn(args, lib), out)
}

/// Wrapper for instantiating object with all needed parameters
///
/// This function will initialize the [`PluginLogger`],
/// parse args into `Args` and call the create_fn with `input` forwarded.
///
/// This function is used by the connector proc macro
pub fn wrap_with_input<A: Default, T, I>(
    args: Option<&A>,
    input: I,
    lib: LibArc,
    logger: Option<&'static PluginLogger>,
    out: &mut MaybeUninit<T>,
    create_fn: impl FnOnce(&A, I, LibArc) -> Result<T, Error>,
) -> i32 {
    if let Some(logger) = logger {
        logger.init().ok();
    }

    let default: A = Default::default();
    let args = args.unwrap_or(&default);

    into_int_out_result(
        create_fn(args, input, lib).inspect_err(|e| {
            ::log::error!("{}", e);
        }),
        out,
    )
}
