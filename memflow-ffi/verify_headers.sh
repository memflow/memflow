#!/usr/bin/env bash

# update cglue-bindgen (see https://github.com/h33p/cglue/pull/15)
cargo +nightly install cbindgen
cargo +nightly install https://github.com/ko1N/cglue/cglue-bindgen

DIFFC=$(diff memflow.h <(rustup run nightly cglue-bindgen +nightly -c cglue.toml -- --config cbindgen.toml --crate memflow-ffi -l C))
DIFFCPP=$(diff memflow.hpp <(rustup run nightly cglue-bindgen +nightly -c cglue.toml -- --config cbindgen.toml --crate memflow-ffi -l C++))
if [ "$DIFFC" != "" ] || [ "$DIFFCPP" != "" ]
then
	exit 1
fi
