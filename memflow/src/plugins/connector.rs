use crate::connector::*;
use crate::error::*;
use crate::mem::PhysicalMemory;

pub mod instance;
pub use instance::{
    ConnectorInstance, OpaquePhysicalMemoryFunctionTable, PhysicalMemoryFunctionTable,
    PhysicalMemoryInstance,
};

use super::{
    Args, CArc, GenericBaseTable, Loadable, OpaqueBaseTable, OsInstance, PluginDescriptor,
    TargetInfo,
};

use crate::types::ReprCString;

use std::mem::MaybeUninit;

pub mod cpu_state;
pub use cpu_state::{
    ArcPluginCpuState, ConnectorCpuStateFunctionTable, OpaqueConnectorCpuStateFunctionTable,
    PluginCpuState,
};

use libloading::Library;

// Type aliases needed for &mut MaybeUninit<T> to work with bindgen
pub type MuPluginCpuState<'a> = MaybeUninit<PluginCpuState<'a>>;
pub type MuArcPluginCpuState = MaybeUninit<ArcPluginCpuState>;
pub type MuConnectorInstance = MaybeUninit<ConnectorInstance>;

/// Subtrait of Plugin where `Self`, and `OsKeyboard::IntoKeyboardType` are `Clone`
pub trait PluginConnectorCpuState<T: CpuState + Clone>:
    'static + Clone + for<'a> ConnectorCpuStateInner<'a, IntoCpuStateType = T>
{
}
impl<
        T: CpuState + Clone,
        C: 'static + Clone + for<'a> ConnectorCpuStateInner<'a, IntoCpuStateType = T>,
    > PluginConnectorCpuState<T> for C
{
}

pub fn create_with_logging<T: 'static + PhysicalMemory + Clone>(
    args: &ReprCString,
    log_level: i32,
    out: &mut MuConnectorInstance,
    create_fn: impl Fn(&Args, log::Level) -> Result<T>,
) -> i32 {
    super::util::create_with_logging(args, log_level, out, |a, l| {
        Ok(create_fn(&a, l).map(ConnectorInstance::builder)?.build())
    })
}

pub fn create_without_logging<T: 'static + PhysicalMemory + Clone>(
    args: &ReprCString,
    out: &mut MuConnectorInstance,
    create_fn: impl Fn(&Args) -> Result<T>,
) -> i32 {
    super::util::create_without_logging(args, out, |a| {
        Ok(create_fn(&a).map(ConnectorInstance::builder)?.build())
    })
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ConnectorFunctionTable {
    /// The vtable for object creation and cloning
    pub base: &'static OpaqueBaseTable,
    /// The vtable for all physical memory function calls to the connector.
    pub phys: &'static OpaquePhysicalMemoryFunctionTable,
    // The vtable for cpu state if available
    pub cpu_state: Option<&'static OpaqueConnectorCpuStateFunctionTable>,
}

impl ConnectorFunctionTable {
    pub fn create_vtable<T: 'static + PhysicalMemory + Clone>() -> Self {
        Self {
            base: <&GenericBaseTable<T>>::default().as_opaque(),
            phys: <&PhysicalMemoryFunctionTable<T>>::default().as_opaque(),
            cpu_state: None,
        }
    }
}

pub type ConnectorDescriptor = PluginDescriptor<LoadableConnector>;

pub struct LoadableConnector {
    descriptor: PluginDescriptor<Self>,
}

impl Loadable for LoadableConnector {
    type Instance = ConnectorInstance;
    type InputArg = Option<OsInstance>;
    type CInputArg = Option<OsInstance>;

    fn ident(&self) -> &str {
        self.descriptor.name
    }

    fn export_prefix() -> &'static str {
        "MEMFLOW_CONNECTOR_"
    }

    fn plugin_type() -> &'static str {
        "Connector"
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

    /// Retrieves the list of available targets for this plugin
    fn target_list(&self) -> Result<Vec<TargetInfo>> {
        match self.descriptor.target_list_callback {
            Some(target_list_callback) => {
                let mut ret = vec![];
                result_from_int_void((target_list_callback)((&mut ret).into()))?;
                Ok(ret)
            }
            None => Err(
                Error(ErrorOrigin::Connector, ErrorKind::NotSupported).log_error(&format!(
                    "Connector `{}` does not support target listing.",
                    self.ident()
                )),
            ),
        }
    }

    /// Creates a new connector instance from this library.
    ///
    /// The connector is initialized with the arguments provided to this function.
    fn instantiate(
        &self,
        library: Option<CArc<Library>>,
        input: Self::InputArg,
        args: &Args,
    ) -> Result<ConnectorInstance> {
        let cstr = ReprCString::from(args.to_string());
        let mut out = MuConnectorInstance::uninit();
        let res = (self.descriptor.create)(&cstr, input, log::max_level() as i32, &mut out);
        result_from_int(res, out).map(|mut c| {
            c.library = library.into();
            c
        })
    }
}
