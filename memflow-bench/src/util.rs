use memflow::prelude::v1::*;

pub fn build_os(
    conn_name: &str,
    cache_size: usize,
    os_name: &str,
    use_tlb: bool,
) -> Result<OsInstanceArcBox<'static>> {
    // this workaround is to prevent loaded libraries
    // from spitting out to much log information and skewing benchmarks
    let filter = log::max_level();
    log::set_max_level(log::Level::Debug.to_level_filter());

    let inventory = Inventory::scan();

    log::set_max_level(log::Level::Error.to_level_filter());

    let mut args = Args::new();

    if !use_tlb {
        args = args.insert("vatcache", "none");
    }

    let page_cache_params = if cache_size > 0 {
        Some(ConnectorMiddlewareArgs::new().cache_size(cache_size))
    } else {
        None
    };

    let conn_args = ConnectorArgs::new(None, Default::default(), page_cache_params);
    let args = OsArgs::new(None, args);

    let ret = if conn_name.is_empty() {
        inventory.builder().os(os_name).args(args).build()
    } else {
        inventory
            .builder()
            .connector(conn_name)
            .args(conn_args)
            .os(os_name)
            .args(args)
            .build()
    }?;

    log::set_max_level(filter);

    Ok(ret)
}

pub fn find_proc<T: Os>(os: &mut T) -> Result<(<T as Os>::ProcessType<'_>, ModuleInfo)> {
    let infos = os.process_info_list()?;

    let mut data = None;

    for info in infos {
        if let Ok(mut proc) = os.process_by_info(info.clone()) {
            let mut module = None;

            proc.module_list_callback(
                None,
                (&mut |info: ModuleInfo| {
                    if info.size > 0x1000 {
                        module = Some(info);
                    }
                    module.is_none()
                })
                    .into(),
            )?;

            if let Some(module) = module {
                data = Some((info, module));
                break;
            }
        }
    }

    data.and_then(move |(info, module)| Some((os.process_by_info(info).ok()?, module)))
        .ok_or_else(|| ErrorKind::NotFound.into())
}
