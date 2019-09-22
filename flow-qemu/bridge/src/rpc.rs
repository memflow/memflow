use libc_print::*;

use std::io;

use tokio::io::AsyncRead;
use tokio::net::UnixListener;
use tokio::prelude::*;
use tokio::runtime::current_thread;

use capnp::{capability::Promise, Error};
use capnp_rpc::{pry, rpc_twoparty_capnp, twoparty, RpcSystem};

use crate::bridge_capnp::bridge;
use crate::cpu;
use crate::mem;

struct BridgeImpl;

impl bridge::Server for BridgeImpl {
    // readPhysicalMemory @0 (address :UInt64, length :UInt64) -> (memory :MemoryRegion);
    fn read_physical_memory(
        &mut self,
        params: bridge::ReadPhysicalMemoryParams,
        mut results: bridge::ReadPhysicalMemoryResults,
    ) -> Promise<(), Error> {
        let address = pry!(params.get()).get_address();
        let length = pry!(params.get()).get_length();
        let data = mem::phys_read(address, length).unwrap_or_else(|_e| Vec::new());
        results.get().set_data(&data);
        Promise::ok(())
    }

    // writePhysicalMemory @1 (address :UInt64, data: Data) -> (length :UInt64);
    fn write_physical_memory(
        &mut self,
        params: bridge::WritePhysicalMemoryParams,
        mut results: bridge::WritePhysicalMemoryResults,
    ) -> Promise<(), Error> {
        let address = pry!(params.get()).get_address();
        let data = pry!(pry!(params.get()).get_data());
        let len = mem::phys_write(address, &data.to_vec()).unwrap_or_else(|_e| 0);
        results.get().set_length(len);
        Promise::ok(())
    }

    // TODO: test
    // readRegisters @2 () -> (data: Data);
    fn read_registers(
        &mut self,
        params: bridge::ReadRegistersParams,
        mut results: bridge::ReadRegistersResults,
    ) -> Promise<(), Error> {
        // TODO:
        cpu::state().unwrap();
        Promise::ok(())
    }
}

pub fn listen(url: &str) -> io::Result<()> {
    let listener = UnixListener::bind(url)?;

    let bridge = bridge::ToClient::new(BridgeImpl).into_client::<::capnp_rpc::Server>();

    current_thread::block_on_all(
        listener
            .incoming()
            .map_err(|e| { libc_eprintln!("client accept failed: {:?}", e) })
            .for_each(|s| {
                libc_eprintln!("client connected");

                //s.set_nodelay(true)?;
                let (reader, writer) = s.split();
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
                Ok(())
            }),
    )
    .map_err(|_e| io::Error::new(io::ErrorKind::Other, "unable to listen for connections"))
    .and_then(|_v| Ok(()))
}
