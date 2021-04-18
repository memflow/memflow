use crate::error::*;
use crate::os::*;
use crate::types::Address;
use crate::types::ReprCString;

pub mod instance;
pub use instance::{OpaqueOsFunctionTable, OsFunctionTable, OsInstance};

pub mod process;
pub use process::{ArcPluginProcess, PluginProcess};

pub mod keyboard;
pub use keyboard::{
    ArcPluginKeyboard, ArcPluginKeyboardState, OpaqueOsKeyboardFunctionTable,
    OsKeyboardFunctionTable, PluginKeyboard,
};

use super::{
    Args, CArc, COption, ConnectorInstance, GenericBaseTable, Loadable, OpaqueBaseTable,
    PluginDescriptor, TargetInfo,
};

use libloading::Library;

use std::mem::MaybeUninit;

// Type aliases needed for &mut MaybeUninit<T> to work with bindgen
pub type MuProcessInfo = MaybeUninit<ProcessInfo>;
pub type MuModuleInfo = MaybeUninit<ModuleInfo>;
pub type MuPluginProcess<'a> = MaybeUninit<PluginProcess<'a>>;
pub type MuArcPluginProcess = MaybeUninit<ArcPluginProcess>;
pub type MuPluginKeyboard<'a> = MaybeUninit<PluginKeyboard<'a>>;
pub type MuArcPluginKeyboard = MaybeUninit<ArcPluginKeyboard>;
pub type MuArcPluginKeyboardState = MaybeUninit<ArcPluginKeyboardState>;
pub type MuAddress = MaybeUninit<Address>;
pub type MuOsInstance = MaybeUninit<OsInstance>;

pub type OptionArchitectureIdent<'a> = Option<&'a crate::architecture::ArchitectureIdent>;

/// Subtrait of Plugin where `Self`, and `Os::IntoProcessType` are `Clone`
pub trait PluginOs<T: Process + Clone>:
    'static + Clone + for<'a> OsInner<'a, IntoProcessType = T>
{
}
impl<T: Process + Clone, K: 'static + Clone + for<'a> OsInner<'a, IntoProcessType = T>> PluginOs<T>
    for K
{
}

/// Subtrait of Plugin where `Self`, and `OsKeyboard::IntoKeyboardType` are `Clone`
pub trait PluginOsKeyboard<T: Keyboard + Clone>:
    'static + Clone + for<'a> OsKeyboardInner<'a, IntoKeyboardType = T>
{
}
impl<
        T: Keyboard + Clone,
        K: 'static + Clone + for<'a> OsKeyboardInner<'a, IntoKeyboardType = T>,
    > PluginOsKeyboard<T> for K
{
}

pub fn create_with_logging<P: 'static + Process + Clone, T: PluginOs<P>>(
    args: &ReprCString,
    conn: ConnectorInstance,
    log_level: i32,
    out: &mut MuOsInstance,
    create_fn: impl Fn(&Args, ConnectorInstance, log::Level) -> Result<T>,
) -> i32 {
    super::util::create_with_logging(args, log_level, out, move |a, l| {
        Ok(create_fn(&a, conn, l).map(OsInstance::builder)?.build())
    })
}

pub fn create_without_logging<P: 'static + Process + Clone, T: PluginOs<P>>(
    args: &ReprCString,
    conn: ConnectorInstance,
    out: &mut MuOsInstance,
    create_fn: impl Fn(&Args, ConnectorInstance) -> Result<T>,
) -> i32 {
    super::util::create_without_logging(args, out, |a| {
        Ok(create_fn(&a, conn).map(OsInstance::builder)?.build())
    })
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OsLayerFunctionTable {
    /// The vtable for object creation and cloning
    pub base: &'static OpaqueBaseTable,
    /// The vtable for all os functions
    pub os: &'static OpaqueOsFunctionTable,
    /// The vtable for the keyboard access if available
    pub keyboard: Option<&'static OpaqueOsKeyboardFunctionTable>,
}

impl OsLayerFunctionTable {
    pub fn new<P: 'static + Process + Clone, T: PluginOs<P>>() -> Self {
        OsLayerFunctionTable {
            base: <&GenericBaseTable<T>>::default().as_opaque(),
            os: <&OsFunctionTable<P, T>>::default().as_opaque(),
            keyboard: None,
        }
    }
}

pub type OsDescriptor = PluginDescriptor<LoadableOs>;

pub struct LoadableOs {
    descriptor: PluginDescriptor<Self>,
}

impl Loadable for LoadableOs {
    type Instance = OsInstance;
    type InputArg = Option<ConnectorInstance>;
    type CInputArg = COption<ConnectorInstance>;

    fn export_prefix() -> &'static str {
        "MEMFLOW_OS_"
    }

    fn ident(&self) -> &str {
        self.descriptor.name
    }

    fn plugin_type() -> &'static str {
        "OS"
    }

    fn new(descriptor: PluginDescriptor<Self>) -> Self {
        Self { descriptor }
    }

    /// Retrieves the help text for this plugin
    fn help(&self) -> Result<String> {
        match self.descriptor.help_callback {
            Some(help_callback) => {
                let mut ret = vec![];
                (help_callback)((&mut ret).into());
                ret.first().map(|h| h.to_string()).ok_or_else(|| {
                    Error(ErrorOrigin::Connector, ErrorKind::NotSupported).log_error(&format!(
                        "Connector `{}` did not return any help text.",
                        self.ident()
                    ))
                })
            }
            None => Err(
                Error(ErrorOrigin::Connector, ErrorKind::NotSupported).log_error(&format!(
                    "Connector `{}` does not support help text.",
                    self.ident()
                )),
            ),
        }
    }

    /// Retrieves the list of available targets for this connector.
    fn target_list(&self) -> Result<Vec<TargetInfo>> {
        Err(Error(ErrorOrigin::Connector, ErrorKind::NotSupported)
            .log_error("Os does not support target listing."))
    }

    /// Creates a new OS instance from this library.
    ///
    /// The OS is initialized with the arguments provided to this function.
    fn instantiate(
        &self,
        library: Option<CArc<Library>>,
        input: Option<ConnectorInstance>,
        args: &Args,
    ) -> Result<OsInstance> {
        let cstr = ReprCString::from(args.to_string());
        let mut out = MuOsInstance::uninit();
        let res = (self.descriptor.create)(&cstr, input.into(), log::max_level() as i32, &mut out);
        result_from_int(res, out).map(|mut c| {
            c.library = library.into();
            c
        })
    }
}
