use std::io::{Error, ErrorKind, Result};
use std::path::Path;

use tokio::io::AsyncRead;
use tokio::net::UnixStream;
use tokio::prelude::*;
use tokio::runtime::current_thread::Runtime;

use capnp::capability::Promise;
use capnp_rpc::{pry, rpc_twoparty_capnp, twoparty, RpcSystem};

use crate::bridge_capnp::bridge;

use flow_core::addr::{Address, Length};
use flow_core::arch::Architecture;
use flow_core::mem::{PhysicalRead, VirtualRead, PhysicalWrite, VirtualWrite};

pub struct BridgeConnector {
    bridge: bridge::Client,
    runtime: Runtime,
}

impl BridgeConnector {
    pub fn connect<'a, P: AsRef<Path>>(path: P) -> Result<BridgeConnector> {
        let mut runtime = Runtime::new().unwrap();
        let stream = runtime.block_on(UnixStream::connect(path))?;
        let (reader, writer) = stream.split();

        let network = Box::new(twoparty::VatNetwork::new(
            reader,
            std::io::BufWriter::new(writer),
            rpc_twoparty_capnp::Side::Client,
            Default::default(),
        ));

        let mut rpc_system = RpcSystem::new(network, None);
        let bridge: bridge::Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);

        runtime.spawn(rpc_system.map_err(|_e| ()));

        Ok(BridgeConnector {
            bridge: bridge,
            runtime: runtime,
        })
    }

    pub fn read_registers(&mut self) -> Result<Vec<u8>> {
        let request = self.bridge.read_registers_request();
        self.runtime
            .block_on(request.send().promise.and_then(|response| Promise::ok(())))
            .map_err(|_e| Error::new(ErrorKind::Other, "unable to read registers"))
            .and_then(|v| Ok(Vec::new()))
    }
}

impl PhysicalRead for BridgeConnector {
    fn phys_read(&mut self, addr: Address, len: Length) -> Result<Vec<u8>> {
        let mut request = self.bridge.phys_read_request();
        request.get().set_address(addr.addr);
        request.get().set_length(len.len);
        self.runtime
            .block_on(
                request.send().promise.and_then(|response| {
                    Promise::ok(Vec::from(pry!(pry!(response.get()).get_data())))
                }),
            )
            .map_err(|_e| Error::new(ErrorKind::Other, "unable to read memory"))
            .and_then(|v| Ok(v))
    }
}

impl VirtualRead for BridgeConnector {
    fn virt_read(&mut self, arch: Architecture, dtb: Address, addr: Address, len: Length) -> Result<Vec<u8>> {
        // TODO
        Err(Error::new(ErrorKind::Other, "not implemented yet"))
    }
}

impl PhysicalWrite for BridgeConnector {
    fn phys_write(&mut self, addr: Address, data: &Vec<u8>) -> Result<Length> {
        let mut request = self.bridge.phys_write_request();
        request.get().set_address(addr.addr);
        request.get().set_data(data);
        self.runtime
            .block_on(
                request
                    .send()
                    .promise
                    .and_then(|response| Promise::ok(Length::from(pry!(response.get()).get_length()))),
            )
            .map_err(|_e| Error::new(ErrorKind::Other, "unable to write memory"))
            .and_then(|v| Ok(v))
    }
}

impl VirtualWrite for BridgeConnector {
    fn write_mem(&mut self, arch: Architecture, dtb: Address, addr: Address, data: &Vec<u8>) -> Result<Length> {
        // TODO:
        Err(Error::new(ErrorKind::Other, "not implemented yet"))
    }
}
