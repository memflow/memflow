use crate::error::{Error, Result};
use log::warn;

use dirs;
use log::info;
use pdb::PDB;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

use crate::kernel::StartBlock;
use flow_core::address::Address;
use flow_core::mem::VirtualRead;

use clap::ArgMatches;
use duma;
use uuid::{self, Uuid};

use crate::kernel::ntos;

use pelite::{
    self,
    image::GUID,
    pe32,
    pe64::{self, debug::CodeView},
    PeView, Wrap,
};

fn cache_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| Error::new("unable to get home directory"))?;
    let cache = home.join(".memflow").join("cache");
    Ok(cache)
}

fn try_download_pdb(url: &str, filename: &str) -> Result<()> {
    let url = duma::utils::parse_url(&format!("{}/{}", url, filename)).map_err(Error::new)?;

    println!("trying to download pdb from {:?}", url);
    duma::download::http_download(url, &ArgMatches::default(), "0.1")?;

    // try to parse pdb
    let file = File::open(filename).map_err(|e| {
        fs::remove_file(filename).ok();
        e
    })?;
    PDB::open(file).map_err(|e| {
        fs::remove_file(filename).ok();
        e
    })?;

    Ok(())
}

fn download_pdb(pdbname: &str, guid: &str) -> Result<()> {
    info!("downloading pdb for {} with guid/age {}", pdbname, guid);

    let base_url = format!(
        "https://msdl.microsoft.com/download/symbols/{}/{}",
        pdbname, guid
    );
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

fn download_pdb_cache(pdbname: &str, guid: &str) -> Result<PathBuf> {
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

// TODO: this function might be omitted in the future if this is merged to pelite internally
fn generate_guid(signature: GUID, age: u32) -> Result<String> {
    let uuid = Uuid::from_fields(
        signature.Data1,
        signature.Data2,
        signature.Data3,
        &signature.Data4,
    )?;

    Ok(format!(
        "{}{:X}",
        uuid.to_simple().to_string().to_uppercase(),
        age
    ))
}

pub fn fetch_pdb_from_pe<'a, Pe32: pe32::Pe<'a>, Pe64: pe64::Pe<'a>>(
    pe: &Wrap<Pe32, Pe64>,
) -> Result<PathBuf> {
    let debug = match pe.debug() {
        Ok(d) => d,
        Err(_) => return Err(Error::new("unable to read debug_data in pe header")),
    };

    let code_view = debug
        .iter()
        .map(|e| e.entry())
        .filter_map(std::result::Result::ok)
        .filter(|&e| e.as_code_view().is_some())
        .nth(0)
        .ok_or_else(|| Error::new("unable to find codeview debug_data entry"))?
        .as_code_view()
        .unwrap();

    let signature = match code_view {
        CodeView::Cv70 { image, .. } => image.Signature,
        CodeView::Cv20 { image, .. } => {
            return Err(Error::new(
                "invalid code_view entry version 2 found, expected 7",
            ))
        }
    };

    download_pdb_cache(
        code_view.pdb_file_name().to_str()?,
        &generate_guid(signature, code_view.age())?,
    )
}

pub fn fetch_pdb_from_mem<T: VirtualRead>(
    mem: &mut T,
    start_block: &StartBlock,
    kernel_base: Address,
) -> Result<PathBuf> {
    let header_buf = ntos::try_fetch_pe_header(mem, start_block, kernel_base)?;
    let header = PeView::from_bytes(&header_buf)?;

    fetch_pdb_from_pe(&header)
}
