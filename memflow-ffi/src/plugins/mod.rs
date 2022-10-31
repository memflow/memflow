use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::PathBuf;

use memflow::plugins::Inventory;
use memflow::plugins::{
    connector::{ConnectorInstanceArcBox, MuConnectorInstanceArcBox},
    os::{MuOsInstanceArcBox, OsInstanceArcBox},
};

use crate::util::*;
use memflow::cglue::result::IntResult;

use log::trace;

/// Create a new connector inventory
///
/// This function will try to find connectors using PATH environment variable
///
/// Note that all functions go through each directories, and look for a `memflow` directory,
/// and search for libraries in those.
///
/// # Safety
///
/// Inventory is inherently unsafe, because it loads shared libraries which can not be
/// guaranteed to be safe.
#[no_mangle]
pub unsafe extern "C" fn inventory_scan() -> &'static mut Inventory {
    to_heap(Inventory::scan())
}

/// Create a new inventory with custom path string
///
/// # Safety
///
/// `path` must be a valid null terminated string
#[no_mangle]
pub unsafe extern "C" fn inventory_scan_path(
    path: *const c_char,
) -> Option<&'static mut Inventory> {
    let rpath = CStr::from_ptr(path).to_string_lossy();
    Inventory::scan_path(rpath.to_string())
        .map_err(inspect_err)
        .ok()
        .map(to_heap)
}

/// Add a directory to an existing inventory
///
/// # Safety
///
/// `dir` must be a valid null terminated string
#[no_mangle]
pub unsafe extern "C" fn inventory_add_dir(inv: &mut Inventory, dir: *const c_char) -> i32 {
    let rdir = CStr::from_ptr(dir).to_string_lossy();

    inv.add_dir(PathBuf::from(rdir.to_string()))
        .into_int_result()
}

/// Create a connector with given arguments
///
/// This creates an instance of `ConnectorInstance`.
///
/// This instance needs to be dropped using `connector_drop`.
///
/// # Arguments
///
/// * `name` - name of the connector to use
/// * `args` - arguments to be passed to the connector upon its creation
///
/// # Safety
///
/// Both `name`, and `args` must be valid null terminated strings.
///
/// Any error strings returned by the connector must not be outputed after the connector gets
/// freed, because that operation could cause the underlying shared library to get unloaded.
#[no_mangle]
pub unsafe extern "C" fn inventory_create_connector(
    inv: &mut Inventory,
    name: *const c_char,
    args: *const c_char,
    out: &mut MuConnectorInstanceArcBox<'static>,
) -> i32 {
    let rname = CStr::from_ptr(name).to_string_lossy();

    if args.is_null() {
        inv.create_connector(&rname, None, None)
            .map_err(inspect_err)
            .into_int_out_result(out)
    } else {
        let rargs = CStr::from_ptr(args).to_string_lossy();
        str::parse(&rargs)
            .map_err(inspect_err)
            .and_then(|args| inv.create_connector(&rname, None, Some(&args)))
            .map_err(inspect_err)
            .into_int_out_result(out)
    }
}

/// Create a OS instance with given arguments
///
/// This creates an instance of `KernelInstance`.
///
/// This instance needs to be freed using `os_drop`.
///
/// # Arguments
///
/// * `name` - name of the OS to use
/// * `args` - arguments to be passed to the connector upon its creation
/// * `mem` - a previously initialized connector instance
/// * `out` - a valid memory location that will contain the resulting os-instance
///
/// # Remarks
///
/// The `mem` connector instance is being _moved_ into the os layer.
/// This means upon calling `os_drop` it is not unnecessary to call `connector_drop` anymore.
///
/// # Safety
///
/// Both `name`, and `args` must be valid null terminated strings.
///
/// Any error strings returned by the connector must not be outputed after the connector gets
/// freed, because that operation could cause the underlying shared library to get unloaded.
#[no_mangle]
pub unsafe extern "C" fn inventory_create_os(
    inv: &mut Inventory,
    name: *const c_char,
    args: *const c_char,
    mem: *mut ConnectorInstanceArcBox<'static>,
    out: &mut MuOsInstanceArcBox<'static>,
) -> i32 {
    let rname = CStr::from_ptr(name).to_string_lossy();
    let _args = CStr::from_ptr(args).to_string_lossy();

    let mem_obj = if mem.is_null() {
        None
    } else {
        let mem_obj = mem.read();
        // Zero out the data so that any automatic destructors on the other side do nothing.
        std::ptr::write_bytes(mem, 0, 1);
        Some(mem_obj)
    };

    if args.is_null() {
        inv.create_os(&rname, mem_obj, None)
            .map_err(inspect_err)
            .into_int_out_result(out)
    } else {
        let rargs = CStr::from_ptr(args).to_string_lossy();
        str::parse(&rargs)
            .map_err(inspect_err)
            .and_then(|args| inv.create_os(&rname, mem_obj, Some(&args)))
            .map_err(inspect_err)
            .into_int_out_result(out)
    }
}

/// Free a os plugin
///
/// # Safety
///
/// `os` must point to a valid `OsInstance` that was created using one of the provided
/// functions.
#[no_mangle]
pub unsafe extern "C" fn os_drop(os: &mut OsInstanceArcBox<'static>) {
    trace!("connector_drop: {:?}", os as *mut _);
    std::ptr::drop_in_place(os);
}

/// Clone a connector
///
/// This method is useful when needing to perform multithreaded operations, as a connector is not
/// guaranteed to be thread safe. Every single cloned instance also needs to be dropped using
/// `connector_drop`.
///
/// # Safety
///
/// `conn` has to point to a a valid `CloneablePhysicalMemory` created by one of the provided
/// functions.
#[no_mangle]
pub unsafe extern "C" fn connector_clone(
    conn: &ConnectorInstanceArcBox<'static>,
    out: &mut MuConnectorInstanceArcBox<'static>,
) {
    trace!("connector_clone: {:?}", conn as *const _);
    *out.as_mut_ptr() = conn.clone();
}

/// Free a connector instance
///
/// # Safety
///
/// `conn` has to point to a valid [`ConnectorInstance`](ConnectorInstanceArcBox) created by one of the provided
/// functions.
///
/// There has to be no instance of `PhysicalMemory` created from the input `conn`, because they
/// will become invalid.
#[no_mangle]
pub unsafe extern "C" fn connector_drop(conn: &mut ConnectorInstanceArcBox<'static>) {
    trace!("connector_drop: {:?}", conn as *mut _);
    std::ptr::drop_in_place(conn)
}

/// Free a connector inventory
///
/// # Safety
///
/// `inv` must point to a valid `Inventory` that was created using one of the provided
/// functions.
#[no_mangle]
pub unsafe extern "C" fn inventory_free(inv: &'static mut Inventory) {
    trace!("inventory_free: {:?}", inv as *mut _);
    let _ = Box::from_raw(inv);
}
