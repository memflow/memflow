use log::{debug, info, trace};

use flow_core::error::{Error, Result};
use std::net::SocketAddr;
use url::Url;

use tokio::io::AsyncRead;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::runtime::current_thread::Runtime;

#[cfg(any(unix))]
use tokio::net::UnixStream;

use capnp::capability::Promise;
use capnp_rpc::{pry, rpc_twoparty_capnp, twoparty, RpcSystem};

use flow_core::address::{Address, Length, Page, PhysicalAddress};
use flow_core::arch::Architecture;
use flow_core::mem::*;

use crate::bridge_capnp::bridge;

pub struct BridgeClient {
    bridge: bridge::Client,
    runtime: Runtime,
}

#[cfg(any(unix))]
fn connect_unix(path: &str, _opts: Vec<&str>) -> Result<BridgeClient> {
    info!("trying to connect via unix -> {}", path);

    let mut runtime = Runtime::new().unwrap();
    let stream = runtime.block_on(UnixStream::connect(path))?;
    let (reader, writer) = stream.split();

    info!("unix connection established -> {}", path);

    Ok(BridgeClient {
        bridge: connect_rpc(&mut runtime, reader, writer)?,
        runtime,
    })
}

#[cfg(not(any(unix)))]
fn connect_unix(_path: &str, _opts: Vec<&str>) -> Result<BridgeClient> {
    Err(Error::new("unix sockets are not supported on this os"))
}

fn connect_tcp(path: &str, opts: Vec<&str>) -> Result<BridgeClient> {
    info!("trying to connect via tcp -> {}", path);

    let addr = path.parse::<SocketAddr>().map_err(Error::new)?;

    let mut runtime = Runtime::new().unwrap();
    let stream = runtime.block_on(TcpStream::connect(&addr))?;

    info!("tcp connection established -> {}", path);

    if opts.iter().any(|&o| o == "nodelay") {
        info!("trying to set TCP_NODELAY on socket");
        stream.set_nodelay(true).unwrap();
    }

    let (reader, writer) = stream.split();

    Ok(BridgeClient {
        bridge: connect_rpc(&mut runtime, reader, writer)?,
        runtime,
    })
}

fn connect_rpc<T, U>(runtime: &mut Runtime, reader: T, writer: U) -> Result<bridge::Client>
where
    T: ::std::io::Read + 'static,
    U: ::std::io::Write + 'static,
{
    let network = Box::new(twoparty::VatNetwork::new(
        reader,
        std::io::BufWriter::new(writer),
        rpc_twoparty_capnp::Side::Client,
        Default::default(),
    ));

    let mut rpc_system = RpcSystem::new(network, None);
    let bridge: bridge::Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);

    runtime.spawn(rpc_system.map_err(|_e| ()));

    Ok(bridge)
}

impl BridgeClient {
    pub fn connect(urlstr: &str) -> Result<BridgeClient> {
        let url = Url::parse(urlstr).map_err(Error::new)?;

        let path = url
            .path()
            .split(',')
            .next()
            .ok_or_else(|| Error::new("invalid url"))?;
        let opts = url.path().split(',').skip(1).collect::<Vec<_>>();

        match url.scheme() {
            "unix" => connect_unix(path, opts),
            "tcp" => connect_tcp(path, opts),
            _ => Err(Error::new("invalid url scheme")),
        }
    }

    pub fn read_registers(&mut self) -> Result<Vec<u8>> {
        let request = self.bridge.read_registers_request();
        self.runtime
            .block_on(request.send().promise.and_then(|_r| Promise::ok(())))
            .map_err(|_e| Error::new("unable to read registers"))
            .and_then(|_v| Ok(Vec::new()))
    }
}

impl AccessPhysicalMemory for BridgeClient {
    // physRead @0 (address :UInt64, length :UInt64) -> (data :Data);
    fn phys_read_raw_into(&mut self, addr: PhysicalAddress, out: &mut [u8]) -> Result<()> {
        trace!("phys_read_raw_into({:?}, {:?})", addr.address, out.len());

        let mut request = self.bridge.phys_read_request();
        request.get().set_address(addr.as_u64());
        request.get().set_length(out.len() as u64);
        let mem =
            self.runtime
                .block_on(request.send().promise.and_then(|response| {
                    Promise::ok(Vec::from(pry!(pry!(response.get()).get_data())))
                }))
                .map_err(|_e| Error::new("unable to read memory"))
                .and_then(Ok)?;
        // TODO: use new read method
        // TODO: speedup and prevent loop!
        mem.iter().enumerate().for_each(|(i, b)| {
            out[i] = *b;
        });
        Ok(())
    }

    // physWrite @1 (address :UInt64, data: Data) -> (length :UInt64);
    fn phys_write_raw(&mut self, addr: PhysicalAddress, data: &[u8]) -> Result<()> {
        trace!("phys_write_raw({:?})", addr.address);

        let mut request = self.bridge.phys_write_request();
        request.get().set_address(addr.as_u64());
        request.get().set_data(data);
        self.runtime
            .block_on(
                request.send().promise.and_then(|response| {
                    Promise::ok(Length::from(pry!(response.get()).get_length()))
                }),
            )
            .map_err(|_e| Error::new("unable to write memory"))
            .and_then(Ok)?;
        Ok(())
    }
}

impl BridgeClient {
    // virtRead @2 (arch: UInt8, dtb :UInt64, address :UInt64, length :UInt64) -> (data: Data);
    fn virt_read_chunk(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        len: Length,
    ) -> Result<Vec<u8>> {
        let mut request = self.bridge.virt_read_request();
        request.get().set_arch(arch.as_u8());
        request.get().set_dtb(dtb.as_u64());
        request.get().set_address(addr.as_u64());
        request.get().set_length(len.as_u64());
        self.runtime
            .block_on(
                request.send().promise.and_then(|response| {
                    Promise::ok(Vec::from(pry!(pry!(response.get()).get_data())))
                }),
            )
            .map_err(|_e| Error::new("unable to read memory"))
            .and_then(Ok)
    }

    // virtWrite @3 (arch: UInt8, dtb: UInt64, address :UInt64, data: Data) -> (length :UInt64);
    fn virt_write_chunk(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        data: &[u8],
    ) -> Result<Length> {
        let mut request = self.bridge.virt_write_request();
        request.get().set_arch(arch.as_u8());
        request.get().set_dtb(dtb.as_u64());
        request.get().set_address(addr.as_u64());
        request.get().set_data(data);
        self.runtime
            .block_on(
                request.send().promise.and_then(|response| {
                    Promise::ok(Length::from(pry!(response.get()).get_length()))
                }),
            )
            .map_err(|_e| Error::new("unable to write memory"))
            .and_then(Ok)
    }
}

//
// TODO: split up sections greater than 32mb into multiple packets due to capnp limitations!
//
impl AccessVirtualMemory for BridgeClient {
    fn virt_read_raw_into(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        out: &mut [u8],
    ) -> Result<()> {
        trace!(
            "virt_read_raw_into({:?}, {:?}, {:?}, {:?})",
            arch,
            dtb,
            addr,
            out.len()
        );

        if out.len() > Length::from_mb(32).as_usize() {
            info!("virt_read_raw_into(): reading multiple 32mb chunks");

            let mut base = addr;
            let end = addr + Length::from(out.len());
            while base < end {
                let mut clamped_len = Length::from_mb(32);
                if base + clamped_len > end {
                    clamped_len = end - base;
                }

                // TODO: improve this with new read method
                info!("virt_read_raw_into(): reading chunk at {:x}", base);
                let mem = self.virt_read_chunk(arch, dtb, base, clamped_len)?;
                let start = (base - addr).as_usize();
                mem.iter().enumerate().for_each(|(i, b)| {
                    out[start + i] = *b;
                });

                base += clamped_len;
            }
        } else {
            // TODO: improve with new read method
            let mem = self.virt_read_chunk(arch, dtb, addr, Length::from(out.len()))?;
            mem.iter().enumerate().for_each(|(i, b)| {
                out[i] = *b;
            });
        }
        Ok(())
    }

    fn virt_write_raw_from(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        data: &[u8],
    ) -> Result<()> {
        // TODO: implement chunk logic
        debug!("virt_write_raw_from({:?}, {:?}, {:?})", arch, dtb, addr);
        self.virt_write_chunk(arch, dtb, addr, data)?;
        Ok(())
    }

    fn virt_page_info(
        &mut self,
        _arch: Architecture,
        _dtb: Address,
        _addr: Address,
    ) -> Result<Page> {
        // TODO:
        Err(Error::new("not implemented yet"))
    }
}
