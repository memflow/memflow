#!/bin/bash
cargo build --release --workspace
cbindgen --config cbindgen.toml --crate memflow-ffi --output memflow.h
