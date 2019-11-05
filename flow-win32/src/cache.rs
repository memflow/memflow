use crate::error::{Error, Result};
use log::warn;

use dirs;
use log::info;
use pdb::PDB;
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

use crate::kernel::StartBlock;
use address::{Address, Length};
use mem::VirtualRead;

use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use clap::ArgMatches;
use duma;
use goblin::pe::{debug::CodeviewPDB70DebugInfo, options::ParseOptions, PE};
use uuid::{self, Uuid};

fn cache_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| Error::new("unable to get home directory"))?;
    let cache = home.join(".memflow").join("cache");
    Ok(cache)
}

fn try_download_pdb(url: &String, filename: &String) -> Result<()> {
    let url = duma::utils::parse_url(&format!("{}/{}", url, filename))
        .map_err(|e| Error::new(e))?;

    println!("trying to download pdb from {:?}", url);
    duma::download::http_download(url, &ArgMatches::default(), "0.1")?;

    // try to parse pdb
    let file = File::open(filename).map_err(|e| { fs::remove_file(filename).ok(); e })?;
    PDB::open(file).map_err(|e| { fs::remove_file(filename).ok(); e })?;

    Ok(())
}

fn download_pdb(pdbname: &String, guid: &String) -> Result<()> {
    info!("downloading pdb for {} with guid/age {}", pdbname, guid);

    let base_url = format!("https://msdl.microsoft.com/download/symbols/{}/{}", pdbname, guid);
    match try_download_pdb(&base_url, pdbname) {
        Ok(_) => return Ok(()),
        Err(e) => warn!("unable to download pdb: {:?}", e),
    }

    let mut pdbname2 = String::from(pdbname);
    pdbname2.truncate(pdbname2.len() - 1);
    pdbname2.push('_');
    match try_download_pdb(&base_url, &pdbname2) {
        Ok(_) => return Ok(()),
        Err(e) => warn!("could not fetch {}: {:?}", pdbname2, e),
    }

    match try_download_pdb(&base_url, &"file.ptr".to_string()) {
        Ok(_) => return Ok(()),
        Err(e) => warn!("could not fetch file.ptr: {:?}", e),
    }

    Err(Error::new("unable to download a valid pdb"))
}

fn download_pdb_cache(pdbname: &String, guid: &String) -> Result<PathBuf> {
    info!("fetching pdb for {} with guid/age {}", pdbname, guid);

    let cache_dir = cache_dir()?.join(pdbname);
    let cache_file = cache_dir.join(guid);
    if !cache_file.exists() {
        info!(
            "{} does not exist in cache, downloading from microsoft servers",
            pdbname
        );
        download_pdb(pdbname, guid)?;

        // create cache dir if necessary and move the downloaded file
        if !cache_dir.exists() {
            info!("creating cache directory {:?}", cache_dir.to_str());
            fs::create_dir_all(&cache_dir)?;
        }

        // TODO: this is super dirty as we cannot decide where to put the resulting file
        // a fork of duma would be necessary to add decent library functionality!
        info!(
            "moving {} to cache {}",
            pdbname,
            cache_file.to_str().unwrap_or_default()
        );
        fs::rename(pdbname, &cache_file)?;
    }

    Ok(cache_file)
}

// TODO: this function might be omitted in the future if this is merged to goblin internally
fn generate_guid(codeview: &CodeviewPDB70DebugInfo) -> std::result::Result<String, uuid::Error> {
    let mut rdr = Cursor::new(codeview.signature);
    let uuid = Uuid::from_fields(
        rdr.read_u32::<LittleEndian>().unwrap_or_default(), // TODO: fix error handling
        rdr.read_u16::<LittleEndian>().unwrap_or_default(),
        rdr.read_u16::<LittleEndian>().unwrap_or_default(),
        &codeview.signature[8..],
    )?;

    Ok(format!(
        "{}{:X}",
        uuid.to_simple().to_string().to_uppercase(),
        codeview.age
    ))
}

pub fn fetch_pdb_from_pe(pe: &PE) -> Result<PathBuf> {
    let debug = match pe.debug_data {
        Some(d) => d,
        None => return Err(Error::new("unable to read debug_data in pe header")),
    };

    let codeview = match debug.codeview_pdb70_debug_info {
        Some(c) => c,
        None => return Err(Error::new("unable to read codeview in debug header")),
    };

    // TODO: map error of generate_guid properly
    download_pdb_cache(
        &String::from_utf8(codeview.filename.to_vec())
            .unwrap_or_default()
            .trim_matches(char::from(0)).to_string(),
        &generate_guid(&codeview).unwrap_or_default(),
    )
}

pub fn fetch_pdb_from_mem<T: VirtualRead>(
    mem: &mut T,
    start_block: &StartBlock,
    kernel_base: Address,
) -> Result<PathBuf> {
    let ntos_buf = mem
        .virt_read(
            start_block.arch,
            start_block.dtb,
            kernel_base,
            Length::from_mb(32),
        )
        .unwrap();

    let mut pe_opts = ParseOptions::default();
    pe_opts.resolve_rva = false;

    let pe = PE::parse_with_opts(&ntos_buf, &pe_opts)?;

    fetch_pdb_from_pe(&pe)
}
