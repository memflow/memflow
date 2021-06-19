use crate::cglue::{
    result::{from_int_result, from_int_result_empty},
    *,
};
use crate::error::*;
use crate::mem::PhysicalMemory;

use super::{
    Args, ConnectorInstance, ConnectorInstanceArcBox, ConnectorInstanceVtableFiller, Loadable,
    MuConnectorInstanceArcBox, OsInstanceArcBox, PluginDescriptor, TargetInfo,
};

use libloading::Library;
use std::ffi::c_void;

pub fn create_with_logging<
    T: 'static
        + PhysicalMemory
        + Clone
        + ConnectorInstanceVtableFiller<
            'static,
            CtxBox<'static, T, COptArc<c_void>>,
            COptArc<c_void>,
            COptArc<c_void>,
        >,
>(
    args: &ReprCString,
    lib: COptArc<c_void>,
    log_level: i32,
    out: &mut MuConnectorInstanceArcBox<'static>,
    create_fn: impl Fn(&Args, log::Level) -> Result<T>,
) -> i32 {
    super::util::create_with_logging(args, lib, log_level, out, |a, lib, l| {
        Ok(group_obj!((create_fn(&a, l)?, lib) as ConnectorInstance))
    })
}

pub fn create_without_logging<
    T: 'static
        + PhysicalMemory
        + Clone
        + ConnectorInstanceVtableFiller<
            'static,
            CtxBox<'static, T, COptArc<c_void>>,
            COptArc<c_void>,
            COptArc<c_void>,
        >,
>(
    args: &ReprCString,
    lib: COptArc<c_void>,
    out: &mut MuConnectorInstanceArcBox<'static>,
    create_fn: impl Fn(&Args) -> Result<T>,
) -> i32 {
    super::util::create_without_logging(args, lib, out, |a, lib| {
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
    type CInputArg = Option<OsInstanceArcBox<'static>>;

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
        library: COptArc<Library>,
        input: Self::InputArg,
        args: &Args,
    ) -> Result<Self::Instance> {
        let cstr = ReprCString::from(args.to_string());
        let mut out = MuConnectorInstanceArcBox::uninit();
        let res = (self.descriptor.create)(
            &cstr,
            input,
            library.into_opaque(),
            log::max_level() as i32,
            &mut out,
        );
        unsafe { from_int_result(res, out) }
    }
}
