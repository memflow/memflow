use crate::cglue::{
    result::{from_int_result, from_int_result_empty},
    *,
};
use crate::error::*;
use crate::mem::phys_mem::*;
use crate::types::{cache::TimedCacheValidator, size};
use std::time::Duration;

use super::{
    args::split_str_args, Args, LibArc, LibContext, Loadable, OsInstanceArcBox, PluginDescriptor,
    TargetInfo,
};

use crate::connector::cpu_state::*;
use cglue::trait_group::c_void;

cglue_trait_group!(ConnectorInstance<'a>, { PhysicalMemory, Clone }, { ConnectorCpuStateInner<'a> });
pub type MuConnectorInstanceArcBox<'a> = std::mem::MaybeUninit<ConnectorInstanceArcBox<'a>>;

pub fn create_instance<T: Send + 'static + PhysicalMemory>(
    conn: T,
    lib: LibArc,
    args: &ConnectorArgs,
) -> ConnectorInstanceArcBox<'static>
where
    (T, LibArc): Into<ConnectorInstanceBaseArcBox<'static, T, c_void>>,
    (
        CachedPhysicalMemory<'static, T, TimedCacheValidator>,
        LibArc,
    ): Into<
        ConnectorInstanceBaseArcBox<
            'static,
            CachedPhysicalMemory<'static, T, TimedCacheValidator>,
            c_void,
        >,
    >,
{
    if let COption::Some(cache) = args.page_cache {
        let mut builder = CachedPhysicalMemory::builder(conn);

        builder = if cache.page_size > 0 {
            builder.page_size(cache.page_size)
        } else {
            builder.page_size(size::kb(4))
        };

        if cache.size > 0 {
            builder = builder.cache_size(cache.size);
        }

        if cache.validity_time > 0 {
            builder = builder.validator(TimedCacheValidator::new(
                Duration::from_millis(cache.validity_time).into(),
            ))
        }

        let conn = builder.build().unwrap();

        group_obj!((conn, lib) as ConnectorInstance)
    } else {
        let obj = (conn, lib).into();
        obj.into_opaque()
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct PageCacheParams {
    pub size: usize,
    pub validity_time: u64,
    pub page_size: usize,
}

impl PageCacheParams {
    pub fn new(size: usize, validity_time: u64, page_size: usize) -> Self {
        Self {
            size,
            validity_time,
            page_size,
        }
    }
}

impl std::str::FromStr for PageCacheParams {
    type Err = crate::error::Error;

    fn from_str(vargs: &str) -> Result<Self> {
        let mut sp = vargs.splitn(2, ',');
        let (size, time, page_size) = (
            sp.next().ok_or_else(|| {
                Error(ErrorOrigin::OsLayer, ErrorKind::Configuration)
                    .log_error("Failed to parse Page Cache size")
            })?,
            sp.next().unwrap_or("0"),
            sp.next().unwrap_or("0"),
        );

        let (size, size_mul) = {
            let mul_arr = &[
                (size::kb(1), ["kb", "k"]),
                (size::mb(1), ["mb", "m"]),
                (size::gb(1), ["gb", "g"]),
            ];

            mul_arr
                .iter()
                .flat_map(|(m, e)| e.iter().map(move |e| (*m, e)))
                .find_map(|(m, e)| {
                    if size.to_lowercase().ends_with(e) {
                        Some((size.trim_end_matches(e), m))
                    } else {
                        None
                    }
                })
                .ok_or_else(|| {
                    Error(ErrorOrigin::OsLayer, ErrorKind::Configuration)
                        .log_error("Invalid Page Cache size unit (or none)!")
                })?
        };

        let size = usize::from_str_radix(size, 16).map_err(|_| {
            Error(ErrorOrigin::OsLayer, ErrorKind::Configuration)
                .log_error("Failed to parse Page Cache size")
        })?;

        let size = size * size_mul;

        let validity_time = time.parse::<u64>().map_err(|_| {
            Error(ErrorOrigin::OsLayer, ErrorKind::Configuration)
                .log_error("Failed to parse Page Cache validity time")
        })?;

        let page_size = usize::from_str_radix(page_size, 16).map_err(|_| {
            Error(ErrorOrigin::OsLayer, ErrorKind::Configuration)
                .log_error("Failed to parse Page size for an entry")
        })?;

        Ok(Self {
            size,
            validity_time,
            page_size,
        })
    }
}

#[repr(C)]
#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct ConnectorArgs {
    pub target: Option<ReprCString>,
    pub extra_args: Args,
    pub page_cache: COption<PageCacheParams>,
}

impl std::str::FromStr for ConnectorArgs {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut iter = split_str_args(s);

        let target = iter
            .next()
            .and_then(|s| if s.is_empty() { None } else { Some(s.into()) });

        Ok(Self {
            target,
            extra_args: iter.next().unwrap_or("").parse()?,
            page_cache: if let Some(s) = iter.next() {
                Some(PageCacheParams::from_str(s)?)
            } else {
                Some(Default::default())
            }
            .into(),
        })
    }
}

impl ConnectorArgs {
    pub fn new(
        target: Option<&str>,
        extra_args: Args,
        page_cache: Option<PageCacheParams>,
    ) -> Self {
        Self {
            target: target.map(<_>::into),
            extra_args,
            page_cache: page_cache.into(),
        }
    }
}

pub type ConnectorDescriptor = PluginDescriptor<LoadableConnector>;

pub struct LoadableConnector {
    descriptor: PluginDescriptor<Self>,
}

impl Loadable for LoadableConnector {
    type Instance = ConnectorInstanceArcBox<'static>;
    type InputArg = Option<OsInstanceArcBox<'static>>;
    type CInputArg = COption<OsInstanceArcBox<'static>>;
    type ArgsType = ConnectorArgs;

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
        args: Option<&ConnectorArgs>,
    ) -> Result<Self::Instance> {
        let mut out = MuConnectorInstanceArcBox::uninit();
        let logger = library.as_ref().map(|lib| unsafe { lib.get_logger() });
        let res =
            (self.descriptor.create)(args, input.into(), library.into_opaque(), logger, &mut out);
        unsafe { from_int_result(res, out) }
    }
}
