extern crate cbindgen;

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    const HEADER_NAME: &str = "memflow.h";

    let config = cbindgen::Config::from_root_or_default(&crate_dir);

    // https://github.com/alexcrichton/proc-macro2/issues/218
    proc_macro2::fallback::force();

    cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(config)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join(HEADER_NAME));

    fs::copy(
        out_path.join(HEADER_NAME),
        PathBuf::from(crate_dir).join(HEADER_NAME),
    )
    .unwrap();
}
