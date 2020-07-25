![](docs/logo.png)

![build and test](https://github.com/memflow/memflow/workflows/Build%20and%20test/badge.svg?branch=dev)
[![codecov](https://codecov.io/gh/memflow/memflow/branch/master/graph/badge.svg?token=XT7R158N6W)](https://codecov.io/gh/memflow/memflow)

memflow - machine introspection framework

#

## building
- run `cargo build --release --all --all-features` to build everything
- run `cargo build --release --all --all-features --examples` to build all examples
- run `cargo test --all --all-features` to run all tests
- run `cargo bench` to run all benchmarks
- run `cargo clippy --all-targets --all-features -- -D warnings` to run clippy linting on everything

## documentation
- run `cargo doc --workspace --no-deps --all-features --open` to compile and open the documentation

## usage
- setup ptrace permissions with `setperms.sh` script, or run `setcap 'CAP_SYS_PTRACE=ep'` on target executable
- run one of the examples with `cargo run --release --example`
- or run the benchmarks `cargo bench` (can pass regex filters)

## Running Examples

All examples support the memflow plugin inventory system.

Run memflow_win32/read_keys example with a procfs connector:

`cargo run --example read_keys -- -vv -i target/release -c qemu_procfs -a [vmname]`

Run memflow_win32/read_bench example with a coredump connector:

`cargo run --example read_bench --release -- -vv -i target/release -c coredump -a coredump_win10_64bit.raw`
