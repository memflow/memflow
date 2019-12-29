use libc::c_char;
use std::cell::RefCell;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::mem::transmute;
use std::ptr;
use std::rc::Rc;

use flow_core::address::*;
use flow_core::arch::*;
use flow_core::connector::bridge::client::BridgeClient;
use flow_core::mem::*;

/// # Safety
///
/// this function has to be called with an initialized cstring
#[no_mangle]
pub unsafe extern "C" fn bridge_connect(urlstr: *const c_char) -> *mut Rc<RefCell<BridgeClient>> {
    if urlstr.is_null() {
        return ptr::null_mut();
    }

    let c_urlstr = CStr::from_ptr(urlstr); // TODO: check null and fail
    match BridgeClient::connect(&c_urlstr.to_string_lossy()) {
        Ok(br) => transmute(Box::new(Rc::new(RefCell::new(br)))),
        Err(_e) => ptr::null_mut(),
    }
}

/// # Safety
///
/// this function has to be called with an initialized bridge client
#[no_mangle]
pub unsafe extern "C" fn bridge_free(ptr: *mut BridgeClient) {
    if !ptr.is_null() {
        let _bridge: Box<Rc<RefCell<BridgeClient>>> = transmute(ptr);
        // drop bridge
    }
}

/// # Safety
///
/// this function has to be called with an initialized bridge client and memory buffer
// fn phys_read(&mut self, addr: Address, len: Length) -> Result<Vec<u8>>
#[no_mangle]
pub unsafe extern "C" fn bridge_phys_read(
    ptr: *mut Rc<RefCell<BridgeClient>>,
    addr: u64,
    buf: *mut u8,
    len: u64,
) -> u64 {
    if ptr.is_null() || buf.is_null() {
        // TODO: error result?
        return 0;
    }

    let mut _bridge = &mut *ptr;
    let bridgecp = _bridge.clone();
    let bridge = &mut bridgecp.borrow_mut();

    match bridge.phys_read(Address::from(addr), Length::from(len)) {
        Ok(b) => {
            // copy array
            let mut _buf = std::slice::from_raw_parts(buf, len as usize);
            _buf.to_owned().copy_from_slice(&b[..]);
            b.len() as u64
        }
        Err(_e) => 0,
    }
}

// fn phys_write(&mut self, addr: Address, data: &Vec<u8>) -> Result<Length>

// TODO: architecture interface (enum -> int)
// 1 -> X64
// 2 -> X86Pae
// 3 -> X86

/// # Safety
///
/// this function has to be called with an initialized bridge client and memory buffer
// fn virt_read(&mut self, arch: Architecture, dtb: Address, addr: Address, len: Length) -> Result<Vec<u8>>
#[no_mangle]
pub unsafe extern "C" fn bridge_virt_read(
    ptr: *mut Rc<RefCell<BridgeClient>>,
    ins: u8,
    dtb: u64,
    addr: u64,
    buf: *mut u8,
    len: u64,
) -> u64 {
    if ptr.is_null() || buf.is_null() {
        // TODO: error result?
        return 0;
    }

    let mut _bridge = &mut *ptr;
    let bridgecp = _bridge.clone();
    let bridge = &mut bridgecp.borrow_mut();

    let _ins = match InstructionSet::try_from(ins) {
        Ok(a) => a,
        Err(_e) => {
            return 0;
        }
    };

    match bridge.virt_read(
        Architecture::from(_ins),
        Address::from(dtb),
        Address::from(addr),
        Length::from(len),
    ) {
        Ok(b) => {
            // copy array
            let mut _buf = std::slice::from_raw_parts(buf, len as usize);
            _buf.to_owned().copy_from_slice(&b[..]);
            b.len() as u64
        }
        Err(_e) => 0,
    }
}

// fn virt_write(&mut self, arch: Architecture, dtb: Address, addr: Address, data: &Vec<u8>) -> Result<Length>
