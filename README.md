![](docs/logo.png)

![build and test](https://github.com/memflow/memflow/workflows/Build%20and%20test/badge.svg?branch=dev)
[![codecov](https://codecov.io/gh/memflow/memflow/branch/master/graph/badge.svg?token=XT7R158N6W)](https://codecov.io/gh/memflow/memflow)

memflow - machine introspection framework

#

## building
- install capnp package (soon to be removed for default builds)
- run `cargo build --release --all --all-features` to build everything
- run `cargo build --release --all --all-features --examples` to build all examples
- run `cargo test --all --all-features` to run all tests
- run ... to run all benchmarks
- run `cargo clippy --all-targets --all-features -- -D warnings` to run clippy linting on everything

## documentation
- run `cargo doc --workspace --no-deps --all-features --open` to compile and open the documentation

## usage
- run `cargo run --release -- -c qemu_procfs -vvv` to run the cli on qemu
