use clap::{App, Arg};
//use std::time::Instant;

use flow_qemu::Qmp;

//////////////////////////////
// TEST IMPL
//////////////////////////////
/*
use std::{fmt, num::ParseIntError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeHexError {
    OddLength,
    ParseInt(ParseIntError),
}

impl From<ParseIntError> for DecodeHexError {
    fn from(e: ParseIntError) -> Self {
        DecodeHexError::ParseInt(e)
    }
}

impl fmt::Display for DecodeHexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodeHexError::OddLength => "input string has an odd number of bytes".fmt(f),
            DecodeHexError::ParseInt(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for DecodeHexError {}
*/

/**
 * 0000000000001000: 0xe8 0x90 0xbf 0x7f 0x00 0x00 0x00 0x00
 */
/*
pub fn decode_hex(s: &str) -> Result<Vec<u8>, DecodeHexError> {
    if s.len() % 2 != 0 {
        Err(DecodeHexError::OddLength)
    } else {
        s.lines().map(|l| {
            l.chars()
                .step_by(5)
                .map(|i| u8::from_str_radix(&s[i + 2..i + 4], 16).map_err(|e| e.into()))
                .collect()
        })
    }
}
*/

/*pub fn encode_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| unsafe {
            let i = 2 * b as usize;
            HEX_BYTES.get_unchecked(i..i + 2)
        })
        .collect()
}*/

fn main() {
    let argv = App::new("examples/bridge_read")
        .version("0.1")
        .arg(
            Arg::with_name("socket")
                .short("s")
                .long("socket")
                .value_name("FILE")
                .help("bridge unix socket file")
                .takes_value(true),
        )
        .get_matches();

    let socket = argv.value_of("socket").unwrap_or("/tmp/qemu-qmp");
    let mut qmp = match Qmp::connect(socket) {
        Ok(s) => s,
        Err(e) => {
            println!("couldn't connect to qmp: {:?}", e);
            return;
        }
    };

    /*
        qmp.write_monitor_cmd("info registers");
        let r = qmp.exec_monitor_cmd("xp /8b 0x1000").unwrap();
        println!("{}", r);
    */
    let r = qmp.exec_monitor_cmd("xp /256xb 0x1000").unwrap();
    println!("{}", r);

    /*
        let start = Instant::now();
        let mut counter = 0;
        loop {
            qmp.exec_monitor_cmd("xp /0x1000b 0x1000").unwrap();

            counter += 1;
            if (counter % 1000) == 0 {
                let elapsed = start.elapsed().as_millis() as f64;
                if elapsed > 0.0 {
                    println!("{} reads/sec", (counter as f64) / elapsed * 1000.0);
                    println!("{} ms/read", elapsed / (counter as f64));
                }
            }
        }
    */
}
