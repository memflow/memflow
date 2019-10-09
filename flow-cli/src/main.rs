use pretty_env_logger;
use clap::{App, Arg};
use std::io::{Error, ErrorKind, Result};

use flow_qemu::BridgeConnector;
use flow_win32;
use flow_win32::Windows;
use mem::VirtualRead;
use address::{Address, Length};
use goblin::pe::{options::ParseOptions, PE};

use uuid::{Uuid, BytesError};
use std::io::Cursor;
use byteorder::{ByteOrder, LittleEndian};
use byteorder::ReadBytesExt;

fn microsoft_download(module: &str, uuid: Uuid) -> Result<()> {
    println!("downloading {} {}", module, uuid.to_string());


    //         use std::fs::File;
    //         use std::path::Path;

    //         let path = Path::new(concat!("cache/", $name));
    //         if !path.exists() {
    //             let url = format!(
    //                 "https://msdl.microsoft.com/download/symbols/{}/{}/{}",
    //                 $name, $id, $name
    //             );

    //             let mut response = reqwest::get(&url).expect(concat!("get ", $name));
    //             let mut target = File::create(path).expect(concat!("create ", $name));
    //             response
    //                 .copy_to(&mut target)
    //                 .expect(concat!("download ", $name));
    //         }

    //         std::fs::read(path).expect(concat!("open ", $name))

    Ok(())
}

fn sig_to_uuid(sig: &[u8; 16]) -> std::result::Result<Uuid, BytesError> {
    let mut rdr = Cursor::new(sig);
    Ok(Uuid::from_fields(rdr.read_u32::<LittleEndian>().unwrap_or_default(), // TODO: fix error handling
                         rdr.read_u16::<LittleEndian>().unwrap_or_default(),
                         rdr.read_u16::<LittleEndian>().unwrap_or_default(),
                         &sig[8..])?)
}

fn microsoft_download_ntos<T: VirtualRead>(mem: &mut T, win: &Windows) -> Result<()>{
    let ntos_buf = mem.virt_read(win.dtb.arch, win.dtb.dtb, win.kernel_base, Length::from_mb(32)).unwrap();

    let mut pe_opts = ParseOptions::default();
    pe_opts.resolve_rva = false;

    let pe = match PE::parse_with_opts(&ntos_buf, &pe_opts) {
        Ok(pe) => {
            //println!("find_x64_with_va: found pe header:\n{:?}", pe);
            pe
        },
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, "unable to parse pe header"));
        }
    };

    if let Some(debug) = pe.debug_data {
        println!("debug_data: {:?}", debug);
        println!("guid: {:?}", debug.guid().unwrap_or_default());
        microsoft_download(pe.name.unwrap_or_default(), sig_to_uuid(&debug.guid().unwrap_or_default()).unwrap_or_default())
    } else {
        Err(Error::new(ErrorKind::Other, "pe.debug_data not found"))
    }
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
