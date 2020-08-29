// TODO: impl inventory

/*

    pub unsafe fn create_connector(
        &self,
        name: &str,
        args: &ConnectorArgs,
    ) -> Result<ConnectorInstance> {

    pub unsafe fn create_connector_default(&self, name: &str) -> Result<ConnectorInstance> {

*/

use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::PathBuf;

use memflow_core::{ConnectorArgs, ConnectorInstance, ConnectorInventory};

use memflow_core::mem::CloneablePhysicalMemory;

use crate::util::*;

use log::error;

#[no_mangle]
pub unsafe extern "C" fn inventory_try_new() -> Option<&'static mut ConnectorInventory> {
    ConnectorInventory::try_new()
        .map_err(inspect_err)
        .ok()
        .map(to_heap)
}

#[no_mangle]
pub unsafe extern "C" fn inventory_with_path(
    path: *const c_char,
) -> Option<&'static mut ConnectorInventory> {
    let rpath = CStr::from_ptr(path).to_string_lossy();
    ConnectorInventory::with_path(rpath.to_string())
        .map_err(inspect_err)
        .ok()
        .map(to_heap)
}

#[no_mangle]
pub unsafe extern "C" fn inventory_add_dir(
    inv: &mut ConnectorInventory,
    dir: *const c_char,
) -> i32 {
    let rdir = CStr::from_ptr(dir).to_string_lossy();

    match inv.add_dir(PathBuf::from(rdir.to_string())) {
        Ok(_) => 0,
        Err(err) => {
            error!("{}", err);
            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn inventory_create_connector(
    inv: &mut ConnectorInventory,
    name: *const c_char,
    args: *const c_char,
) -> Option<&'static mut ConnectorInstance> {
    let rname = CStr::from_ptr(name).to_string_lossy();

    if args.is_null() {
        inv.create_connector_default(&rname)
            .map_err(inspect_err)
            .ok()
            .map(to_heap)
    } else {
        let rargs = CStr::from_ptr(args).to_string_lossy();
        let conn_args = ConnectorArgs::try_parse_str(&rargs)
            .map_err(inspect_err)
            .ok()?;

        inv.create_connector(&rname, &conn_args)
            .map_err(inspect_err)
            .ok()
            .map(to_heap)
    }
}

#[no_mangle]
pub unsafe extern "C" fn connector_into_mem(
    inv: &'static mut ConnectorInstance,
) -> &'static mut &dyn CloneablePhysicalMemory {
    Box::leak(Box::new(inv))
}

#[no_mangle]
pub unsafe extern "C" fn connector_free(inv: &'static mut ConnectorInstance) {
    let _ = Box::from_raw(inv);
}

#[no_mangle]
pub unsafe extern "C" fn inventory_free(inv: &'static mut ConnectorInventory) {
    let _ = Box::from_raw(inv);
}
