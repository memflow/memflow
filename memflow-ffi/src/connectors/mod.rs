use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::PathBuf;

use memflow::connector::{ConnectorArgs, ConnectorInventory};

use crate::util::*;

use crate::mem::phys_mem::CloneablePhysicalMemoryObj;

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
/// ConnectorInventory is inherently unsafe, because it loads shared libraries which can not be
/// guaranteed to be safe.
#[no_mangle]
pub unsafe extern "C" fn inventory_scan() -> &'static mut ConnectorInventory {
    to_heap(ConnectorInventory::scan())
}

/// Create a new inventory with custom path string
///
/// # Safety
///
/// `path` must be a valid null terminated string
#[no_mangle]
pub unsafe extern "C" fn inventory_scan_path(
    path: *const c_char,
) -> Option<&'static mut ConnectorInventory> {
    let rpath = CStr::from_ptr(path).to_string_lossy();
    ConnectorInventory::scan_path(rpath.to_string())
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
pub unsafe extern "C" fn inventory_add_dir(
    inv: &mut ConnectorInventory,
    dir: *const c_char,
) -> i32 {
    let rdir = CStr::from_ptr(dir).to_string_lossy();

    inv.add_dir(PathBuf::from(rdir.to_string()))
        .int_result_logged()
}

/// Create a connector with given arguments
///
/// This creates an instance of a `CloneablePhysicalMemory`. To use it for physical memory
/// operations, please call `downcast_cloneable` to create a instance of `PhysicalMemory`.
///
/// Regardless, this instance needs to be freed using `connector_free`.
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
    inv: &mut ConnectorInventory,
    name: *const c_char,
    args: *const c_char,
) -> Option<&'static mut CloneablePhysicalMemoryObj> {
    let rname = CStr::from_ptr(name).to_string_lossy();

    if args.is_null() {
        inv.create_connector_default(&rname)
            .map_err(inspect_err)
            .ok()
            .map(to_heap)
            .map(|c| c as CloneablePhysicalMemoryObj)
            .map(to_heap)
    } else {
        let rargs = CStr::from_ptr(args).to_string_lossy();
        let conn_args = ConnectorArgs::parse(&rargs).map_err(inspect_err).ok()?;

        inv.create_connector(&rname, &conn_args)
            .map_err(inspect_err)
            .ok()
            .map(to_heap)
            .map(|c| c as CloneablePhysicalMemoryObj)
            .map(to_heap)
    }
}

/// Clone a connector
///
/// This method is useful when needing to perform multithreaded operations, as a connector is not
/// guaranteed to be thread safe. Every single cloned instance also needs to be freed using
/// `connector_free`.
///
/// # Safety
///
/// `conn` has to point to a a valid `CloneablePhysicalMemory` created by one of the provided
/// functions.
#[no_mangle]
pub unsafe extern "C" fn connector_clone(
    conn: &CloneablePhysicalMemoryObj,
) -> &'static mut CloneablePhysicalMemoryObj {
    trace!("connector_clone: {:?}", conn as *const _);
    Box::leak(Box::new(Box::leak(conn.clone_box())))
}

/// Free a connector instance
///
/// # Safety
///
/// `conn` has to point to a valid `CloneablePhysicalMemoryObj` created by one of the provided
/// functions.
///
/// There has to be no instance of `PhysicalMemory` created from the input `conn`, because they
/// will become invalid.
#[no_mangle]
pub unsafe extern "C" fn connector_free(conn: &'static mut CloneablePhysicalMemoryObj) {
    trace!("connector_free: {:?}", conn as *mut _);
    let _ = Box::from_raw(*Box::from_raw(conn));
}

/// Free a connector inventory
///
/// # Safety
///
/// `inv` must point to a valid `ConnectorInventory` that was created using one of the provided
/// functions.
#[no_mangle]
pub unsafe extern "C" fn inventory_free(inv: &'static mut ConnectorInventory) {
    trace!("inventory_free: {:?}", inv as *mut _);
    let _ = Box::from_raw(inv);
}
