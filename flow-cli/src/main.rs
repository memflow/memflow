use clap::{App, Arg};
use pretty_env_logger;
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;

use flow_core::address::{Address, Length};
use flow_core::mem::VirtualRead;
use flow_qemu::BridgeConnector;
use flow_win32;
use flow_win32::cache;
use flow_win32::win::Windows;
use goblin::pe::{options::ParseOptions, PE};
use pdb::{FallibleIterator, PdbInternalSectionOffset};

fn print_row(offset: PdbInternalSectionOffset, kind: &str, name: pdb::RawString<'_>) {
    println!(
        "{:x}\t{:x}\t{}\t{}",
        offset.section, offset.offset, kind, name
    );
}

fn print_symbol(symbol: &pdb::Symbol<'_>) -> pdb::Result<()> {
    match symbol.parse()? {
        pdb::SymbolData::Public(data) => {
            print_row(data.offset, "function", data.name);
        }
        pdb::SymbolData::Data(data) => {
            print_row(data.offset, "data", data.name);
        }
        pdb::SymbolData::Procedure(data) => {
            print_row(data.offset, "function", data.name);
        }
        _ => {
            // ignore everything else
        }
    }

    Ok(())
}

fn walk_symbols(mut symbols: pdb::SymbolIter<'_>) -> pdb::Result<()> {
    println!("segment\toffset\tkind\tname");

    while let Some(symbol) = symbols.next()? {
        match print_symbol(&symbol) {
            Ok(_) => (),
            Err(e) => eprintln!("error printing symbol {:?}: {}", symbol, e),
        }
    }

    Ok(())
}

fn dump_pdb(filename: &str) -> pdb::Result<()> {
    let file = std::fs::File::open(filename)?;
    let mut pdb = pdb::PDB::open(file)?;
    let symbol_table = pdb.global_symbols()?;
    println!("Global symbols:");
    walk_symbols(symbol_table.iter())?;

    println!("Module private symbols:");
    let dbi = pdb.debug_information()?;
    let mut modules = dbi.modules()?;
    while let Some(module) = modules.next()? {
        println!("Module: {}", module.object_file_name());
        let info = match pdb.module_info(&module)? {
            Some(info) => info,
            None => {
                println!("  no module info");
                continue;
            }
        };

        walk_symbols(info.symbols()?)?;
    }
    Ok(())
}

fn microsoft_download_ntos<T: VirtualRead>(mem: &mut T, win: &Windows) -> Result<()> {
    //let pdb = cache::fetch_pdb_from_mem(mem, win)?;
    //dump_pdb(pdb.to_str().unwrap_or_default()).unwrap();
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
    //microsoft_download_ntos(&mut bridge, &win).unwrap();
    
    // iterate processes
    
}
