#!/bin/bash

run_twice() {
	echo $@

	v=$(exec $@)

	if [ $? -ne 0 ]; then
		echo "Error occured! Maybe a fluke. Retrying..."
		v=$(exec $@)
	fi
}

run_twice rustup run nightly cglue-bindgen +nightly -c cglue.toml -- --config cbindgen.toml --crate memflow-ffi --output memflow.h -l C
run_twice rustup run nightly cglue-bindgen +nightly -c cglue.toml -- --config cbindgen.toml --crate memflow-ffi --output memflow.hpp -l C++
