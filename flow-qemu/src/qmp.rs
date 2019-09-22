use std::io;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::path::Path;

use bufstream::BufStream;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Cmd {
    execute: String,
}

impl Cmd {
    pub fn new(e: &str) -> Cmd {
        Cmd {
            execute: String::from(e),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MonitorArgs {
    #[serde(rename(serialize = "command-line", deserialize = "command-line"))]
    command_line: String,
}

#[derive(Serialize, Deserialize)]
pub struct MonitorCmd {
    execute: String,
    arguments: MonitorArgs,
}

impl MonitorCmd {
    pub fn new(e: &str) -> MonitorCmd {
        MonitorCmd {
            execute: String::from("human-monitor-command"),
            arguments: MonitorArgs {
                command_line: String::from(e),
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MonitorCmdResponse {
    #[serde(rename(serialize = "return", deserialize = "return"))]
    response: String,
}

pub struct Qmp {
    buffer: BufStream<UnixStream>,
}

impl Qmp {
    pub fn connect<P: AsRef<Path>>(path: P) -> io::Result<Qmp> {
        let stream = UnixStream::connect(path)?;
        let mut qmp = Qmp {
            buffer: BufStream::new(stream),
        };

        // negotiate
        let _header = qmp.read_cmd(); // header
        qmp.write_cmd("qmp_capabilities")?;
        qmp.read_cmd()?; // capabilities

        Ok(qmp)
    }

    pub fn read_cmd(&mut self) -> io::Result<String> {
        let mut response = String::new();
        self.buffer.read_line(&mut response)?;
        Ok(response)
    }

    pub fn write_cmd(&mut self, cmd: &str) -> io::Result<()> {
        let c = Cmd::new(cmd);
        let j = serde_json::to_string(&c)?;
        self.buffer.write_all(j.as_bytes())?;
        self.buffer.flush()?;
        Ok(())
    }

    fn read_monitor_cmd(&mut self) -> io::Result<String> {
        let mut response = String::new();
        self.buffer.read_line(&mut response)?;
        let r: MonitorCmdResponse = serde_json::from_str(&response)?;
        Ok(r.response)
    }

    fn write_monitor_cmd(&mut self, cmd: &str) -> io::Result<()> {
        let c = MonitorCmd::new(cmd);
        let j = serde_json::to_string(&c)?;
        self.buffer.write_all(j.as_bytes())?;
        self.buffer.flush()?;
        Ok(())
    }

    pub fn exec_monitor_cmd(&mut self, cmd: &str) -> io::Result<String> {
        self.write_monitor_cmd(cmd)?;
        self.read_monitor_cmd()
    }
}
