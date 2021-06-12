use crate::connector::*;
use crate::error::*;
use crate::mem::PhysicalMemory;
use cglue::{arc::CArc, repr_cstring::ReprCString};

use super::{
    Args, ConnectorInstance, ConnectorInstanceBox, GenericBaseTable, Loadable,
    MuConnectorInstanceBox, OpaqueBaseTable, OsInstance, OsInstanceBox, PluginDescriptor,
    TargetInfo,
};

use std::mem::MaybeUninit;

use libloading::Library;

pub fn create_with_logging<T: 'static + PhysicalMemory + Clone>(
    args: &ReprCString,
    log_level: i32,
    out: &mut MuConnectorInstanceBox,
    create_fn: impl Fn(&Args, log::Level) -> Result<T>,
) -> i32 {
    super::util::create_with_logging(args, log_level, out, |a, l| {
        Ok(create_fn(&a, l).map(ConnectorInstance::builder)?.build())
    })
}

pub fn create_without_logging<T: 'static + PhysicalMemory + Clone>(
    args: &ReprCString,
    out: &mut MuConnectorInstanceBox,
    create_fn: impl Fn(&Args) -> Result<T>,
) -> i32 {
    super::util::create_without_logging(args, out, |a| {
        Ok(create_fn(&a).map(ConnectorInstance::builder)?.build())
    })
}

pub type ConnectorDescriptor = PluginDescriptor<LoadableConnector>;

pub struct LoadableConnector {
    descriptor: PluginDescriptor<Self>,
}

impl Loadable for LoadableConnector {
    type Instance = ConnectorInstanceBox<'static>;
    type InputArg = Option<OsInstanceBox<'static>>;
    type CInputArg = Option<OsInstanceBox<'static>>;

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
    ) -> Result<Self::Instance> {
        let cstr = ReprCString::from(args.to_string());
        let mut out = MuConnectorInstanceBox::uninit();
        let res = (self.descriptor.create)(&cstr, input, log::max_level() as i32, &mut out);
        result_from_int(res, out).map(|mut c| {
            c.library = library.into();
            c
        })
    }
}
