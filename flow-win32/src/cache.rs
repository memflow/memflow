use dirs;
use log::{info, trace, warn};
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;

use clap::ArgMatches;
use duma;
use uuid::{BytesError, Uuid};

fn cache_dir() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| Error::new(ErrorKind::Other, "unable to get home directory"))?;
    let cache = home.join(".memflow").join("cache");
    Ok(cache)
}

fn download_pdb(pdbname: &str, guid: &String) -> Result<()> {
    info!("downloading pdb for {} with guid/age {}", pdbname, guid);

    let url = match duma::utils::parse_url(&format!(
        "https://msdl.microsoft.com/download/symbols/{}/{}/{}",
        pdbname, guid, pdbname
    )) {
        Ok(u) => u,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("unable to format download url: {}", e),
            ))
        }
    };

    match duma::download::http_download(url, &ArgMatches::default(), "0.1") {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(
            ErrorKind::Other,
            format!("unable to download pdb file: {}", e),
        )),
    }
}

pub fn fetch_pdb(pdbname: &str, guid: &String) -> Result<PathBuf> {
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
