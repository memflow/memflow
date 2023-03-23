use ::log::info;
use ::std::time::Duration;

use crate::cglue::{
    result::{from_int_result, from_int_result_empty},
    *,
};
use crate::error::*;
use crate::mem::phys_mem::*;
use crate::types::{cache::TimedCacheValidator, size};

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
{
    // check if user explicitly enabled caching or alternatively fall back to auto configuration of the connector
    let use_cache = Option::<bool>::from(args.middleware_args.cache).unwrap_or(!no_default_cache);
    let conn = if use_cache {
        let cache_page_size = if args.middleware_args.cache_page_size > 0 {
            args.middleware_args.cache_page_size
        } else {
            size::kb(4)
        };

        info!("Inserting `CachedPhysicalMemory` middleware with size={}, validity_time={}, page_size={}",
            args.middleware_args.cache_size, args.middleware_args.cache_validity_time, cache_page_size);

        let mut builder = CachedPhysicalMemory::builder(conn).page_size(cache_page_size);

        if args.middleware_args.cache_size > 0 {
            builder = builder.cache_size(args.middleware_args.cache_size);
        }

        if args.middleware_args.cache_validity_time > 0 {
            builder = builder.validator(TimedCacheValidator::new(
                Duration::from_millis(args.middleware_args.cache_validity_time).into(),
            ))
        }

        let conn = builder.build().unwrap();
        group_obj!((conn, lib.clone()) as ConnectorInstance)
    } else {
        group_obj!((conn, lib.clone()) as ConnectorInstance)
    };

    let conn = if args.middleware_args.delay > 0 {
        info!(
            "Inserting `DelayedPhysicalMemory` middleware with delay={}",
            args.middleware_args.delay
        );

        let conn = DelayedPhysicalMemory::builder(conn)
            .delay(Duration::from_micros(args.middleware_args.delay))
            .build()
            .unwrap();
        group_obj!((conn, lib.clone()) as ConnectorInstance)
    } else {
        conn
    };

    if args.middleware_args.metrics {
        info!("Inserting `PhysicalMemoryMetrics` middleware",);
        let conn = PhysicalMemoryMetrics::new(conn);
        group_obj!((conn, lib) as ConnectorInstance)
    } else {
        conn
    }

    // TODO: optional features not forwarded?
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct ConnectorMiddlewareArgs {
    pub cache: COption<bool>,
    pub cache_size: usize,
    pub cache_validity_time: u64,
    pub cache_page_size: usize,

    pub delay: u64,

    pub metrics: bool,
}

impl ConnectorMiddlewareArgs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cache(mut self, cache: bool) -> Self {
        self.cache = COption::Some(cache);
        self
    }
    pub fn cache_size(mut self, size: usize) -> Self {
        self.cache_size = size;
        self
    }
    pub fn cache_validity_time(mut self, validity_time: u64) -> Self {
        self.cache_validity_time = validity_time;
        self
    }
    pub fn cache_page_size(mut self, page_size: usize) -> Self {
        self.cache_page_size = page_size;
        self
    }

    pub fn delay(mut self, delay: u64) -> Self {
        self.delay = delay;
        self
    }

    pub fn metrics(mut self, metrics: bool) -> Self {
        self.metrics = metrics;
        self
    }
}

impl std::str::FromStr for ConnectorMiddlewareArgs {
    type Err = crate::error::Error;

    fn from_str(vargs: &str) -> Result<Self> {
        let args: Args = vargs.parse()?;

        let (cache, size, time, page_size) = (
            args.get("cache")
                .map(|s| s.to_lowercase() == "true" || s == "1"),
            args.get("cache_size").unwrap_or("0kb"),
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

        let metrics = args
            .get("metrics")
            .map(|s| s.to_lowercase() == "true" || s == "1")
            .unwrap_or_default();

        Ok(Self {
            cache: cache.into(),
            cache_size,
            cache_validity_time,
            cache_page_size,

            delay,

            metrics,
        })
    }
}

#[repr(C)]
#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct ConnectorArgs {
    pub target: Option<ReprCString>,
    pub extra_args: Args,
    pub middleware_args: ConnectorMiddlewareArgs,
}

impl std::str::FromStr for ConnectorArgs {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut iter = split_str_args(s, ':');

        let target = iter
            .next()
            .and_then(|s| if s.is_empty() { None } else { Some(s.into()) });

        let extra_args = iter.next().unwrap_or("").parse()?;

        let middleware_args = if let Some(s) = iter.next() {
            // allow user to see the parse error
            s.parse()?
        } else {
            ConnectorMiddlewareArgs::default()
        };

        Ok(Self {
            target,
            extra_args,
            middleware_args,
        })
    }
}

impl ConnectorArgs {
    pub fn new(
        target: Option<&str>,
        extra_args: Args,
        middleware_args: Option<ConnectorMiddlewareArgs>,
    ) -> Self {
        Self {
            target: target.map(<_>::into),
            extra_args,
            middleware_args: middleware_args.unwrap_or_default(),
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
        assert_eq!(Option::<bool>::from(args.middleware_args.cache), None);
        assert_eq!(args.middleware_args.cache_size, 1024);
        assert_eq!(args.middleware_args.cache_validity_time, 10);
        assert_eq!(args.middleware_args.cache_page_size, 0x1000);
    }

    #[test]
    pub fn connector_args_with_cache() {
        let args: ConnectorArgs =
            "target:extra=value:cache=true,cache_size=1kb,cache_time=10,cache_page_size=1000"
                .parse()
                .expect("unable to parse args");
        assert_eq!(args.target.unwrap(), ReprCString::from("target"));
        assert_eq!(args.extra_args.get("extra").unwrap(), "value");
        assert_eq!(Option::<bool>::from(args.middleware_args.cache), Some(true));
        assert_eq!(args.middleware_args.cache_size, 1024);
        assert_eq!(args.middleware_args.cache_validity_time, 10);
        assert_eq!(args.middleware_args.cache_page_size, 0x1000);
    }

    #[test]
    pub fn connector_args_url() {
        let args: ConnectorArgs = ":device=\"RAWUDP://ip=127.0.0.1:8080\":"
            .parse()
            .expect("unable to parse args");
        assert_eq!(args.target, None);
        assert_eq!(
            args.extra_args.get("device").unwrap(),
            "RAWUDP://ip=127.0.0.1:8080"
        );
    }
}
