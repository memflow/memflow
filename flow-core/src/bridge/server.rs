use libc_print::*;

use std::convert::TryFrom;
use std::io::{self, Error, ErrorKind, Result};

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

use crate::address::{Address, Length};
use crate::arch::{Architecture, InstructionSet};
use crate::mem::{PhysicalRead, PhysicalWrite, VirtualRead, VirtualWrite};
use crate::vat::{VatImpl, VirtualAddressTranslation};

use std::cell::RefCell;
use std::rc::Rc;

use crate::bridge_capnp::bridge;

#[derive(Clone)]
pub struct BridgeServer<T: PhysicalRead + PhysicalWrite> {
    pub mem: Rc<RefCell<T>>,
}

#[cfg(any(unix))]
fn listen_unix<T>(bridge: &BridgeServer<T>, path: &str, opts: Vec<&str>) -> Result<()>
where
    T: PhysicalRead + PhysicalWrite + 'static,
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
    .map_err(|_e| Error::new(ErrorKind::Other, "unable to listen for connections"))
    .and_then(|_v| Ok(()))
}

#[cfg(not(any(unix)))]
fn listen_unix<T>(bridge: &BridgeServer<T>, path: &str, opts: Vec<&str>) -> Result<()>
where
    T: PhysicalRead + PhysicalWrite + 'static,
{
    Err(Error::new(
        ErrorKind::Other,
        "unix sockets are not supported on this os",
    ))
}

fn listen_tcp<T>(bridge: &BridgeServer<T>, path: &str, opts: Vec<&str>) -> Result<()>
where
    T: PhysicalRead + PhysicalWrite + 'static,
{
    let bridgecp = BridgeServer::<T> {
        mem: bridge.mem.clone(),
    };

    let addr = path
        .parse::<SocketAddr>()
        .map_err(|e| Error::new(ErrorKind::Other, e))?;
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

                if let Some(_) = opts.iter().filter(|&&o| o == "nodelay").nth(0) {
                    libc_eprintln!("trying to set TCP_NODELAY on socket");
                    s.set_nodelay(true).unwrap();
                }

                let (reader, writer) = s.split();
                listen_rpc(&bridge, reader, writer);

                Ok(())
            }),
    )
    .map_err(|_e| Error::new(ErrorKind::Other, "unable to listen for connections"))
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

impl<T: PhysicalRead + PhysicalWrite + 'static> BridgeServer<T> {
    pub fn new(mem: Rc<RefCell<T>>) -> Self {
        BridgeServer { mem: mem }
    }

    pub fn listen(&self, urlstr: &str) -> Result<()> {
        // TODO: error convert
        let url = Url::parse(urlstr).map_err(|e| Error::new(ErrorKind::Other, e))?;

        let path = url
            .path()
            .split(",")
            .nth(0)
            .ok_or_else(|| Error::new(ErrorKind::Other, "invalid url"))?;
        let opts = url.path().split(",").skip(1).collect::<Vec<_>>();

        // TODO: a cleaner way would be to split ServerBuilder / ServerImpl
        let selfcp = Self {
            mem: self.mem.clone(),
        };

        match url.scheme() {
            "unix" => listen_unix(&selfcp, path, opts),
            "tcp" => listen_tcp(&selfcp, path, opts),
            _ => Err(Error::new(ErrorKind::Other, "invalid url scheme")),
        }
    }
}

impl<T: PhysicalRead + PhysicalWrite + 'static> bridge::Server for BridgeServer<T> {
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

        let data = memory
            .phys_read(address, length)
            .unwrap_or_else(|_e| Vec::new());
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

        let len = memory
            .phys_write(address, &data.to_vec())
            .unwrap_or_else(|_e| Length::from(0));
        results.get().set_length(len.as_u64());

        Promise::ok(())
    }

    // virtRead @2 (arch: UInt8, dtb :UInt64, address :UInt64, length :UInt64) -> (data: Data);
    fn virt_read(
        &mut self,
        params: bridge::VirtReadParams,
        mut results: bridge::VirtReadResults,
    ) -> Promise<(), capnp::Error> {
        let memcp = self.mem.clone();
        let memory = &mut memcp.borrow_mut();

        let ins = pry!(InstructionSet::try_from(pry!(params.get()).get_arch()));
        let dtb = Address::from(pry!(params.get()).get_dtb());
        let address = Address::from(pry!(params.get()).get_address());
        let length = Length::from(pry!(params.get()).get_length());

        let data = VatImpl::new(&mut **memory)
            .virt_read(Architecture::from(ins), dtb, address, length)
            .unwrap_or_else(|_e| Vec::new());
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

        let ins = pry!(InstructionSet::try_from(pry!(params.get()).get_arch()));
        let dtb = Address::from(pry!(params.get()).get_dtb());
        let address = Address::from(pry!(params.get()).get_address());
        let data = pry!(pry!(params.get()).get_data());

        let len = VatImpl::new(&mut **memory)
            .virt_write(Architecture::from(ins), dtb, address, &data.to_vec())
            .unwrap_or_else(|_e| Length::from(0));
        results.get().set_length(len.as_u64());

        Promise::ok(())
    }

    // TODO: test
    // readRegisters @4 () -> (data: Data);
    fn read_registers(
        &mut self,
        params: bridge::ReadRegistersParams,
        results: bridge::ReadRegistersResults,
    ) -> Promise<(), capnp::Error> {
        // TODO:
        //cpu::state().unwrap();
        Promise::ok(())
    }
}
