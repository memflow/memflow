# memflow
[![Crates.io](https://img.shields.io/crates/v/memflow.svg)](https://crates.io/crates/memflow)
![build and test](https://github.com/memflow/memflow/workflows/Build%20and%20test/badge.svg?branch=dev)
[![codecov](https://codecov.io/gh/memflow/memflow/branch/master/graph/badge.svg?token=XT7R158N6W)](https://codecov.io/gh/memflow/memflow)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Discord](https://img.shields.io/discord/738739624976973835?color=%20%237289da&label=Discord)](https://discord.gg/afsEtMR)

## physical memory introspection framework

memflow is a library that allows live memory introspection of running systems and their snapshots. Due to its modular approach, it is trivial to support almost any scenario where Direct Memory Access is available.

The very core of the library is a [PhysicalMemory](https://docs.rs/memflow/latest/memflow/mem/phys_mem/trait.PhysicalMemory.html) that provides direct memory access in an abstract environment. This object that can be defined both statically, and dynamically with the use of the `plugins` feature. If `plugins` is enabled, it is possible to dynamically load libraries that provide Direct Memory Access.

Through the use of OS abstraction layers, like [memflow-win32](https://github.com/memflow/memflow/tree/master/memflow-win32), users can gain access to virtual memory of individual processes by creating objects that implement [VirtualMemory](https://docs.rs/memflow/latest/memflow/mem/virt_mem/trait.VirtualMemory.html).

Bridging the two is done by a highly throughput optimized virtual address translation function, which allows for crazy fast memory transfers at scale.

The core is architecture-independent (as long as addresses fit in 64-bits), and currently, both 32, and 64-bit versions of the x86 family are available to be used.

For non-rust libraries, it is possible to use the [FFI](https://github.com/memflow/memflow/tree/master/memflow-ffi) to interface with the library.

In the repository, you can find various examples available (which use the memflow-win32 layer)

## Getting started

Make sure that your rustc version is at least `1.51.0` or newer.

memflow uses a plugin based approach and is capable of loading different physical memory backends (so-called [`connectors`](#connectors)) at runtime. On top of the physical memory backends memflow is also capable of loading plugins for interfacing with a specific target OS at runtime.

To get started, you want to at least install one connector. On Linux based hosts you can simply execute the `install.sh` found in each connector repository to install the connector. When running `./install.sh --system` the connector is installed system-wide. When omitting the `--system` argument the connector is just installed for the current user.

When using the memflow-daemon it is required to install each connector system-wide (or at least under the root user) so the daemon can access it. Some connectors also require elevated privileges, which might also require them to be accessible from the root user.

Note that all connectors should be built with the `--all-features` flag to be accessible as a dynamically loaded plugin.

The recommended installation locations for connectors on Linux are:
```
/usr/lib/memflow/libmemflow_xxx.so
$HOME/.local/lib/memflow/libmemflow_xxx.so
```

The recommended installation locations for connectors on Windows are:
```
[Username]/Documents/memflow/libmemflow_xxx.dll
```

Additionally, connectors can be placed in any directory of the environment PATH or the working directory of the program as well.

For Windows target support the `win32` plugin has to be built:
```bash
cargo build --release --all-features --workspace
```

This will create the OS plugin in `target/release/libmemflow_win32.so` which has to be copied to one of the plugin folders mentioned above.

For more information about how to get started with memflow please head over to the YouTube series produced by [h33p](https://github.com/h33p/):

- [memflow basics](https://www.youtube.com/playlist?list=PLrC4R7zDrxB3RSJQk9ahmXNCw8m3pdP6z)
- [memflow applied](https://www.youtube.com/watch?v=xJXkRMy71dc&list=PLrC4R7zDrxB17iWCy9eEdCaluCR3Bkn8q)

## Running Examples

You can either run one of the examples with `cargo run --release --example`. Pass nothing to get a list of examples.

Some connectors like `qemu_procfs` will require elevated privileges. Refer to the readme of the connector for additional information on their required access rights.

To simplify running examples, tests, and benchmarks through different connectors, we added a simple cargo runner script for Linux to this repository.
Simply set any of the following environment variables when running the `cargo` command to elevate privileges:

- `RUST_SUDO` will start the resulting binary via sudo.
- `RUST_SETPTRACE` will enable PTRACE permissions on the resulting binary before executing it.

Alternatively, you can run the benchmarks via `cargo bench` (can pass regex filters). Win32 benchmarks currently work only on Linux.

All examples support the memflow connector `plugins` inventory system.
You will have to install at least one `connector` to use the examples. Refer to the [getting started](#getting-started) section for more details.

Run memflow\_win32/read\_keys example with a procfs connector:

`RUST_SETPTRACE=1 cargo run --example read_keys -- -vv -c qemu_procfs -a [vmname]`

Run memflow\_win32/read\_bench example with a coredump connector:

`cargo run --example read_bench --release -- -vv -c coredump -a coredump_win10_64bit.raw`

Note: In the examples above the `qemu_procfs` connector requires `'CAP_SYS_PTRACE=ep'` permissions. The runner script in this repository will set the appropriate flags when the `RUST_SETPTRACE` environment variable is passed to it.

## Documentation

Extensive code documentation can be found at [docs.rs](https://docs.rs/memflow/0.1/).

An additional getting started guide as well as a higher level
explanation of the inner workings of memflow can be found at [memflow.github.io](https://memflow.github.io).

If you decide to build the latest documentation you can do it by issuing:

`cargo doc --workspace --no-deps --open`

## Compilation support

memflow currently requires at least rustc version `1.51.0` or newer.

| target        | build              | tests              | benches            | compiles on stable |
|---------------|--------------------|--------------------|--------------------|--------------------|
| linux x86_64  | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| mac x86_64    | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| win x86_64    | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| linux aarch64 | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| no-std        | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :x:                |

## Target support

memflow-win32 is tested on the latest Windows 11 and Windows 10 versions all the way down to Windows NT 4.0. If you found a version that does not work please submit an issue with the major/minor version as well as the build number.

## Connectors

All examples provided in this repository are using the `plugins` inventory to
dynamically load a connector at runtime. When using the library programmatically it is possible to just statically link a connector into the code.

Some connectors also require different permissions. Please refer to the individual connector repositories for more information.

These are the currently officially existing connectors:
- [qemu_procfs](https://github.com/memflow/memflow-qemu-procfs)
- [kvm](https://github.com/memflow/memflow-kvm)
- [pcileech](https://github.com/memflow/memflow-pcileech)
- [coredump](https://github.com/memflow/memflow-coredump)

In case you write your own connector please hit us up with a pull request so we can maintain a list of third-party connectors as well.

## Acknowledgements
- [CasualX](https://github.com/casualx/) for his wonderful pelite crate
- [ufrisk](https://github.com/ufrisk/) for his prior work on the subject and many inspirations

## Contributing

Please check [CONTRIBUTE.md](CONTRIBUTE.md)