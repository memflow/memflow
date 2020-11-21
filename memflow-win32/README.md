# memflow-win32
[![Crates.io](https://img.shields.io/crates/v/memflow.svg)](https://crates.io/crates/memflow)
![build and test](https://github.com/memflow/memflow/workflows/Build%20and%20test/badge.svg?branch=dev)
[![codecov](https://codecov.io/gh/memflow/memflow/branch/master/graph/badge.svg?token=XT7R158N6W)](https://codecov.io/gh/memflow/memflow)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Discord](https://img.shields.io/discord/738739624976973835?color=%20%237289da&label=Discord)](https://discord.gg/afsEtMR)

This crate provides integration for win32 targets for [memflow](https://github.com/memflow/memflow). This library can be used in addition to the memflow core itself read processes, modules, drivers, etc.

Example initializing a win32 target:
```rust
use std::fs::File;
use std::io::Write;

use log::{error, Level};

use memflow::connector::*;
use memflow_win32::win32::{Kernel, Win32OffsetFile};

pub fn main() {
    let connector_name = std::env::args().nth(1).unwrap();
    let connector_args = std::env::args().nth(2).unwrap_or_default();

    // create inventory + connector
    let inventory = unsafe { ConnectorInventory::try_new() }.unwrap();
    let connector = unsafe {
        inventory.create_connector(
            &connector_name,
            &ConnectorArgs::parse(&connector_args).unwrap(),
        )
    }
    .unwrap();

    // initialize kernel
    let kernel = Kernel::builder(connector)
        .build_default_caches()
        .build()
        .unwrap();

    println!("{:?}", kernel);
}
```

Additional examples can be found in the `examples` subdirectory.
