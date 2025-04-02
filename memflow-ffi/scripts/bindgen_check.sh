#!/usr/bin/env bash

# remove any RUSTC_WRAPPER like sccache which might cause issues with cglue-bindgen
export RUSTC_WRAPPER=""

# Execute bindgen_prereq.sh from the directory of the current script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
"$SCRIPT_DIR/bindgen_prereq.sh"

DIFFC=$(diff memflow.h <(rustup run nightly cglue-bindgen +nightly -c cglue.toml -- --config cbindgen.toml --crate memflow-ffi -l C))
DIFFCPP=$(diff memflow.hpp <(rustup run nightly cglue-bindgen +nightly -c cglue.toml -- --config cbindgen.toml --crate memflow-ffi -l C++))
if [ "$DIFFC" != "" ] || [ "$DIFFCPP" != "" ]
then
	exit 1
fi
