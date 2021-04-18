use memflow::os::*;
use memflow::plugins::*;

// os_process_address_list_callback
// void os_process_address_list(struct OsInstance *os);

// os_process_info_list_callback
// os_process_info_list

// os_process_info_by_address

// os_process_info_by_name

// os_process_info_by_pid

// os_process_by_info

// os_into_process_by_info

// os_process_by_address
// os_process_by_name
// os_process_by_pid

// os_into_process_by_address
// os_into_process_by_name
// os_into_process_by_pid

// os_module_address_list_callback
// os_module_list_callback
// os_module_list

// os_module_by_address
// os_module_by_name

// os_info

/// Returns a reference to the [`PhysicalMemory`] object this OS uses.
/// The [`PhysicalMemory`] usually is just the Connector this OS was intitialized with.
///
/// If no connector is used `null` is returned.
///
/// # Safety
///
/// `os` must point to a valid `OsInstance` that was created using one of the provided
/// functions.
#[no_mangle]
pub unsafe extern "C" fn os_phys_mem(os: &mut OsInstance) -> Option<&mut PhysicalMemoryInstance> {
    os.phys_mem()
}

/// Returns a reference to the [`VirtualMemory`] object this OS uses.
///
/// If no [`VirtualMemory`] object is used `null` is returned.
///
/// # Safety
///
/// `os` must point to a valid `OsInstance` that was created using one of the provided
/// functions.
#[no_mangle]
pub unsafe extern "C" fn os_virt_mem(os: &mut OsInstance) -> Option<&mut VirtualMemoryInstance> {
    os.virt_mem()
}
