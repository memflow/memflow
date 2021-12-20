use crate::cglue::{result::from_int_result, *};
use crate::error::*;
use crate::mem::memory_view::*;
use crate::mem::phys_mem::*;
use crate::mem::virt_translate::*;
use crate::os::keyboard::*;
use crate::os::process::*;
use crate::os::root::*;

use super::{
    Args, ConnectorInstanceArcBox, LibContext, Loadable, PluginDescriptor, PluginLogger, TargetInfo,
};

use std::ffi::c_void;

pub type OptionArchitectureIdent<'a> = Option<&'a crate::architecture::ArchitectureIdent>;

cglue_trait_group!(OsInstance<'a>, { OsInner<'a>, Clone }, { PhysicalMemory, MemoryView, OsKeyboardInner<'a> });
pub type MuOsInstanceArcBox<'a> = std::mem::MaybeUninit<OsInstanceArcBox<'a>>;

cglue_trait_group!(ProcessInstance, { Process, MemoryView }, { VirtualTranslate });
cglue_trait_group!(IntoProcessInstance, { Process, MemoryView, Clone }, { VirtualTranslate });

pub fn create<
    T: 'static + Os + Clone + OsInstanceVtableFiller<'static, CBox<'static, T>, CArc<c_void>>,
>(
    args: &ReprCString,
    conn: ConnectorInstanceArcBox,
    lib: CArc<c_void>,
    logger: Option<&'static PluginLogger>,
    out: &mut MuOsInstanceArcBox<'static>,
    create_fn: impl Fn(&Args, ConnectorInstanceArcBox) -> Result<T>,
) -> i32
where
    (T, CArc<c_void>): Into<OsInstanceBaseArcBox<'static, T, c_void>>,
{
    super::util::create(args, lib, logger, out, |a, lib| {
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
        unsafe { self.descriptor.name.into_str() }
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
        library: CArc<LibContext>,
        input: Self::InputArg,
        args: &Args,
    ) -> Result<Self::Instance> {
        let cstr = ReprCString::from(args.to_string());
        let mut out = MuOsInstanceArcBox::uninit();
        let logger = library.as_ref().map(|lib| unsafe { lib.get_logger() });
        let res =
            (self.descriptor.create)(&cstr, input.into(), library.into_opaque(), logger, &mut out);
        unsafe { from_int_result(res, out) }
    }
}
