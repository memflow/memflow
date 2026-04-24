#!/usr/bin/env bash

set -euo pipefail

# Check if cbindgen is installed and has the correct version
if ! command -v cbindgen &>/dev/null || [[ "$(cbindgen --version)" != "cbindgen 0.26.0" ]]; then
	echo "Installing cbindgen 0.26.0"
	cargo +stable install --locked --version 0.26.0 cbindgen
fi

if ! command -v cglue-bindgen &>/dev/null || [[ "$(cglue-bindgen --version)" != "cglue-bindgen 0.3.0" ]]; then
	echo "Installing cglue-bindgen 0.3.0"
	cargo +stable install --locked --version 0.3.0 cglue-bindgen
fi
