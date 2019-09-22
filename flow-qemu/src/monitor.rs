use std::io;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::path::Path;

use bufstream::BufStream;

pub struct QemuMonitor {
    buffer: BufStream<UnixStream>,
}

impl QemuMonitor {
    pub fn connect<P: AsRef<Path>>(path: P) -> io::Result<QemuMonitor> {
        let stream = UnixStream::connect(path)?;
        let mut mon = QemuMonitor {
            buffer: BufStream::new(stream),
        };

        // skip header
        mon.read_cmd()?; // header

        Ok(mon)
    }

    pub fn read_cmd(&mut self) -> io::Result<String> {
        let mut response = String::new();
        self.buffer.read_line(&mut response)?;
        Ok(response)
    }

    pub fn write_cmd(&mut self, cmd: &str) -> io::Result<()> {
        let b = String::from(cmd) + &"\n";
        self.buffer.write_all(b.as_bytes())?;
        self.buffer.flush()?;
        Ok(())
    }

    pub fn exec_cmd(&mut self, cmd: &str) -> io::Result<String> {
        self.write_cmd(cmd)?;

        let mut response = String::new();
        self.buffer.read_line(&mut response)?; // echo of cmd

        // TODO: sometimes qemu will send multiple lines, read until empty line?
        response.clear();
        self.buffer.read_line(&mut response)?;
        Ok(response)
    }
}
