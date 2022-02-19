#!/usr/bin/env bash

DIFFC=$(diff memflow.h <(rustup run nightly cglue-bindgen +nightly -c cglue.toml -- --config cbindgen.toml --crate memflow-ffi -l C))
DIFFCPP=$(diff memflow.hpp <(rustup run nightly cglue-bindgen +nightly -c cglue.toml -- --config cbindgen.toml --crate memflow-ffi -l C++))
if [ "$DIFFC" != "" ] || [ "$DIFFCPP" != "" ]
then
	exit 1
fi
