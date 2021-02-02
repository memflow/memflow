use crate::error::*;
use crate::os::*;
use crate::types::Address;
use crate::types::ReprCStr;

pub mod osinstance;
pub use osinstance::{OSFunctionTable, OSInstance, OpaqueOSFunctionTable};

pub mod process;
pub use process::{ArcPluginProcess, PluginProcess};

use super::{
    Args, CArc, COption, ConnectorInstance, GenericBaseTable, Loadable, OpaqueBaseTable,
    OpaquePhysicalMemoryFunctionTable, OpaqueVirtualMemoryFunctionTable, PluginDescriptor,
};

use libloading::Library;

use std::mem::MaybeUninit;

// Type aliases needed for &mut MaybeUninit<T> to work with bindgen
pub type MUProcessInfo = MaybeUninit<ProcessInfo>;
pub type MUModuleInfo = MaybeUninit<ModuleInfo>;
pub type MUPluginProcess<'a> = MaybeUninit<PluginProcess<'a>>;
pub type MUArcPluginProcess = MaybeUninit<ArcPluginProcess>;
pub type MUAddress = MaybeUninit<Address>;
pub type MUOSInstance = MaybeUninit<OSInstance>;

pub type OptionArchitectureIdent<'a> = Option<&'a crate::architecture::ArchitectureIdent>;

/// Subtrait of Plugin where `Self`, and `OS::IntoProcessType` are `Clone`
pub trait PluginOS<T: Process + Clone>:
    'static + Clone + for<'a> OSInner<'a, IntoProcessType = T>
{
}
impl<T: Process + Clone, K: 'static + Clone + for<'a> OSInner<'a, IntoProcessType = T>> PluginOS<T>
    for K
{
}

pub fn create_with_logging<P: 'static + Process + Clone, T: PluginOS<P>>(
    args: &ReprCStr,
    conn: ConnectorInstance,
    log_level: i32,
    out: &mut MUOSInstance,
    create_fn: impl Fn(&Args, ConnectorInstance, log::Level) -> Result<T>,
) -> i32 {
    super::util::create_with_logging(args, log_level, out, move |a, l| {
        create_fn(&a, conn, l).map(OSInstance::new)
    })
}

pub fn create_without_logging<P: 'static + Process + Clone, T: PluginOS<P>>(
    args: &ReprCStr,
    conn: ConnectorInstance,
    out: &mut MUOSInstance,
    create_fn: impl Fn(&Args, ConnectorInstance) -> Result<T>,
) -> i32 {
    super::util::create_without_logging(args, out, |a| create_fn(&a, conn).map(OSInstance::new))
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OSLayerFunctionTable {
    /// The vtable for object creation and cloning
    pub base: &'static OpaqueBaseTable,
    /// The vtable for all os functions
    pub os: &'static OpaqueOSFunctionTable,
    /// The vtable for all physical memory access if available
    pub phys: Option<&'static OpaquePhysicalMemoryFunctionTable>,
    /// The vtable for all virtual memory access if available
    pub virt: Option<&'static OpaqueVirtualMemoryFunctionTable>,
}

impl OSLayerFunctionTable {
    pub fn new<P: 'static + Process + Clone, T: PluginOS<P>>() -> Self {
        OSLayerFunctionTable {
            base: <&GenericBaseTable<T>>::default().as_opaque(),
            os: <&OSFunctionTable<P, T>>::default().as_opaque(),
            phys: None,
            virt: None,
        }
    }
}

pub type OSDescriptor = PluginDescriptor<LoadableOS>;

pub struct LoadableOS {
    descriptor: PluginDescriptor<Self>,
}

impl Loadable for LoadableOS {
    type Instance = OSInstance;
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
    ) -> Result<OSInstance> {
        let cstr = ReprCStr::from(args.to_string());
        let mut out = MUOSInstance::uninit();
        let res = (self.descriptor.create)(&cstr, input.into(), log::max_level() as i32, &mut out);
        result_from_int(res, out).map(|mut c| {
            c.library = library.into();
            c
        })
    }
}
