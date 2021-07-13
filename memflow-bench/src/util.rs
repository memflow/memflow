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

    if cache_size > 0 {
        args = args.insert(
            "pagecache",
            &format!("page:{}kb", (cache_size + 1023) / 1024),
        );
    } else {
        args = args.insert("pagecache", "none")
    }

    let ret = if conn_name == "" {
        inventory.builder().os(os_name).args(args).build()
    } else {
        inventory
            .builder()
            .connector(conn_name)
            .os(os_name)
            .args(args)
            .build()
    }?;

    log::set_max_level(filter);

    Ok(ret)
}

pub fn find_proc<T: Os>(os: &mut T) -> Result<(<T as OsInner<'_>>::ProcessType, ModuleInfo)> {
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
        .ok_or(ErrorKind::NotFound.into())
}
