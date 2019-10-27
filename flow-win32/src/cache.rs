use dirs;
use log::{info, trace, warn};
use std::fs;
use std::io::{Cursor, Error, ErrorKind, Result};
use std::path::PathBuf;

use crate::dtb::DTB;
use mem::{PhysicalRead, VirtualRead};
use address::{Address, Length};

use byteorder::ReadBytesExt;
use byteorder::{ByteOrder, LittleEndian};
use clap::ArgMatches;
use duma;
use goblin::pe::{debug::CodeviewPDB70DebugInfo, options::ParseOptions, PE};
use uuid::{BytesError, Uuid};

fn cache_dir() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| Error::new(ErrorKind::Other, "unable to get home directory"))?;
    let cache = home.join(".memflow").join("cache");
    Ok(cache)
}

fn download_pdb(pdbname: &str, guid: &String) -> Result<()> {
    info!("downloading pdb for {} with guid/age {}", pdbname, guid);

    /*
    TODO: in reality this is 3 urls  which we check in order:
    ntkrnlmp.pdb/ID/ntkrnlmp.pdb
    ntkrnlmp.pdb/ID/ntkrnlmp.pd_
    ntkrnlmp.pdb/ID/file.ptr
    */
    let url = match duma::utils::parse_url(&format!(
        "https://msdl.microsoft.com/download/symbols/{}/{}/{}",
        pdbname, guid, pdbname
    )) {
        Ok(u) => u,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("unable to format download url: {}", e),
            ));
        }
    };

    println!("downloading pdb from {:?}", url);
    match duma::download::http_download(url, &ArgMatches::default(), "0.1") {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(
            ErrorKind::Other,
            format!("unable to download pdb file: {}", e),
        )),
    }

    // TODO: check if file is empty (404, other error)
}

fn download_pdb_cache(pdbname: &str, guid: &String) -> Result<PathBuf> {
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
fn generate_guid(codeview: &CodeviewPDB70DebugInfo) -> std::result::Result<String, BytesError> {
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
        None => {
            return Err(Error::new(
                ErrorKind::Other,
                "unable to read debug_data in pe header",
            ))
        }
    };

    let codeview = match debug.codeview_pdb70_debug_info {
        Some(c) => c,
        None => {
            return Err(Error::new(
                ErrorKind::Other,
                "unable to read codeview in debug header",
            ))
        }
    };

    // TODO: map error of generate_guid properly
    download_pdb_cache(
        &String::from_utf8(codeview.filename.to_vec())
            .unwrap_or_default()
            .trim_matches(char::from(0)),
        &generate_guid(&codeview).unwrap_or_default(),
    )
}

pub fn fetch_pdb_from_mem<T: VirtualRead>(mem: &mut T, dtb: &DTB, kernel_base: Address) -> Result<PathBuf> {
    let ntos_buf = mem
        .virt_read(
            dtb.arch,
            dtb.dtb,
            kernel_base,
            Length::from_mb(32),
        )
        .unwrap();

    let mut pe_opts = ParseOptions::default();
    pe_opts.resolve_rva = false;

    let pe = match PE::parse_with_opts(&ntos_buf, &pe_opts) {
        Ok(pe) => pe,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("unable to parse pe header: {}", e),
            ))
        }
    };

    fetch_pdb_from_pe(&pe)
}
