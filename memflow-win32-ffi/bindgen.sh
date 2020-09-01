#!/bin/bash
cargo build --release --workspace
cbindgen --config cbindgen.toml --crate memflow-win32-ffi --output memflow_win32.h
