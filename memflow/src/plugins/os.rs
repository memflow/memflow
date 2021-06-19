use crate::cglue::{result::from_int_result, *};
use crate::error::*;
use crate::os::Os;

use super::{
    Args, ConnectorInstanceArcBox, Loadable, MuOsInstanceArcBox, OsInstance, OsInstanceArcBox,
    OsInstanceBaseArcBox, OsInstanceVtableFiller, PluginDescriptor, TargetInfo,
};

use libloading::Library;
use std::ffi::c_void;

pub type OptionArchitectureIdent<'a> = Option<&'a crate::architecture::ArchitectureIdent>;

pub fn create_with_logging<
    T: 'static
        + Os
        + Clone
        + OsInstanceVtableFiller<
            'static,
            CtxBox<'static, T, COptArc<c_void>>,
            COptArc<c_void>,
            COptArc<c_void>,
        >,
>(
    args: &ReprCString,
    conn: ConnectorInstanceArcBox,
    lib: COptArc<c_void>,
    log_level: i32,
    out: &mut MuOsInstanceArcBox<'static>,
    create_fn: impl Fn(&Args, ConnectorInstanceArcBox, log::Level) -> Result<T>,
) -> i32
where
    (T, COptArc<c_void>): Into<OsInstanceBaseArcBox<'static, T, c_void>>,
{
    super::util::create_with_logging(args, lib, log_level, out, move |a, lib, l| {
        Ok(group_obj!((create_fn(&a, conn, l)?, lib) as OsInstance))
    })
}

pub fn create_without_logging<
    T: 'static
        + Os
        + Clone
        + OsInstanceVtableFiller<
            'static,
            CtxBox<'static, T, COptArc<c_void>>,
            COptArc<c_void>,
            COptArc<c_void>,
        >,
>(
    args: &ReprCString,
    conn: ConnectorInstanceArcBox,
    lib: COptArc<c_void>,
    out: &mut MuOsInstanceArcBox<'static>,
    create_fn: impl Fn(&Args, ConnectorInstanceArcBox) -> Result<T>,
) -> i32
where
    (T, COptArc<c_void>): Into<OsInstanceBaseArcBox<'static, T, c_void>>,
{
    super::util::create_without_logging(args, lib, out, |a, lib| {
        Ok(group_obj!((create_fn(&a, conn)?, lib) as OsInstance))
    })
}

pub type OsDescriptor = PluginDescriptor<LoadableOs>;

pub struct LoadableOs {
    descriptor: PluginDescriptor<Self>,
}

impl Loadable for LoadableOs {
    type Instance = OsInstanceArcBox<'static>;
    type InputArg = Option<ConnectorInstanceArcBox<'static>>;
    type CInputArg = COption<ConnectorInstanceArcBox<'static>>;

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
        let mut out = MuOsInstanceArcBox::uninit();
        let res = (self.descriptor.create)(
            &cstr,
            input.into(),
            library.into_opaque(),
            log::max_level() as i32,
            &mut out,
        );
        unsafe { from_int_result(res, out) }
    }
}
