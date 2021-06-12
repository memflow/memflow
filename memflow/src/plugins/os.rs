use crate::error::*;
use crate::os::*;
use crate::types::Address;

use super::{
    Args, ConnectorInstance, ConnectorInstanceBox, Loadable, MuOsInstanceBox, OsInstance,
    OsInstanceBox, PluginDescriptor, TargetInfo,
};

use cglue::*;
use cglue::{
    arc::CArc,
    arc::{ArcWrapped, COptArc},
    boxed::CBox,
    option::COption,
    repr_cstring::ReprCString,
    result::{from_int_result, from_int_result_empty},
};
use libloading::Library;

use std::mem::MaybeUninit;

pub type OptionArchitectureIdent<'a> = Option<&'a crate::architecture::ArchitectureIdent>;

pub fn create_with_logging<T: 'static>(
    args: &ReprCString,
    conn: ConnectorInstanceBox,
    lib: COptArc<Library>,
    log_level: i32,
    out: &mut MuOsInstanceBox<'static>,
    create_fn: impl Fn(&Args, ConnectorInstanceBox, log::Level) -> Result<T>,
) -> i32
where
    OsInstance<'static, CBox<'static, T>, T>: From<T>,
{
    super::util::create_with_logging(args, lib, log_level, out, move |a, l| {
        Ok(group_obj!(create_fn(&a, conn, l)? as OsInstance))
    })
}

pub fn create_without_logging<T: 'static>(
    args: &ReprCString,
    conn: ConnectorInstanceBox,
    lib: COptArc<Library>,
    out: &mut MuOsInstanceBox<'static>,
    create_fn: impl Fn(&Args, ConnectorInstanceBox) -> Result<T>,
) -> i32
where
    OsInstance<'static, CBox<'static, T>, T>: From<T>,
{
    super::util::create_without_logging(args, lib, out, |a| {
        Ok(group_obj!(create_fn(&a, conn)? as OsInstance))
    })
}

pub type OsDescriptor = PluginDescriptor<LoadableOs>;

pub struct LoadableOs {
    descriptor: PluginDescriptor<Self>,
}

impl Loadable for LoadableOs {
    type Instance = OsInstanceBox<'static>;
    type InputArg = Option<ConnectorInstanceBox<'static>>;
    type CInputArg = COption<ConnectorInstanceBox<'static>>;

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
                        "Os-Plugin `{}` did not return any help text.",
                        self.ident()
                    ))
                })
            }
            None => Err(
                Error(ErrorOrigin::Connector, ErrorKind::NotSupported).log_error(&format!(
                    "Os-Plugin `{}` does not support help text.",
                    self.ident()
                )),
            ),
        }
    }

    /// Retrieves the list of available targets for this connector.
    fn target_list(&self) -> Result<Vec<TargetInfo>> {
        Err(Error(ErrorOrigin::Connector, ErrorKind::NotSupported)
            .log_error("Os-Plugin does not support target listing."))
    }

    /// Creates a new OS instance from this library.
    ///
    /// The OS is initialized with the arguments provided to this function.
    fn instantiate(
        &self,
        library: COptArc<Library>,
        input: Self::InputArg,
        args: &Args,
    ) -> Result<Self::Instance> {
        let cstr = ReprCString::from(args.to_string());
        let mut out = MuOsInstanceBox::uninit();
        let res = (self.descriptor.create)(
            &cstr,
            input.into(),
            library,
            log::max_level() as i32,
            &mut out,
        );
        unsafe { from_int_result(res, out) }
    }
}
