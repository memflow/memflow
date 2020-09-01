![](docs/logo.png)

[![Crates.io](https://img.shields.io/crates/v/memflow-core.svg)](https://crates.io/crates/memflow-core)
![build and test](https://github.com/memflow/memflow/workflows/Build%20and%20test/badge.svg?branch=dev)
[![codecov](https://codecov.io/gh/memflow/memflow/branch/master/graph/badge.svg?token=XT7R158N6W)](https://codecov.io/gh/memflow/memflow)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Discord](https://img.shields.io/discord/738739624976973835?color=%20%237289da&label=Discord)](https://discord.gg/afsEtMR)

# memflow - machine introspection framework

memflow is a library that allows live memory introspection of running systems and their snapshots. Due to its modular approach it trivial to support almost any scenario where Direct Memory Access is available.

The very core of the library is a [PhysicalMemory](https://docs.rs/memflow-core/latest/memflow_core/mem/phys_mem/trait.PhysicalMemory.html) that provides direct memory access in an abstract environment. This object that can be defined both statically, and dynamically with the use of the `inventory` feature. If `inventory` is enabled, it is possible to dynamically load libraries that provide Direct Memory Access.

Through the use of OS abstraction layers, like [memflow-win32](https://github.com/memflow/memflow/tree/master/memflow-win32), user can gain access to virtual memory of individual processes, by creating objects that implement [VirtualMemory](https://docs.rs/memflow-core/latest/memflow_core/mem/virt_mem/trait.VirtualMemory.html).

Bridging the two is done by a highly throughput optimized virtual address translation function, which allows for crazy fast memory transfers on scale.

The core is architecture independent (as long as addresses fit in 64-bits), and currently both 32, and 64-bit versions of the x86 family are available to be used.

For non-rust libraries, it is possible to use the [FFI](https://github.com/memflow/memflow/tree/master/memflow-core-ffi) to interface with the library.

In the repository you can find various examples available (which use memflow-win32 layer)

## building

- run `cargo build --release --workspace --all-features` to build everything
- run `cargo build --release --workspace --all-features --examples` to build all examples
- run `cargo test --workspace --all-features` to run all tests
- run `cargo bench` to run all benchmarks
- run `cargo clippy --all-targets --all-features -- -D warnings` to run clippy linting on everything

## documentation
- run `cargo doc --workspace --no-deps --all-features --open` to compile and open the documentation

## usage
- run one of the examples with `cargo run --release --example` (pass nothing to get a list of them).
- if ran with `qemu_procfs` connector, the runner will request root permissions to set `'CAP_SYS_PTRACE=ep'` on the executables
- or run the benchmarks `cargo bench` (can pass regex filters). Win32 benchmarks currently work only on Linux.

## Running Examples

All examples support the memflow connector inventory system.

Run memflow\_win32/read\_keys example with a procfs connector:

`cargo run --example read_keys -- -vv -i target/release -c qemu_procfs -a [vmname]`

Run memflow\_win32/read\_bench example with a coredump connector:

`cargo run --example read_bench --release -- -vv -i target/release -c coredump -a coredump_win10_64bit.raw`

## Documentation on Windows

`dummy` connector is used throughout the documentation, which uses the `x86_64` crate, and it does not compile on windows without nightly feature set.
