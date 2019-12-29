use std::cell::RefCell;
use std::mem::transmute;
use std::ptr;
use std::rc::Rc;

use flow_core::connector::bridge::client::BridgeClient;

use flow_win32::{self, win::Windows};

/// # Safety
///
/// this function has to be called with an initialized bridge client
#[no_mangle]
pub unsafe extern "C" fn win32_init_bridge(
    ptr: *mut Rc<RefCell<BridgeClient>>,
) -> *mut Windows<BridgeClient> {
    let mut _bridge = &mut *ptr;
    match flow_win32::init(_bridge.clone()) {
        Ok(win) => transmute(Box::new(win)),
        Err(_e) => ptr::null_mut(),
    }
}
