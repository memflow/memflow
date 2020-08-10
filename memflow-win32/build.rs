use dataview::Pod;
use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

#[path = "src/offsets/offset_data.rs"]
mod offset_data;

use offset_data::Win32OffsetsFile;

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("win32_offsets.bin");
    let mut all_the_files = File::create(&dest_path)?;

    // iterate offsets folder
    let mut vec = Vec::new();
    for f in fs::read_dir("./offsets")? {
        let f = f?;

        if !f.file_type()?.is_file() {
            continue;
        }

        let mut file = File::open(f.path())?;
        let mut tomlstr = String::new();
        file.read_to_string(&mut tomlstr)?;

        let offsets: Win32OffsetsFile = toml::from_str(&tomlstr)?;
        vec.push(offsets);
    }

    all_the_files.write_all(vec.as_slice().as_bytes())?;

    Ok(())
}
