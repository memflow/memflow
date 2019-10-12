use clap::{App, Arg};
use pretty_env_logger;
use std::io::{Error, ErrorKind, Result};

use address::{Address, Length};
use flow_qemu::BridgeConnector;
use flow_win32;
use flow_win32::cache;
use flow_win32::win::Windows;
use goblin::pe::{options::ParseOptions, PE};
use mem::VirtualRead;

use byteorder::ReadBytesExt;
use byteorder::{ByteOrder, LittleEndian};
use std::io::Cursor;
use uuid::{BytesError, Uuid};

// test
use clap::ArgMatches;
use duma::{download, utils};

fn microsoft_download(module: &str, uuid: String) -> Result<()> {
    println!("downloading {} {}", module, uuid);

    let url = utils::parse_url(&format!(
        "https://msdl.microsoft.com/download/symbols/{}/{}/{}",
        module, uuid, module
    ))
    .unwrap();
    download::http_download(url, &ArgMatches::default(), "0.1").unwrap();

    Ok(())
}

fn sig_to_uuid(sig: &[u8; 16], age: u32) -> std::result::Result<String, BytesError> {
    let mut rdr = Cursor::new(sig);
    let uuid = Uuid::from_fields(
        rdr.read_u32::<LittleEndian>().unwrap_or_default(), // TODO: fix error handling
        rdr.read_u16::<LittleEndian>().unwrap_or_default(),
        rdr.read_u16::<LittleEndian>().unwrap_or_default(),
        &sig[8..],
    )?;

    Ok(format!(
        "{}{:X}",
        uuid.to_simple().to_string().to_uppercase(),
        age
    ))
}

fn microsoft_download_ntos<T: VirtualRead>(mem: &mut T, win: &Windows) -> Result<()> {
    let ntos_buf = mem
        .virt_read(
            win.dtb.arch,
            win.dtb.dtb,
            win.kernel_base,
            Length::from_mb(32),
        )
        .unwrap();

    let mut pe_opts = ParseOptions::default();
    pe_opts.resolve_rva = false;

    let pe = match PE::parse_with_opts(&ntos_buf, &pe_opts) {
        Ok(pe) => {
            //println!("find_x64_with_va: found pe header:\n{:?}", pe);
            pe
        }
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, "unable to parse pe header"));
        }
    };

    if let Some(debug) = pe.debug_data {
        //println!("debug_data: {:?}", debug);
        if let Some(codeview) = debug.codeview_pdb70_debug_info {
            /*
            microsoft_download(
                &String::from_utf8(codeview.filename.to_vec())
                    .unwrap_or_default()
                    .trim_matches(char::from(0)),
                sig_to_uuid(&codeview.signature, codeview.age).unwrap_or_default(),
            )
            .unwrap();
            */
            cache::fetch_pdb(
                &String::from_utf8(codeview.filename.to_vec())
                    .unwrap_or_default()
                    .trim_matches(char::from(0)),
                &sig_to_uuid(&codeview.signature, codeview.age).unwrap_or_default(),
            )
            .unwrap();
        }
    } else {
        //Err(Error::new(ErrorKind::Other, "pe.debug_data not found"))
    }

    Ok(())
}

fn main() {
    pretty_env_logger::init();

    let argv = App::new("flow-core")
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

    // this is just some test code
    let socket = argv
        .value_of("socket")
        .unwrap_or("/tmp/qemu-connector-bridge");
    let mut bridge = match BridgeConnector::connect(socket) {
        Ok(s) => s,
        Err(e) => {
            println!("couldn't connect to bridge: {:?}", e);
            return;
        }
    };

    // os functionality should be located in core!
    let win = flow_win32::init(&mut bridge).unwrap();
    microsoft_download_ntos(&mut bridge, &win).unwrap();
}
