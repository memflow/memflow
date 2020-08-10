use dataview::Pod;
use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

#[path = "src/offsets/offset_table.rs"]
mod offset_table;

use offset_table::Win32OffsetFile;

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("win32_offsets.bin");
    let mut all_the_files = File::create(&dest_path)?;

    // iterate offsets folder
    for f in fs::read_dir("./offsets")? {
        let f = f?;

        if !f.file_type()?.is_file() {
            continue;
        }

        let mut file = File::open(f.path())?;
        let mut tomlstr = String::new();
        file.read_to_string(&mut tomlstr)?;

        let offsets: Win32OffsetFile = toml::from_str(&tomlstr)?;
        all_the_files.write_all(offsets.as_bytes())?;
    }

    Ok(())
}
