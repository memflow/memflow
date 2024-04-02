# Test binaries

These binaries are just cross-compiled versions of the memflow-coredump connector.\
You can find the source code [here](https://github.com/memflow/memflow-coredump).

To reproduce the binaries run the following:
```
$ cross build --target i686-pc-windows-gnu --release --all-features
```

More information can be found in the [cross repo](https://github.com/cross-rs/cross).
