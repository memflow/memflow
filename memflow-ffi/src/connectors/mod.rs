// TODO: impl inventory

/*

    pub unsafe fn create_connector(
        &self,
        name: &str,
        args: &ConnectorArgs,
    ) -> Result<ConnectorInstance> {

    pub unsafe fn create_connector_default(&self, name: &str) -> Result<ConnectorInstance> {

*/

use std::ffi::{c_void, CStr};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::ptr;

use log::error;

use memflow_core::{ConnectorArgs, ConnectorInstance, ConnectorInventory};

#[no_mangle]
pub unsafe extern "C" fn inventory_try_new() -> *mut c_void {
    match ConnectorInventory::try_new() {
        Ok(inv) => Box::into_raw(Box::new(inv)) as *mut c_void,
        Err(err) => {
            error!("{}", err);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn inventory_with_path(path: *const c_char) -> *mut c_void {
    let rpath = CStr::from_ptr(path).to_string_lossy();
    match ConnectorInventory::with_path(rpath.to_string()) {
        Ok(inv) => Box::into_raw(Box::new(inv)) as *mut c_void,
        Err(err) => {
            error!("{}", err);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn inventory_add_dir(thisptr: *mut c_void, dir: *const c_char) -> i32 {
    let mut inv = Box::from_raw(thisptr as *mut ConnectorInventory);
    let rdir = CStr::from_ptr(dir).to_string_lossy();

    match inv.add_dir(PathBuf::from(rdir.to_string())) {
        Ok(_) => {
            Box::leak(inv);
            0
        }
        Err(err) => {
            error!("{}", err);
            Box::leak(inv);
            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn inventory_create_connector(
    thisptr: *mut c_void,
    name: *const c_char,
    args: *const c_char,
) -> *mut c_void {
    let inv = Box::from_raw(thisptr as *mut ConnectorInventory);
    let rname = CStr::from_ptr(name).to_string_lossy();

    if args.is_null() {
        match inv.create_connector_default(&rname) {
            Ok(conn) => {
                Box::leak(inv);
                Box::into_raw(Box::new(conn)) as *mut c_void
            }
            Err(err) => {
                error!("{}", err);
                Box::leak(inv);
                ptr::null_mut()
            }
        }
    } else {
        let rargs = CStr::from_ptr(args).to_string_lossy();
        match ConnectorArgs::try_parse_str(&rargs) {
            Ok(conn_args) => match inv.create_connector(&rname, &conn_args) {
                Ok(conn) => {
                    Box::leak(inv);
                    Box::into_raw(Box::new(conn)) as *mut c_void
                }
                Err(err) => {
                    error!("{}", err);
                    Box::leak(inv);
                    ptr::null_mut()
                }
            },
            Err(err) => {
                error!("{}", err);
                Box::leak(inv);
                ptr::null_mut()
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn connector_free(thisptr: *mut c_void) {
    let _inv = Box::from_raw(thisptr as *mut ConnectorInstance);
    // drop _inv
}

#[no_mangle]
pub unsafe extern "C" fn inventory_free(thisptr: *mut c_void) {
    let _inv = Box::from_raw(thisptr as *mut ConnectorInventory);
    // drop _inv
}
