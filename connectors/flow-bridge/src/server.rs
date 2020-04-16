use libc_print::*;

use flow_core::error::{Error, Result};
use std::convert::TryFrom;

use std::net::SocketAddr;
use url::Url;

use tokio::io::AsyncRead;
use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::runtime::current_thread;

#[cfg(any(unix))]
use tokio::net::UnixListener;

use capnp::{self, capability::Promise};
use capnp_rpc::{pry, rpc_twoparty_capnp, twoparty, RpcSystem};

use flow_core::address::{Address, Length};
use flow_core::arch::{Architecture, InstructionSet};
use flow_core::mem::*;
use flow_core::vat::VatImpl;

use std::cell::RefCell;
use std::rc::Rc;

use crate::bridge_capnp::bridge;

#[derive(Clone)]
pub struct BridgeServer<T: AccessPhysicalMemory> {
    pub mem: Rc<RefCell<T>>,
}

#[cfg(any(unix))]
fn listen_unix<T>(bridge: &BridgeServer<T>, path: &str, _opts: Vec<&str>) -> Result<()>
where
    T: AccessPhysicalMemory + 'static,
{
    let bridgecp = BridgeServer::<T> {
        mem: bridge.mem.clone(),
    };

    let listener = UnixListener::bind(path)?;
    let bridge = bridge::ToClient::new(bridgecp).into_client::<::capnp_rpc::Server>();

    current_thread::block_on_all(
        listener
            .incoming()
            .map_err(|e| {
                libc_eprintln!("client accept failed: {:?}", e);
            })
            .for_each(move |s| {
                libc_eprintln!("client connected");

                let (reader, writer) = s.split();
                listen_rpc(&bridge, reader, writer);

                Ok(())
            }),
    )
    .map_err(|_e| Error::new("unable to listen for connections"))
    .and_then(|_v| Ok(()))
}

#[cfg(not(any(unix)))]
fn listen_unix<T>(_bridge: &BridgeServer<T>, _path: &str, _opts: Vec<&str>) -> Result<()>
where
    T: AccessPhysicalMemory + 'static,
{
    Err(Error::new("unix sockets are not supported on this os"))
}

fn listen_tcp<T>(bridge: &BridgeServer<T>, path: &str, opts: Vec<&str>) -> Result<()>
where
    T: AccessPhysicalMemory + 'static,
{
    let bridgecp = BridgeServer::<T> {
        mem: bridge.mem.clone(),
    };

    let addr = path.parse::<SocketAddr>().map_err(Error::new)?;
    let listener = TcpListener::bind(&addr)?;
    let bridge = bridge::ToClient::new(bridgecp).into_client::<::capnp_rpc::Server>();

    current_thread::block_on_all(
        listener
            .incoming()
            .map_err(|e| {
                libc_eprintln!("client accept failed: {:?}", e);
            })
            .for_each(move |s| {
                libc_eprintln!("client connected");

                if opts.iter().filter(|&&o| o == "nodelay").nth(0).is_some() {
                    libc_eprintln!("trying to set TCP_NODELAY on socket");
                    s.set_nodelay(true).unwrap();
                }

                let (reader, writer) = s.split();
                listen_rpc(&bridge, reader, writer);

                Ok(())
            }),
    )
    .map_err(|_e| Error::new("unable to listen for connections"))
    .and_then(|_v| Ok(()))
}

fn listen_rpc<T, U>(bridge: &bridge::Client, reader: T, writer: U)
where
    T: ::std::io::Read + 'static,
    U: ::std::io::Write + 'static,
{
    let network = twoparty::VatNetwork::new(
        reader,
        std::io::BufWriter::new(writer),
        rpc_twoparty_capnp::Side::Server,
        Default::default(),
    );

    let rpc_system = RpcSystem::new(Box::new(network), Some(bridge.clone().client));
    current_thread::spawn(rpc_system.map_err(|e| {
        libc_eprintln!("error: {:?}", e);
    }));
}

impl<T: AccessPhysicalMemory + 'static> BridgeServer<T> {
    pub fn new(mem: Rc<RefCell<T>>) -> Self {
        BridgeServer { mem }
    }

    pub fn listen(&self, urlstr: &str) -> Result<()> {
        // TODO: error convert
        let url = Url::parse(urlstr).map_err(Error::new)?;

        let path = url
            .path()
            .split(',')
            .nth(0)
            .ok_or_else(|| Error::new("invalid url"))?;
        let opts = url.path().split(',').skip(1).collect::<Vec<_>>();

        // TODO: a cleaner way would be to split ServerBuilder / ServerImpl
        let selfcp = Self {
            mem: self.mem.clone(),
        };

        match url.scheme() {
            "unix" => listen_unix(&selfcp, path, opts),
            "tcp" => listen_tcp(&selfcp, path, opts),
            _ => Err(Error::new("invalid url scheme")),
        }
    }
}

impl<T: AccessPhysicalMemory + 'static> bridge::Server for BridgeServer<T> {
    // physRead @0 (address :UInt64, length :UInt64) -> (memory :MemoryRegion);
    fn phys_read(
        &mut self,
        params: bridge::PhysReadParams,
        mut results: bridge::PhysReadResults,
    ) -> Promise<(), capnp::Error> {
        let memcp = self.mem.clone();
        let memory = &mut memcp.borrow_mut();

        let address = Address::from(pry!(params.get()).get_address());
        let length = Length::from(pry!(params.get()).get_length());

        let mut data = vec![0; length.as_usize()];
        memory
            .phys_read_raw_into(address, &mut data)
            .unwrap_or_else(|_| ());
        results.get().set_data(&data);

        Promise::ok(())
    }

    // physWrite @1 (address :UInt64, data: Data) -> (length :UInt64);
    fn phys_write(
        &mut self,
        params: bridge::PhysWriteParams,
        mut results: bridge::PhysWriteResults,
    ) -> Promise<(), capnp::Error> {
        let memcp = self.mem.clone();
        let memory = &mut memcp.borrow_mut();

        let address = Address::from(pry!(params.get()).get_address());
        let data = pry!(pry!(params.get()).get_data());

        memory
            .phys_write_raw(address, &data.to_vec())
            .unwrap_or_else(|_| ());
        results.get().set_length(data.len() as u64);

        Promise::ok(())
    }

    // virtRead @2 (arch: UInt8, dtb :UInt64, address :UInt64, length :UInt64) -> (data: Data);
    fn virt_read(
        &mut self,
        params: bridge::VirtReadParams,
        mut results: bridge::VirtReadResults,
    ) -> Promise<(), capnp::Error> {
        let memory = &mut self.mem.borrow_mut();

        let ins = pry!(InstructionSet::try_from(pry!(params.get()).get_arch())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)));
        let dtb = Address::from(pry!(params.get()).get_dtb());
        let address = Address::from(pry!(params.get()).get_address());
        let length = Length::from(pry!(params.get()).get_length());

        let mut data = vec![0; length.as_usize()];
        VatImpl::new(&mut **memory)
            .virt_read_raw_into(Architecture::from(ins), dtb, address, &mut data)
            .unwrap_or_else(|_| ());
        results.get().set_data(&data);

        Promise::ok(())
    }

    // virtWrite @3 (arch: UInt8, dtb: UInt64, address :UInt64, data: Data) -> (length :UInt64);
    fn virt_write(
        &mut self,
        params: bridge::VirtWriteParams,
        mut results: bridge::VirtWriteResults,
    ) -> Promise<(), capnp::Error> {
        let memcp = self.mem.clone();
        let memory = &mut memcp.borrow_mut();

        let ins = pry!(InstructionSet::try_from(pry!(params.get()).get_arch())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)));
        let dtb = Address::from(pry!(params.get()).get_dtb());
        let address = Address::from(pry!(params.get()).get_address());
        let data = pry!(pry!(params.get()).get_data());

        VatImpl::new(&mut **memory)
            .virt_write_raw(Architecture::from(ins), dtb, address, &data.to_vec())
            .unwrap_or_else(|_| ());
        results.get().set_length(data.len() as u64);

        Promise::ok(())
    }

    // TODO: test
    // readRegisters @4 () -> (data: Data);
    fn read_registers(
        &mut self,
        _params: bridge::ReadRegistersParams,
        _results: bridge::ReadRegistersResults,
    ) -> Promise<(), capnp::Error> {
        // TODO:
        //cpu::state().unwrap();
        Promise::ok(())
    }
}
