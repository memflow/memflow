#ifndef MEMFLOW_WIN32_H
#define MEMFLOW_WIN32_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include "memflow.h"

typedef struct Kernel_FFIMemory__FFIVirtualTranslate Kernel_FFIMemory__FFIVirtualTranslate;

typedef struct Win32ModuleInfo Win32ModuleInfo;

typedef struct Win32ProcessInfo Win32ProcessInfo;

typedef struct Win32Process_FFIVirtualMemory Win32Process_FFIVirtualMemory;

typedef Kernel_FFIMemory__FFIVirtualTranslate Kernel;

typedef struct StartBlock {
    Address kernel_hint;
    Address dtb;
} StartBlock;

typedef struct Win32Version {
    uint32_t nt_major_version;
    uint32_t nt_minor_version;
    uint32_t nt_build_number;
} Win32Version;

/**
 * Type alias for a PID.
 */
typedef uint32_t PID;

typedef Win32Process_FFIVirtualMemory Win32Process;

typedef struct Win32ArchOffsets {
    uintptr_t peb_ldr;
    uintptr_t ldr_list;
    uintptr_t ldr_data_base;
    uintptr_t ldr_data_size;
    uintptr_t ldr_data_full_name;
    uintptr_t ldr_data_base_name;
} Win32ArchOffsets;

typedef struct Win32ModuleListInfo {
    Address module_base;
    Win32ArchOffsets offsets;
} Win32ModuleListInfo;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Build a cloneable kernel object with default caching parameters
 *
 * This function will take ownership of the input `mem` object.
 *
 * # Safety
 *
 * `mem` must be a heap allocated memory reference, created by one of the API's functions.
 * Reference to it becomes invalid.
 */
Kernel *kernel_build(CloneablePhysicalMemoryObj *mem);

/**
 * Build a cloneable kernel object with custom caching parameters
 *
 * This function will take ownership of the input `mem` object.
 *
 * vat_cache_entries must be positive, or the program will panic upon memory reads or writes.
 *
 * # Safety
 *
 * `mem` must be a heap allocated memory reference, created by one of the API's functions.
 * Reference to it becomes invalid.
 */
Kernel *kernel_build_custom(CloneablePhysicalMemoryObj *mem,
                            uint64_t page_cache_time_ms,
                            PageType page_cache_flags,
                            uintptr_t page_cache_size_kb,
                            uint64_t vat_cache_time_ms,
                            uintptr_t vat_cache_entries);

Kernel *kernel_clone(const Kernel *kernel);

/**
 * Free a kernel object
 *
 * This will free the input `kernel` object (including the underlying memory object)
 *
 * # Safety
 *
 * `kernel` must be a valid reference heap allocated by one of the above functions.
 */
void kernel_free(Kernel *kernel);

/**
 * Destroy a kernel object and return its underlying memory object
 *
 * This will free the input `kernel` object, and return the underlying memory object. It will free
 * the object from any additional caching that `kernel` had in place.
 *
 * # Safety
 *
 * `kernel` must be a valid reference heap allocated by one of the above functions.
 */
CloneablePhysicalMemoryObj *kernel_destroy(Kernel *kernel);

StartBlock kernel_start_block(const Kernel *kernel);

Win32Version kernel_winver(const Kernel *kernel);

Win32Version kernel_winver_unmasked(const Kernel *kernel);

/**
 * Retrieve a list of peorcess addresses
 *
 * # Safety
 *
 * `buffer` must be a valid buffer of size at least `max_size`
 */
uintptr_t kernel_eprocess_list(Kernel *kernel, Address *buffer, uintptr_t max_size);

/**
 * Retrieve a list of processes
 *
 * This will fill `buffer` with a list of win32 process information. These processes will need to be
 * individually freed with `process_info_free`
 *
 * # Safety
 *
 * `buffer` must be a valid that can contain at least `max_size` references to `Win32ProcessInfo`.
 */
uintptr_t kernel_process_info_list(Kernel *kernel, Win32ProcessInfo **buffer, uintptr_t max_size);

Win32ProcessInfo *kernel_kernel_process_info(Kernel *kernel);

Win32ProcessInfo *kernel_process_info_from_eprocess(Kernel *kernel, Address eprocess);

/**
 * Retrieve process information by name
 *
 * # Safety
 *
 * `name` must be a valid null terminated string
 */
Win32ProcessInfo *kernel_process_info(Kernel *kernel, const char *name);

Win32ProcessInfo *kernel_process_info_pid(Kernel *kernel, PID pid);

/**
 * Create a process by looking up its name
 *
 * This will consume `kernel` and free it later on.
 *
 * # Safety
 *
 * `name` must be a valid null terminated string
 *
 * `kernel` must be a valid reference to `Kernel`. After the function the reference to it becomes
 * invalid.
 */
Win32Process *kernel_into_process(Kernel *kernel, const char *name);

/**
 * Create a process by looking up its PID
 *
 * This will consume `kernel` and free it later on.
 *
 * # Safety
 *
 * `kernel` must be a valid reference to `Kernel`. After the function the reference to it becomes
 * invalid.
 */
Win32Process *kernel_into_process_pid(Kernel *kernel, PID pid);

/**
 * Create a kernel process insatance
 *
 * This will consume `kernel` and free it later on.
 *
 * # Safety
 *
 * `kernel` must be a valid reference to `Kernel`. After the function the reference to it becomes
 * invalid.
 */
Win32Process *kernel_into_kernel_process(Kernel *kernel);

OsProcessModuleInfoObj *module_info_trait(Win32ModuleInfo *info);

/**
 * Free a win32 module info instance.
 *
 * Note that it is not the same as `OsProcessModuleInfoObj`, and those references need to be freed
 * manually.
 *
 * # Safety
 *
 * `info` must be a unique heap allocated reference to `Win32ModuleInfo`, and after this call the
 * reference will become invalid.
 */
void module_info_free(Win32ModuleInfo *info);

/**
 * Create a process with kernel and process info
 *
 * # Safety
 *
 * `kernel` must be a valid heap allocated reference to a `Kernel` object. After the function
 * call, the reference becomes invalid.
 */
Win32Process *process_with_kernel(Kernel *kernel, const Win32ProcessInfo *proc_info);

/**
 * Retrieve refernce to the underlying virtual memory object
 *
 * This will return a static reference to the virtual memory object. It will only be valid as long
 * as `process` if valid, and needs to be freed manually using `virt_free` regardless if the
 * process if freed or not.
 */
VirtualMemoryObj *process_virt_mem(Win32Process *process);

Win32Process *process_clone(const Win32Process *process);

/**
 * Frees the `process`
 *
 * # Safety
 *
 * `process` must be a valid heap allocated reference to a `Win32Process` object. After the
 * function returns, the reference becomes invalid.
 */
void process_free(Win32Process *process);

/**
 * Retrieve a process module list
 *
 * This will fill up to `max_len` elements into `out` with references to `Win32ModuleInfo` objects.
 *
 * These references then need to be freed with `module_info_free`
 *
 * # Safety
 *
 * `out` must be a valid buffer able to contain `max_len` references to `Win32ModuleInfo`.
 */
uintptr_t process_module_list(Win32Process *process, Win32ModuleInfo **out, uintptr_t max_len);

/**
 * Retrieve the main module of the process
 *
 * This function searches for a module with a base address
 * matching the section_base address from the ProcessInfo structure.
 * It then returns a reference to a newly allocated
 * `Win32ModuleInfo` object, if a module was found (null otherwise).
 *
 * The reference later needs to be freed with `module_info_free`
 *
 * # Safety
 *
 * `process` must be a valid Win32Process pointer.
 */
Win32ModuleInfo *process_main_module_info(Win32Process *process);

/**
 * Lookup a module
 *
 * This will search for a module called `name`, and return a reference to a newly allocated
 * `Win32ModuleInfo` object, if a module was found (null otherwise).
 *
 * The reference later needs to be freed with `module_info_free`
 *
 * # Safety
 *
 * `process` must be a valid Win32Process pointer.
 * `name` must be a valid null terminated string.
 */
Win32ModuleInfo *process_module_info(Win32Process *process, const char *name);

OsProcessInfoObj *process_info_trait(Win32ProcessInfo *info);

Address process_info_dtb(const Win32ProcessInfo *info);

Address process_info_section_base(const Win32ProcessInfo *info);

int32_t process_info_exit_status(const Win32ProcessInfo *info);

Address process_info_ethread(const Win32ProcessInfo *info);

Address process_info_wow64(const Win32ProcessInfo *info);

Address process_info_peb(const Win32ProcessInfo *info);

Address process_info_peb_native(const Win32ProcessInfo *info);

Address process_info_peb_wow64(const Win32ProcessInfo *info);

Address process_info_teb(const Win32ProcessInfo *info);

Address process_info_teb_wow64(const Win32ProcessInfo *info);

Win32ModuleListInfo process_info_module_info(const Win32ProcessInfo *info);

Win32ModuleListInfo process_info_module_info_native(const Win32ProcessInfo *info);

/**
 * Free a process information reference
 *
 * # Safety
 *
 * `info` must be a valid heap allocated reference to a Win32ProcessInfo structure
 */
void process_info_free(Win32ProcessInfo *info);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* MEMFLOW_WIN32_H */
