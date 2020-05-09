use crate::error::{Error, Result};
use log::warn;

use log::info;
use pdb::PDB;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

use clap::ArgMatches;

use crate::kernel::ntos::Win32GUID;

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

fn download_pdb(guid: &Win32GUID) -> Result<()> {
    info!(
        "downloading pdb for {} with guid/age {}",
        guid.file_name, guid.guid
    );

    let base_url = format!(
        "https://msdl.microsoft.com/download/symbols/{}/{}",
        guid.file_name, guid.guid
    );
    match try_download_pdb(&base_url, &guid.file_name) {
        Ok(_) => return Ok(()),
        Err(e) => warn!("unable to download pdb: {:?}", e),
    }

    let mut pdbname = guid.file_name.clone();
    pdbname.truncate(pdbname.len() - 1);
    pdbname.push('_');
    match try_download_pdb(&base_url, &pdbname) {
        Ok(_) => return Ok(()),
        Err(e) => warn!("could not fetch {}: {:?}", pdbname, e),
    }

    match try_download_pdb(&base_url, &"file.ptr".to_string()) {
        Ok(_) => return Ok(()),
        Err(e) => warn!("could not fetch file.ptr: {:?}", e),
    }

    Err(Error::new("unable to download a valid pdb"))
}

pub fn try_get_pdb(guid: &Win32GUID) -> Result<PathBuf> {
    info!(
        "fetching pdb for {} with guid/age {}",
        guid.file_name, guid.guid
    );

    let cache_dir = cache_dir()?.join(guid.file_name.clone());
    let cache_file = cache_dir.join(guid.guid.clone());
    if !cache_file.exists() {
        info!(
            "{} does not exist in cache, downloading from microsoft servers",
            guid.file_name
        );
        download_pdb(guid)?;

        // create cache dir if necessary and move the downloaded file
        if !cache_dir.exists() {
            info!("creating cache directory {:?}", cache_dir.to_str());
            fs::create_dir_all(&cache_dir)?;
        }

        // TODO: this is super dirty as we cannot decide where to put the resulting file
        // a fork of duma would be necessary to add decent library functionality!
        info!(
            "moving {} to cache {}",
            guid.file_name,
            cache_file.to_str().unwrap_or_default()
        );
        if fs::rename(guid.file_name.clone(), &cache_file).is_err() {
            fs::copy(guid.file_name.clone(), &cache_file)?;
            fs::remove_file(guid.file_name.clone())?;
        }
    }

    Ok(cache_file)
}
