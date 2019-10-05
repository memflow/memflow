use libc::{dlsym, RTLD_DEFAULT};
use libc_print::*;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_ulonglong, c_void};

macro_rules! find_sym {
    ($type: ty, $name: expr) => {{
        let name_cstr = CString::new(stringify!($name)).unwrap();
        let addr = unsafe { dlsym(RTLD_DEFAULT, name_cstr.as_ptr()) };
        if addr.is_null() {
            libc_eprintln!("{} not found!", stringify!($name));
            None
        } else {
            libc_eprintln!("found {} at {:?}", stringify!($name), addr);
            let func: $type = unsafe { std::mem::transmute(addr) };
            Some(func)
        }
    }};
}

lazy_static! {
    // void *cpu_physical_memory_map(hwaddr addr, hwaddr *plen, int is_write)
    pub static ref CPU_PHYSICAL_MEMORY_MAP: Option<extern "C" fn(addr: c_ulonglong, plen: *mut c_ulonglong, is_write: c_int) -> *mut c_void> = {
        find_sym!(extern "C" fn(addr: c_ulonglong, plen: *mut c_ulonglong, is_write: c_int) -> *mut c_void, cpu_physical_memory_map)
    };

    // void cpu_physical_memory_unmap(void *buffer, hwaddr len, int is_write, hwaddr access_len)
    pub static ref CPU_PHYSICAL_MEMORY_UNMAP: Option<extern "C" fn(buffer: *mut c_void, len: c_ulonglong, is_write: c_int, access_len: c_ulonglong) -> ()> = {
        find_sym!(extern "C" fn(buffer: *mut c_void, len: c_ulonglong, is_write: c_int, access_len: c_ulonglong) -> (), cpu_physical_memory_unmap)
    };

    // TODO: seperate nicely

    // void qemu_mutex_lock_iothread_impl(const char *file, int line);
    pub static ref QEMU_MUTEX_LOCK_IOTHREAD_IMPL: Option<extern "C" fn(file: *const c_char, line: c_int) -> ()> = {
        find_sym!(extern "C" fn(file: *const c_char, line: c_int) -> (), qemu_mutex_lock_iothread_impl)
    };

    // void qemu_mutex_unlock_iothread(void);
    pub static ref QEMU_MUTEX_UNLOCK_IOTHREAD: Option<extern "C" fn() -> ()> = {
        find_sym!(extern "C" fn() -> (), qemu_mutex_unlock_iothread)
    };

    // CPUState *mon_get_cpu(void)
    pub static ref MON_GET_CPU: Option<extern "C" fn() -> *mut c_void> = {
        find_sym!(extern "C" fn() -> *mut c_void, mon_get_cpu)
    };

    // CPUArchState *mon_get_cpu_env(void)
    pub static ref MON_GET_CPU_ENV: Option<extern "C" fn() -> *mut c_void> = {
        find_sym!(extern "C" fn() -> *mut c_void, mon_get_cpu_env)
    };
}

// TODO: implement those later
/*
size_t qemu_ram_pagesize(RAMBlock *block);
size_t qemu_ram_pagesize_largest(void);
*/
