# memflow
[![Crates.io](https://img.shields.io/crates/v/memflow.svg)](https://crates.io/crates/memflow)
![build and test](https://github.com/memflow/memflow/workflows/Build%20and%20test/badge.svg?branch=dev)
[![codecov](https://codecov.io/gh/memflow/memflow/branch/master/graph/badge.svg?token=XT7R158N6W)](https://codecov.io/gh/memflow/memflow)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Discord](https://img.shields.io/discord/738739624976973835?color=%20%237289da&label=Discord)](https://discord.gg/afsEtMR)

## physical memory introspection framework

memflow is a library that allows live memory introspection of running systems and their snapshots. Due to its modular approach it trivial to support almost any scenario where Direct Memory Access is available.

The very core of the library is a [PhysicalMemory](https://docs.rs/memflow/latest/memflow/mem/phys_mem/trait.PhysicalMemory.html) that provides direct memory access in an abstract environment. This object can be defined both statically, and dynamically with the use of the `inventory` feature. If `inventory` is enabled, it is possible to dynamically load libraries that provide Direct Memory Access.

Through the use of OS abstraction layers, like [memflow-win32](https://github.com/memflow/memflow/tree/master/memflow-win32), users can gain access to virtual memory of individual processes by creating objects that implement [VirtualMemory](https://docs.rs/memflow/latest/memflow/mem/virt_mem/trait.VirtualMemory.html).

Bridging the two is done by a highly throughput optimized virtual address translation function, which allows for crazy fast memory transfers at scale.

The core is architecture-independent (as long as addresses fit in 64-bits), and currently, both 32, and 64-bit versions of the x86 family are available to be used.

For non-rust libraries, it is possible to use the [FFI](https://github.com/memflow/memflow/tree/master/memflow-ffi) to interface with the library.

In the repository, you can find various examples available (which use the memflow-win32 layer)

## Building from source

To build all projects in the memflow workspace:

`cargo build --release --workspace`

To build all examples:

`cargo build --release --workspace --examples`

Run all tests:

`cargo test --workspace`

Execute the benchmarks:

`cargo bench`

## Documentation

Extensive code documentation can be found at [docs.rs](https://docs.rs/memflow/0.1/).

An additional getting started guide as well as a higher level
explanation of the inner workings of memflow can be found at [memflow.github.io](https://memflow.github.io).

If you decide to build the latest documentation you can do it by issuing:

`cargo doc --workspace --no-deps --open`

## Basic usage

You can either run one of the examples with `cargo run --release --example`. Pass nothing to get a list of examples.

Some connectors like `qemu_procfs` will require elevated privileges. See the Connectors section of this Readme for more information.

To simplify running examples, tests, and benchmarks through different connectors we added a simple cargo runner script for Linux to this repository.
Simply set any of the following environment variables when running the `cargo` command to elevate privileges:

- `RUST_SUDO` will start the resulting binary via sudo.
- `RUST_SETPTRACE` will enable PTRACE permissions on the resulting binary before executing it.

Alternatively, you can run the benchmarks via `cargo bench` (can pass regex filters). Win32 benchmarks currently work only on Linux.

## Running Examples

All examples support the memflow connector inventory system.
You will have to install at least one `connector` to use the examples.

To install a connector just use the [memflowup](https://github.com/memflow/memflowup) utility,
or, head over to the corresponding repository and install them via the `install.sh` script.

You will find a folder called `memflow` in any of the following locations:
```
/opt
/lib
/usr/lib/
/usr/local/lib
/lib32
/lib64
/usr/lib32
/usr/lib64
/usr/local/lib32
/usr/local/lib64
```

On Windows, you can put the connector DLL in a folder named `memflow`
that is either in your current PATH or put it in `C:\Users\{Username}\.local\lib\memflow`.
Additionally connectors can be placed in the working directory of the process as well.

Now you can just run the examples by providing the appropriate connector name:

Run memflow\_win32/read\_keys example with a procfs connector:

`RUST_SETPTRACE=1 cargo run --example read_keys -- -vv -c qemu_procfs -a [vmname]`

Run memflow\_win32/read\_bench example with a coredump connector:

`cargo run --example read_bench --release -- -vv -c coredump -a coredump_win10_64bit.raw`

Note: In the examples above the `qemu_procfs` connector requires `'CAP_SYS_PTRACE=ep'` permissions. The runner script in this repository will set the appropriate flags when the `RUST_SETPTRACE` environment variable is passed to it.

## Compilation support

| target        | build              | tests              | benches            | compiles on stable |
|---------------|--------------------|--------------------|--------------------|--------------------|
| linux x86_64  | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| mac x86_64    | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| win x86_64    | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| linux aarch64 | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| no-std        | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :x:                |

## Target support

memflow-win32 is tested on the latest Windows 10 versions all the way down to Windows NT 4.0. If you found a version that does not work please submit an issue with the major/minor version as well as the build number.

## Connectors

All examples provided in this repository are using the inventory to
dynamically load a connector at runtime. When using the library programmatically it is possible to just statically link a connector into the code.

Some connectors also require different permissions. Please refer to the individual connector repositories for more information.

These are the currently officially existing connectors:
- [qemu_procfs](https://github.com/memflow/memflow-qemu-procfs)
- [kvm](https://github.com/memflow/memflow-kvm)
- [pcileech](https://github.com/memflow/memflow-pcileech)
- [coredump](https://github.com/memflow/memflow-coredump)

In case you write your own connector please hit us up with a merge request so we can maintain a list of third-party connectors as well.

## Road map / Future Development

- Provide a rust native connector for PCILeech based hardware
- Provide a UEFI Demo
- Linux target support

## Acknowledgements
- [CasualX](https://github.com/casualx/) for his wonderful pelite crate
- [ufrisk](https://github.com/ufrisk/) for his prior work on the subject and many inspirations

## Contributing

Please check [CONTRIBUTE.md](CONTRIBUTE.md)
