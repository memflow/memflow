use std::prelude::v1::*;

use crate::error::{Error, Result};
use crate::offsets::Win32GUID;

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use dirs::home_dir;
use log::info;

#[cfg(feature = "download_progress")]
use {
    pbr::ProgressBar,
    progress_streams::ProgressReader,
    std::sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    std::sync::Arc,
};

#[cfg(feature = "download_progress")]
fn read_to_end<T: Read>(reader: &mut T, len: usize) -> Result<Vec<u8>> {
    let mut buffer = vec![];

    let total = Arc::new(AtomicUsize::new(0));
    let mut reader = ProgressReader::new(reader, |progress: usize| {
        total.fetch_add(progress, Ordering::SeqCst);
    });
    let mut pb = ProgressBar::new(len as u64);

    let finished = Arc::new(AtomicBool::new(false));
    let thread = {
        let finished_thread = finished.clone();
        let total_thread = total.clone();

        std::thread::spawn(move || {
            while !finished_thread.load(Ordering::Relaxed) {
                pb.set(total_thread.load(Ordering::SeqCst) as u64);
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            pb.finish();
        })
    };

    reader
        .read_to_end(&mut buffer)
        .map_err(|_| Error::SymbolStore("unable to read from http request"))?;
    finished.store(true, Ordering::Relaxed);
    thread.join().unwrap();

    Ok(buffer)
}

#[cfg(not(feature = "download_progress"))]
fn read_to_end<T: Read>(reader: &mut T, _len: usize) -> Result<Vec<u8>> {
    let mut buffer = vec![];
    reader.read_to_end(&mut buffer)?;
    Ok(buffer)
}

#[derive(Debug, Clone)]
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&self, guid: &Win32GUID) -> Result<Vec<u8>> {
        if let Some(cache_path) = &self.cache_path {
            let cache_dir = cache_path.join(guid.file_name.clone());
            let cache_file = cache_dir.join(guid.guid.clone());

            let buffer = if cache_file.exists() {
                info!(
                    "reading pdb from local cache: {}",
                    cache_file.to_string_lossy()
                );
                let mut file = File::open(cache_file)
                    .map_err(|_| Error::SymbolStore("unable to open pdb in local cache"))?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)
                    .map_err(|_| Error::SymbolStore("unable to read pdb from local cache"))?;
                buffer
            } else {
                let buffer = self.download(guid)?;

                if !cache_dir.exists() {
                    info!("creating cache directory {:?}", cache_dir.to_str());
                    fs::create_dir_all(&cache_dir).map_err(|_| {
                        Error::SymbolStore("unable to create folder in local pdb cache")
                    })?;
                }

                info!(
                    "writing pdb to local cache: {}",
                    cache_file.to_string_lossy()
                );
                let mut file = File::create(cache_file)
                    .map_err(|_| Error::SymbolStore("unable to create file in local pdb cache"))?;
                file.write_all(&buffer[..])
                    .map_err(|_| Error::SymbolStore("unable to write pdb to local cache"))?;

                buffer
            };

            Ok(buffer)
        } else {
            self.download(guid)
        }
    }

    fn download(&self, guid: &Win32GUID) -> Result<Vec<u8>> {
        let pdb_url = format!("{}/{}/{}", self.base_url, guid.file_name, guid.guid);

        self.download_file(&format!("{}/{}", pdb_url, guid.file_name))
            .or_else(|_| self.download_file(&format!("{}/{}", pdb_url, "file.ptr")))
    }

    fn download_file(&self, url: &str) -> Result<Vec<u8>> {
        info!("downloading pdb from {}", url);
        let resp = ureq::get(url).call();
        if !resp.ok() {
            return Err(Error::SymbolStore("unable to download pdb"));
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

    // symbol store configurations
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
}
