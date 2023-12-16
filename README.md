# memflow
[![Crates.io](https://img.shields.io/crates/v/memflow.svg)](https://crates.io/crates/memflow)
![build and test](https://github.com/memflow/memflow/workflows/Build%20and%20test/badge.svg?branch=dev)
[![codecov](https://codecov.io/gh/memflow/memflow/branch/master/graph/badge.svg?token=XT7R158N6W)](https://codecov.io/gh/memflow/memflow)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Discord](https://img.shields.io/discord/738739624976973835?color=%20%237289da&label=Discord)](https://discord.gg/afsEtMR)

## machine introspection made easy

memflow is a library that enables introspection of various machines (hardware, virtual machines, memory dumps) in a generic fashion. There are 2 primary types of objects in memflow - _Connectors_ and _OS layers_. Connector provides raw access to physical memory of a machine. Meanwhile, OS layer builds a higher level abstraction over running operating system, providing access to running processes, input events, etc. These objects are incredibly flexible as they can be chained together to gain access to a process running multiple levels of virtualization deep (see figure below).

```
+-----------+        +-----------+
| native OS |        | leechcore |
+-+---------+        +-+---------+
  |                    |
  |  +-----------+     |  +----------+
  +->|  QEMU VM  |     +->| Win32 OS |
     +-+---------+        +-+--------+
       |                    |
       |  +----------+      |  +-----------+
       +->| Win32 OS |      +->| lsass.exe |
          +-+--------+         +-----------+
            |
            |  +-----------+
            +->|  Hyper-V  |
               +-+---------+
                 |
                 |  +----------+
                 +->| Linux OS |
                    +-+--------+
                      |
                      |  +-----------+
                      +->| SSHD Proc |
                         +-----------+

(Example chains of access. For illustrative purposes only - Hyper-V Connector and Linux OS are not yet available)
```

As a library user, you do not have to worry about delicacies of chaining - everything is provided, batteries included. See one of our [examples](memflow/examples/process_list.rs) on how simple it is to build a chain (excluding parsing). All Connectors and OS layers are dynamically loadable with common interface binding them.

All of this flexibility is provided with very robust and efficient backend - memory interface is batchable and divisible, which gets taken advantage of by our throughput optimized virtual address translation pipeline that is able to walk the entire process virtual address space in under a second. Connectors and OS layers can be composed with the vast library of generic caching mechanisms, utility functions and data structures.

The memflow ecosystem is not bound to just Rust - Connector and OS layer functions are linked together using C ABI, thus users can write code that interfaces with them in other languages, such as C, C++, Zig, etc. In addition, these plugins can too be implemented in foreign languages - everything is open.

Overall, memflow is the most robust, efficient and flexible solution out there for machine introspection.

## Getting started

Make sure that your rustc version is at least `1.70.0` or newer.

memflow uses a plugin based approach and is capable of loading different physical memory backends (so-called [`connectors`](#connectors)) at runtime. On top of the physical memory backends memflow is also capable of loading plugins for interfacing with a specific target OS at runtime.

To get started, you want to at least install one connector. For that, use the [memflowup](https://github.com/memflow/memflowup) utility (use dev channel).

### Manual installation

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

For more information about how to get started with memflow please head over to the YouTube series produced by [h33p](https://github.com/h33p/):

- [memflow basics](https://www.youtube.com/playlist?list=PLrC4R7zDrxB3RSJQk9ahmXNCw8m3pdP6z)
- [memflow applied](https://www.youtube.com/watch?v=xJXkRMy71dc&list=PLrC4R7zDrxB17iWCy9eEdCaluCR3Bkn8q)

## Running Examples

You can either run one of the examples with `cargo run --release --example`. Pass nothing to get a list of examples.

Some connectors like `qemu` will require elevated privileges. Refer to the readme of the connector for additional information on their required access rights.

To simplify running examples, tests, and benchmarks through different connectors, we added a simple cargo runner script for Linux to this repository.
Simply set any of the following environment variables when running the `cargo` command to elevate privileges:

- `RUST_SUDO` will start the resulting binary via sudo.
- `RUST_SETPTRACE` will enable PTRACE permissions on the resulting binary before executing it.

Alternatively, you can run the benchmarks via `cargo bench` (can pass regex filters). Win32 benchmarks currently work only on Linux.

All examples support the memflow connector `plugins` inventory system.
You will have to install at least one `connector` to use the examples. Refer to the [getting started](#getting-started) section for more details.

Run memflow/read\_keys example with a qemu connector:

`RUST_SETPTRACE=1 cargo run --example read_keys -- -vv -c qemu -a [vmname] -o win32`

Run memflow/read\_bench example with a coredump connector:

`cargo run --example read_bench --release -- -vv -c coredump -a coredump_win10_64bit.raw -o win32`

Note: In the examples above the `qemu` connector requires `'CAP_SYS_PTRACE=ep'` permissions. The runner script in this repository will set the appropriate flags when the `RUST_SETPTRACE` environment variable is passed to it.

## Documentation

Extensive code documentation can be found at [docs.rs](https://docs.rs/memflow/0.2.0-beta/)
(it currently is relatively out of date).

An additional getting started guide as well as a higher level
explanation of the inner workings of memflow can be found at [memflow.github.io](https://memflow.github.io).

If you decide to build the latest documentation you can do it by issuing:

`cargo doc --workspace --no-deps --open`

## Compilation support

memflow currently requires at least rustc version `1.70.0` or newer.

| target        | build              | tests              | benches            | compiles on stable |
|---------------|--------------------|--------------------|--------------------|--------------------|
| linux x86_64  | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| mac x86_64    | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| win x86_64    | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| linux aarch64 | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| linux i686    | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| linux armv7   | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| no-std        | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :x:                |

## Target support

By default, memflow supports analyzing 64-bit machines on any machine - be it 32 or 64 bit. Using memflow without `default_features` can disable 64-bit support on 32-bit machines for an efficiency gain, while enabling `128_bit_mem` feature can be done for theoretical future 128-bit machine analysis. Note that all connectors and OS layers must be compiled with the same memory features enabled, and memflowup currently only compiles the default set of features.

memflow-win32 is tested on the latest Windows 11 and Windows 10 versions all the way down to Windows NT 4.0. If you found a version that does not work please submit an issue with the major/minor version as well as the build number.

## Connectors

All examples provided in this repository are using the `plugins` inventory to
dynamically load a connector at runtime. When using the library programmatically it is possible to just statically link a connector into the code.

Some connectors also require different permissions. Please refer to the individual connector repositories for more information.

These are the currently officially existing connectors:
- [qemu](https://github.com/memflow/memflow-qemu-procfs)
- [kvm](https://github.com/memflow/memflow-kvm)
- [pcileech](https://github.com/memflow/memflow-pcileech)
- [coredump](https://github.com/memflow/memflow-coredump)

In case you write your own connector please hit us up with a pull request so we can maintain a list of third-party connectors as well.

## Build on memflow

Officialy supported projects:
- [memflow-py](https://github.com/memflow/memflow-py) Python Wrapper for memflow (thanks to [emesare](https://github.com/emesare))


Additional projects from the community:
- [.NET wrapper for memflow-ffi](https://github.com/uberhalit/memflow.NET) by [uberhalit](https://github.com/uberhalit)
- [rhai integration](https://github.com/dankope/rhai-memflow) by [emesare](https://github.com/emesare)

## Acknowledgements
- [CasualX](https://github.com/casualx/) for his wonderful pelite crate
- [ufrisk](https://github.com/ufrisk/) for his prior work on the subject and many inspirations

## Contributing

Please check [CONTRIBUTE.md](CONTRIBUTE.md)
