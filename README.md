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
