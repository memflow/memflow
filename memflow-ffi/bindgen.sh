#!/bin/bash
rustup run nightly cglue-bindgen -- --config cbindgen.toml --crate memflow-ffi --output memflow.h
