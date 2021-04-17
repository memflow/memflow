use crate::error::*;
use crate::mem::{PhysicalMemory, VirtualMemory};
use crate::os::*;
use crate::types::Address;
use crate::types::ReprCStr;

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
    PluginDescriptor,
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
pub trait PluginOs<T: Process + Clone, P: PhysicalMemory, V: VirtualMemory>:
    'static
    + Clone
    + for<'a> OsInner<'a, IntoProcessType = T, PhysicalMemoryType = P, VirtualMemoryType = V>
{
}
impl<
        T: Process + Clone,
        P: 'static + PhysicalMemory,
        V: 'static + VirtualMemory,
        K: 'static
            + Clone
            + for<'a> OsInner<'a, IntoProcessType = T, PhysicalMemoryType = P, VirtualMemoryType = V>,
    > PluginOs<T, P, V> for K
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

pub fn create_with_logging<
    P: 'static + Process + Clone,
    PM: 'static + PhysicalMemory,
    VM: 'static + VirtualMemory,
    T: PluginOs<P, PM, VM>,
>(
    args: &ReprCStr,
    conn: ConnectorInstance,
    log_level: i32,
    out: &mut MuOsInstance,
    create_fn: impl Fn(&Args, ConnectorInstance, log::Level) -> Result<T>,
) -> i32 {
    super::util::create_with_logging(args, log_level, out, move |a, l| {
        Ok(create_fn(&a, conn, l).map(OsInstance::builder)?.build())
    })
}

pub fn create_without_logging<
    P: 'static + Process + Clone,
    PM: 'static + PhysicalMemory,
    VM: 'static + VirtualMemory,
    T: PluginOs<P, PM, VM>,
>(
    args: &ReprCStr,
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
    pub fn new<
        P: 'static + Process + Clone,
        PM: 'static + PhysicalMemory,
        VM: 'static + VirtualMemory,
        T: PluginOs<P, PM, VM>,
    >() -> Self {
        OsLayerFunctionTable {
            base: <&GenericBaseTable<T>>::default().as_opaque(),
            os: <&OsFunctionTable<P, PM, VM, T>>::default().as_opaque(),
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

    /// Creates a new OS instance from this library.
    ///
    /// The OS is initialized with the arguments provided to this function.
    fn instantiate(
        &self,
        library: Option<CArc<Library>>,
        input: Option<ConnectorInstance>,
        args: &Args,
    ) -> Result<OsInstance> {
        let cstr = ReprCStr::from(args.to_string());
        let mut out = MuOsInstance::uninit();
        let res = (self.descriptor.create)(&cstr, input.into(), log::max_level() as i32, &mut out);
        result_from_int(res, out).map(|mut c| {
            c.library = library.into();
            c
        })
    }
}
