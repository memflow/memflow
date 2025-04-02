#!/usr/bin/env bash

# Check if cbindgen is installed and has the correct version
if ! command -v cbindgen &> /dev/null || [[ "$(cbindgen --version)" != "cbindgen 0.26.0" ]]; then
  echo "Installing cbindgen 0.26.0"
  cargo install --version=0.26.0 cbindgen
fi

if ! command -v cglue-bindgen &> /dev/null; then
  cargo install --version =0.3.0 cglue-bindgen
fi
