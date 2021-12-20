use crate::cglue::{
    result::{from_int_result, from_int_result_empty},
    *,
};
use crate::error::*;
use crate::mem::phys_mem::*;

use super::{
    Args, LibContext, Loadable, OsInstanceArcBox, PluginDescriptor, PluginLogger, TargetInfo,
};

use crate::connector::cpu_state::*;
use std::ffi::c_void;

cglue_trait_group!(ConnectorInstance<'a>, { PhysicalMemory, Clone }, { ConnectorCpuStateInner<'a> });
pub type MuConnectorInstanceArcBox<'a> = std::mem::MaybeUninit<ConnectorInstanceArcBox<'a>>;

pub fn create<
    T: 'static
        + PhysicalMemory
        + Clone
        + ConnectorInstanceVtableFiller<'static, CBox<'static, T>, CArc<c_void>>,
>(
    args: &ReprCString,
    lib: CArc<c_void>,
    logger: Option<&'static PluginLogger>,
    out: &mut MuConnectorInstanceArcBox<'static>,
    create_fn: impl Fn(&Args) -> Result<T>,
) -> i32 {
    super::util::create(args, lib, logger, out, |a, lib| {
        Ok(group_obj!((create_fn(&a)?, lib) as ConnectorInstance))
    })
}

pub type ConnectorDescriptor = PluginDescriptor<LoadableConnector>;

pub struct LoadableConnector {
    descriptor: PluginDescriptor<Self>,
}

impl Loadable for LoadableConnector {
    type Instance = ConnectorInstanceArcBox<'static>;
    type InputArg = Option<OsInstanceArcBox<'static>>;
    type CInputArg = COption<OsInstanceArcBox<'static>>;

    fn ident(&self) -> &str {
        unsafe { self.descriptor.name.into_str() }
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
                from_int_result_empty::<Error>((target_list_callback)((&mut ret).into()))?;
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
        library: CArc<LibContext>,
        input: Self::InputArg,
        args: &Args,
    ) -> Result<Self::Instance> {
        let cstr = ReprCString::from(args.to_string());
        let mut out = MuConnectorInstanceArcBox::uninit();
        let logger = library.as_ref().map(|lib| unsafe { lib.get_logger() });
        let res =
            (self.descriptor.create)(&cstr, input.into(), library.into_opaque(), logger, &mut out);
        unsafe { from_int_result(res, out) }
    }
}
