![](docs/logo.png)

![build and test](https://github.com/memflow/memflow/workflows/Build%20and%20test/badge.svg?branch=dev)
[![codecov](https://codecov.io/gh/memflow/memflow/branch/master/graph/badge.svg?token=XT7R158N6W)](https://codecov.io/gh/memflow/memflow)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Discord](https://img.shields.io/discord/738739624976973835?color=%20%237289da&label=Discord)](https://discord.gg/afsEtMR)

# memflow - machine introspection framework

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
