# memflow-bench
[![Crates.io](https://img.shields.io/crates/v/memflow.svg)](https://crates.io/crates/memflow)
![build and test](https://github.com/memflow/memflow/workflows/Build%20and%20test/badge.svg?branch=dev)
[![codecov](https://codecov.io/gh/memflow/memflow/branch/master/graph/badge.svg?token=XT7R158N6W)](https://codecov.io/gh/memflow/memflow)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Discord](https://img.shields.io/discord/738739624976973835?color=%20%237289da&label=Discord)](https://discord.gg/afsEtMR)

The bench crate contains benchmarks for the [memflow](https://github.com/memflow/memflow) library by utiziling the [criterion.rs](https://github.com/bheisler/criterion.rs) framework.

You can run the benchmarks by executing `cargo bench` in the memflow workspace root.

Current benchmarks contain:
- physical reads
- virtual address translations
- virtual reads
