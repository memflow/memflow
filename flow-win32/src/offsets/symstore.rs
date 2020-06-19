use crate::error::{Error, Result};
use log::{info, warn};

use pdb::PDB;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

use dirs::home_dir;

use crate::kernel::ntos::Win32GUID;

/*
fn cache_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| Error::new("unable to get home directory"))?;
    let cache = home.join(".memflow").join("cache");
    Ok(cache)
}

fn try_download_pdb(url: &str, filename: &str) -> Result<()> {
    let url = duma::utils::parse_url(&format!("{}/{}", url, filename)).map_err(Error::new)?;

    info!("trying to download pdb from {:?}", url);
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
*/

use std::io::Read;
use ureq;

#[cfg(feature = "download_progress")]
use cursive::utils::{Counter, ProgressReader};
#[cfg(feature = "download_progress")]
use indicatif::{ProgressBar, ProgressStyle};
#[cfg(feature = "download_progress")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "download_progress")]
use std::sync::Arc;

#[cfg(feature = "download_progress")]
fn read_to_end<T: Read>(reader: &mut T, len: usize) -> Result<Vec<u8>> {
    let mut buffer = vec![];

    let counter = Counter::new(0);
    let mut reader = ProgressReader::new(counter.clone(), reader);

    let pb = ProgressBar::new(len as u64);
    pb.set_style(ProgressStyle::default_bar()
                 .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                 .progress_chars("#>-"));

    let finished = Arc::new(AtomicBool::new(false));
    let finished_thread = finished.clone();
    let thread = std::thread::spawn(move || {
        while !finished_thread.load(Ordering::Relaxed) {
            let progress = counter.get();
            pb.set_position(progress as u64)
        }
        pb.finish();
    });

    reader.read_to_end(&mut buffer)?;
    finished.store(true, Ordering::Relaxed);
    thread.join().unwrap();

    Ok(buffer)
}

#[cfg(not(feature = "download_progress"))]
fn read_to_end<T: Read>(reader: &mut T, len: usize) -> Result<Vec<u8>> {
    let mut buffer = vec![];
    reader.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub struct SymbolStore {
    base_url: String,
    cache_path: Option<PathBuf>,
}

impl Default for SymbolStore {
    fn default() -> Self {
        let home_dir = home_dir().expect("unable to get home directory");
        Self {
            base_url: "https://msdl.microsoft.com/download/symbols".to_string(),
            cache_path: Some(home_dir.join(".memflow").join("cache")),
        }
    }
}

impl SymbolStore {
    pub fn base_url(mut self, base_url: &str) -> Self {
        self.base_url = base_url.to_string();
        self
    }

    pub fn no_cache(mut self) -> Self {
        self.cache_path = None;
        self
    }

    pub fn cache_path<P: AsRef<Path>>(mut self, cache_path: P) -> Self {
        self.cache_path = Some(cache_path.as_ref().to_path_buf());
        self
    }

    pub fn download(&self, guid: &Win32GUID) -> Result<Vec<u8>> {
        let pdb_url = format!("{}/{}/{}", self.base_url, guid.file_name, guid.guid);

        self.download_file(&format!("{}/{}", pdb_url, guid.file_name))
            .or_else(|_| self.download_file(&format!("{}/{}", pdb_url, "file.ptr")))
    }

    fn download_file(&self, url: &str) -> Result<Vec<u8>> {
        println!("downloading pdb from {}", url);
        let resp = ureq::get(url).call();
        if !resp.ok() {
            return Err(Error::new(format!(
                "unable to download pdb: {}",
                resp.status_line()
            )));
        }

        assert!(resp.has("Content-Length"));
        let len = resp
            .header("Content-Length")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap();

        let mut reader = resp.into_reader();
        let buffer = read_to_end(&mut reader, len)?;

        assert_eq!(buffer.len(), len);
        Ok(buffer)
    }

    pub fn load(&self, guid: &Win32GUID) -> Result<Vec<u8>> {
        if let Some(cache_path) = &self.cache_path {
            // TODO: open file from cache :)
            Err(Error::new("cache path not implemented yet"))
        } else {
            // TODO: put file into cache :)
            self.download(guid)
        }
    }
}
