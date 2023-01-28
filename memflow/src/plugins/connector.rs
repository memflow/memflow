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

cglue_trait_group!(ConnectorInstance, { PhysicalMemory, Clone }, { ConnectorCpuState });
pub type MuConnectorInstanceArcBox<'a> = std::mem::MaybeUninit<ConnectorInstanceArcBox<'a>>;

/// This creates a cglue plugin instance from the given [`PhysicalMemory`] object.
/// This also configures caching based on the provided input `args`.
pub fn create_instance<T: Send + 'static + PhysicalMemory>(
    conn: T,
    lib: LibArc,
    args: &ConnectorArgs,
    no_default_cache: bool,
) -> ConnectorInstanceArcBox<'static>
// TODO: get rid of these trait bounds
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
    (
        DelayedPhysicalMemory<CachedPhysicalMemory<'static, T, TimedCacheValidator>>,
        LibArc,
    ): Into<
        ConnectorInstanceBaseArcBox<
            'static,
            DelayedPhysicalMemory<CachedPhysicalMemory<'static, T, TimedCacheValidator>>,
            c_void,
        >,
    >,
{
    let default_cache = !no_default_cache;
    let middleware = if args.middleware.is_some() || default_cache {
        Some(args.middleware.as_ref().copied().unwrap_or_default())
    } else {
        None
    };
    if let Some(middleware) = middleware {
        let mut builder = CachedPhysicalMemory::builder(conn);

        builder = if middleware.cache_page_size > 0 {
            builder.page_size(middleware.cache_page_size)
        } else {
            builder.page_size(size::kb(4))
        };

        if middleware.cache_size > 0 {
            builder = builder.cache_size(middleware.cache_size);
        }

        if middleware.cache_validity_time > 0 {
            builder = builder.validator(TimedCacheValidator::new(
                Duration::from_millis(middleware.cache_validity_time).into(),
            ))
        }

        // TODO: optional features not forwarded?
        let conn = builder.build().unwrap();

        if middleware.delay > 0 {
            let conn = DelayedPhysicalMemory::builder(conn)
                .delay(Duration::from_millis(middleware.delay))
                .build()
                .unwrap();
            group_obj!((conn, lib) as ConnectorInstance)
        } else {
            group_obj!((conn, lib) as ConnectorInstance)
        }
    } else {
        // this is identical to: `group_obj!((conn, lib) as ConnectorInstance)`
        let obj = (conn, lib).into();
        obj.into_opaque()
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct ConnectorMiddlewareArgs {
    pub cache_size: usize,
    pub cache_validity_time: u64,
    pub cache_page_size: usize,
    pub delay: u64,
}

impl ConnectorMiddlewareArgs {
    pub fn new(
        cache_size: usize,
        cache_validity_time: u64,
        cache_page_size: usize,
        delay: u64,
    ) -> Self {
        Self {
            cache_size,
            cache_validity_time,
            cache_page_size,
            delay,
        }
    }
}

impl std::str::FromStr for ConnectorMiddlewareArgs {
    type Err = crate::error::Error;

    fn from_str(vargs: &str) -> Result<Self> {
        let args: Args = vargs.parse()?;

        let (size, time, page_size) = (
            args.get("cache_size").ok_or_else(|| {
                Error(ErrorOrigin::OsLayer, ErrorKind::Configuration)
                    .log_error("Failed to parse Page Cache size")
            })?,
            args.get("cache_time").unwrap_or("0"),
            args.get("cache_page_size").unwrap_or("0"),
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

        let cache_size = size * size_mul;

        let cache_validity_time = time.parse::<u64>().map_err(|_| {
            Error(ErrorOrigin::OsLayer, ErrorKind::Configuration)
                .log_error("Failed to parse Page Cache validity time")
        })?;

        let cache_page_size = usize::from_str_radix(page_size, 16).map_err(|_| {
            Error(ErrorOrigin::OsLayer, ErrorKind::Configuration)
                .log_error("Failed to parse Page size for an entry")
        })?;

        let delay = args
            .get("delay")
            .unwrap_or("0")
            .parse::<u64>()
            .map_err(|_| {
                Error(ErrorOrigin::OsLayer, ErrorKind::Configuration)
                    .log_error("Failed to parse delay configuration")
            })?;

        Ok(Self {
            cache_size,
            cache_validity_time,
            cache_page_size,
            delay,
        })
    }
}

#[repr(C)]
#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct ConnectorArgs {
    pub target: Option<ReprCString>,
    pub extra_args: Args,
    pub middleware: COption<ConnectorMiddlewareArgs>,
}

impl std::str::FromStr for ConnectorArgs {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut iter = split_str_args(s);

        let target = iter
            .next()
            .and_then(|s| if s.is_empty() { None } else { Some(s.into()) });

        let extra_args = iter.next().unwrap_or("").parse()?;

        let middleware = if let Some(s) = iter.next() {
            // allow user to see the parse error
            Some(s.parse()?)
        } else {
            None
        }
        .into();

        Ok(Self {
            target,
            extra_args,
            middleware,
        })
    }
}

impl ConnectorArgs {
    pub fn new(
        target: Option<&str>,
        extra_args: Args,
        middleware: Option<ConnectorMiddlewareArgs>,
    ) -> Self {
        Self {
            target: target.map(<_>::into),
            extra_args,
            middleware: middleware.into(),
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
                    Error(ErrorOrigin::Connector, ErrorKind::NotSupported).log_error(format!(
                        "Connector `{}` did not return any help text.",
                        self.ident()
                    ))
                })
            }
            None => Err(
                Error(ErrorOrigin::Connector, ErrorKind::NotSupported).log_error(format!(
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
                Error(ErrorOrigin::Connector, ErrorKind::NotSupported).log_error(format!(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn connector_args_parse() {
        let args: ConnectorArgs =
            "target:extra=value:cache_size=1kb,cache_time=10,cache_page_size=1000"
                .parse()
                .expect("unable to parse args");
        assert_eq!(args.target.unwrap(), ReprCString::from("target"));
        assert_eq!(args.extra_args.get("extra").unwrap(), "value");
        assert_eq!(args.middleware.unwrap().cache_size, 1024);
        assert_eq!(args.middleware.unwrap().cache_validity_time, 10);
        assert_eq!(args.middleware.unwrap().cache_page_size, 0x1000);
    }
}
