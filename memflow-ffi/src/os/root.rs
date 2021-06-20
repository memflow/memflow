#[allow(unused)]
pub use memflow::mem::phys_mem::*;
#[allow(unused)]
pub use memflow::mem::virt_mem::*;
#[allow(unused)]
pub use memflow::os::*;
#[allow(unused)]
pub use memflow::plugins::*;

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
//#[no_mangle]
//pub unsafe extern "C" fn os_phys_mem<'a>(os: &'a mut OsInstanceArcBox) -> Option<&'a mut CGlueBoxPhysicalMemory> {
//}

//#[no_mangle]
//pub unsafe extern "C" fn os_virt_mem(os: &mut OsInstanceArcBox) -> Option<&mut CGlueBoxVirtualMemory> {
//    os.virt_mem()
//}
