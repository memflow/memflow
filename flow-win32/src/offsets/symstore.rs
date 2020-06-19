use crate::error::{Error, Result};
use crate::kernel::ntos::Win32GUID;

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use dirs::home_dir;
use log::info;

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
            let cache_dir = cache_path.join(guid.file_name.clone());
            let cache_file = cache_dir.join(guid.guid.clone());

            let buffer = if cache_file.exists() {
                info!(
                    "reading pdb from local cache: {}",
                    cache_file.to_string_lossy()
                );
                let mut file = File::open(cache_file)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                buffer
            } else {
                let buffer = self.download(guid)?;

                if !cache_dir.exists() {
                    info!("creating cache directory {:?}", cache_dir.to_str());
                    fs::create_dir_all(&cache_dir)?;
                }

                info!(
                    "writing pdb to local cache: {}",
                    cache_file.to_string_lossy()
                );
                let mut file = File::create(cache_file)?;
                file.write_all(&buffer[..])?;

                buffer
            };

            Ok(buffer)
        } else {
            self.download(guid)
        }
    }
}
