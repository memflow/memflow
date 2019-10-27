use libc_print::*;

use std::convert::TryFrom;
use std::io;

use tokio::io::AsyncRead;
use tokio::net::UnixListener;
use tokio::prelude::*;
use tokio::runtime::current_thread;

use capnp::{capability::Promise, Error};
use capnp_rpc::{pry, rpc_twoparty_capnp, twoparty, RpcSystem};

use ::mem::{PhysicalRead, PhysicalWrite, VirtualRead, VirtualWrite};
use address::{Address, Length};
use arch::{Architecture, InstructionSet};

use flow_va::VatImpl;

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
    ) -> Promise<(), Error> {
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
    ) -> Promise<(), Error> {
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
    ) -> Promise<(), Error> {
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
    ) -> Promise<(), Error> {
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
            .map_err(|e| libc_eprintln!("client accept failed: {:?}", e))
            .for_each(move |s| {
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
