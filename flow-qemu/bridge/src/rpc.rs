use libc_print::*;

use std::convert::TryFrom;
use std::io::{self, Error, ErrorKind, Result};

use std::net::SocketAddr;
use url::Url;

use tokio::io::AsyncRead;
use tokio::net::{UnixListener, TcpListener};
use tokio::prelude::*;
use tokio::runtime::current_thread;

use capnp::{self, capability::Promise};
use capnp_rpc::{pry, rpc_twoparty_capnp, twoparty, RpcSystem};

use ::mem::{PhysicalRead, PhysicalWrite, VirtualRead, VirtualWrite};
use address::{Address, Length};
use arch::{Architecture, InstructionSet};
use vat::VatImpl;

use crate::bridge_capnp::bridge;
use crate::cpu;
use crate::mem;

struct BridgeImpl;

impl bridge::Server for BridgeImpl {
    // physRead @0 (address :UInt64, length :UInt64) -> (memory :MemoryRegion);
    fn phys_read(
        &mut self,
        params: bridge::PhysReadParams,
        mut results: bridge::PhysReadResults,
    ) -> Promise<(), capnp::Error> {
        let address = Address::from(pry!(params.get()).get_address());
        let length = Length::from(pry!(params.get()).get_length());
        let data = mem::Wrapper::new()
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
        let address = Address::from(pry!(params.get()).get_address());
        let data = pry!(pry!(params.get()).get_data());
        let len = mem::Wrapper::new()
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
        let ins = pry!(InstructionSet::try_from(pry!(params.get()).get_arch()));
        let dtb = Address::from(pry!(params.get()).get_dtb());
        let address = Address::from(pry!(params.get()).get_address());
        let length = Length::from(pry!(params.get()).get_length());
        let data = VatImpl::new(mem::Wrapper::new())
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
        let ins = pry!(InstructionSet::try_from(pry!(params.get()).get_arch()));
        let dtb = Address::from(pry!(params.get()).get_dtb());
        let address = Address::from(pry!(params.get()).get_address());
        let data = pry!(pry!(params.get()).get_data());
        let len = VatImpl::new(mem::Wrapper::new())
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
        cpu::state().unwrap();
        Promise::ok(())
    }
}

fn listen_rpc<T, U>(bridge: &bridge::Client, reader: T, writer: U)
    where T: ::std::io::Read + 'static,
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

pub fn listen(urlstr: &str) -> Result<()> {
    // TODO: error convert
    let url = Url::parse(urlstr)
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    let path = url.path().split(",").nth(0).ok_or_else(|| Error::new(ErrorKind::Other, "invalid url"))?;
    let opts = url.path().split(",").skip(1).collect::<Vec<_>>();

    match url.scheme() {
        "unix" => {
            let listener = UnixListener::bind(path)?;
            let bridge = bridge::ToClient::new(BridgeImpl).into_client::<::capnp_rpc::Server>();

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
        },
        "tcp" => {
            let addr = path.parse::<SocketAddr>()
                .map_err(|e| Error::new(ErrorKind::Other, e))?;
            let listener = TcpListener::bind(&addr)?;
            let bridge = bridge::ToClient::new(BridgeImpl).into_client::<::capnp_rpc::Server>();

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
        },
        _ => {
            Err(Error::new(ErrorKind::Other, "invalid url scheme"))
        }
    }
}
