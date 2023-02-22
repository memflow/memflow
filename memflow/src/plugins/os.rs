use crate::cglue::{result::from_int_result, *};
use crate::error::*;
use crate::mem::memory_view::*;
use crate::mem::phys_mem::*;
use crate::mem::virt_translate::*;
use crate::os::keyboard::*;
use crate::os::process::*;
use crate::os::root::*;

use super::LibArc;
use super::{
    args::split_str_args, Args, ConnectorInstanceArcBox, LibContext, Loadable, PluginDescriptor,
    TargetInfo,
};

use cglue::trait_group::c_void;

pub type OptionArchitectureIdent<'a> = Option<&'a crate::architecture::ArchitectureIdent>;

cglue_trait_group!(OsInstance, { Os, Clone }, { PhysicalMemory, MemoryView, VirtualTranslate, OsKeyboard });
pub type MuOsInstanceArcBox<'a> = std::mem::MaybeUninit<OsInstanceArcBox<'a>>;

cglue_trait_group!(ProcessInstance, { Process, MemoryView }, { VirtualTranslate });
cglue_trait_group!(IntoProcessInstance, { Process, MemoryView, Clone }, { VirtualTranslate });

/// This creates a cglue plugin instance from the given [`Os`] object.
/// In the future this also might enable features (like caching) based on the input `args`.
pub fn create_instance<T: Send + 'static + Os>(
    conn: T,
    lib: LibArc,
    _args: &OsArgs,
) -> OsInstanceArcBox<'static>
where
    (T, LibArc): Into<OsInstanceBaseArcBox<'static, T, c_void>>,
{
    group_obj!((conn, lib) as OsInstance)
}

#[repr(C)]
#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct OsArgs {
    pub target: Option<ReprCString>,
    pub extra_args: Args,
}

impl std::str::FromStr for OsArgs {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut iter = split_str_args(s, ':');

        let target = iter
            .next()
            .and_then(|s| if s.is_empty() { None } else { Some(s.into()) });

        Ok(Self {
            target,
            extra_args: iter.next().unwrap_or("").parse()?,
        })
    }
}

impl OsArgs {
    pub fn new(target: Option<&str>, extra_args: Args) -> Self {
        Self {
            target: target.map(<_>::into),
            extra_args,
        }
    }
}

pub type OsDescriptor = PluginDescriptor<LoadableOs>;

pub struct LoadableOs {
    descriptor: PluginDescriptor<Self>,
}

impl Loadable for LoadableOs {
    type Instance = OsInstanceArcBox<'static>;
    type InputArg = Option<ConnectorInstanceArcBox<'static>>;
    type CInputArg = COption<ConnectorInstanceArcBox<'static>>;
    type ArgsType = OsArgs;

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
                    Error(ErrorOrigin::Connector, ErrorKind::NotSupported).log_error(format!(
                        "Os-Plugin `{}` did not return any help text.",
                        self.ident()
                    ))
                })
            }
            None => Err(
                Error(ErrorOrigin::Connector, ErrorKind::NotSupported).log_error(format!(
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
        args: Option<&OsArgs>,
    ) -> Result<Self::Instance> {
        let mut out = MuOsInstanceArcBox::uninit();
        let logger = library.as_ref().map(|lib| unsafe { lib.get_logger() });
        let res =
            (self.descriptor.create)(args, input.into(), library.into_opaque(), logger, &mut out);
        unsafe { from_int_result(res, out) }
    }
}
