#!/usr/bin/env bash

run_twice() {
	echo $@

	v=$(exec $@)

	if [ $? -ne 0 ]; then
		echo "Error occured! Maybe a fluke. Retrying..."
		v=$(exec $@)
	fi
}

# remove any RUSTC_WRAPPER like sccache which might cause issues with cglue-bindgen
export RUSTC_WRAPPER=""

# update cglue-bindgen
cargo +nightly install cbindgen
cargo +nightly install cglue-bindgen

# generate c and cpp bindings
run_twice rustup run nightly cglue-bindgen +nightly -c cglue.toml -- --config cbindgen.toml --crate memflow-ffi --output memflow.h -l C
run_twice rustup run nightly cglue-bindgen +nightly -c cglue.toml -- --config cbindgen.toml --crate memflow-ffi --output memflow.hpp -l C++
