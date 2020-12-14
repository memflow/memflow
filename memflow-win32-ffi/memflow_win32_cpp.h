#ifndef MEMFLOW_WIN32_HLAPI_H
#define MEMFLOW_WIN32_HLAPI_H

#include "memflow_cpp.h"
#include "memflow_win32.h"
#include "binddestr.h"

#ifndef NO_STL_CONTAINERS
#include <vector>
// Maximum number of entries allowed in the returned lists
#ifndef AUTO_VEC_SIZE
#define AUTO_VEC_SIZE 2048
#endif
#endif

struct CKernel;

struct CWin32ModuleInfo
    : BindDestr<Win32ModuleInfo, module_info_free>
{
    CWin32ModuleInfo(Win32ModuleInfo *modinfo)
        : BindDestr(modinfo) {}

    WRAP_FN_TYPE(COsProcessModuleInfo, module, info_trait);
};

struct CWin32Process
    : BindDestr<Win32Process, process_free>
{
    CWin32Process(Win32Process *process)
        : BindDestr(process) {}

    CWin32Process(CKernel &kernel, Win32ProcessInfo *info);

    WRAP_FN_TYPE(CWin32ModuleInfo, process, module_info);
    WRAP_FN_TYPE(CVirtualMemory, process, virt_mem);
};

struct CWin32ProcessInfo
    : BindDestr<Win32ProcessInfo, process_info_free>
{
    CWin32ProcessInfo(Win32ProcessInfo *info)
        : BindDestr(info) {}

    WRAP_FN_TYPE(COsProcessInfo, process_info, trait);
    WRAP_FN(process_info, dtb);
    WRAP_FN(process_info, section_base);
    WRAP_FN(process_info, wow64);
    WRAP_FN(process_info, peb);
    WRAP_FN(process_info, peb_native);
    WRAP_FN(process_info, peb_wow64);
    WRAP_FN(process_info, teb);
    WRAP_FN(process_info, teb_wow64);
    WRAP_FN(process_info, module_info);
    WRAP_FN(process_info, module_info_native);

    inline operator COsProcessInfo() {
        return this->trait();
    }
};

struct CKernel
    : BindDestr<Kernel, kernel_free>
{
    CKernel(Kernel *kernel)
        : BindDestr(kernel) {}

    CKernel(CCloneablePhysicalMemory &mem)
        : BindDestr(kernel_build(mem.invalidate())) {}

    CKernel(
        CCloneablePhysicalMemory &mem,
        uint64_t page_cache_time_ms,
        PageType page_cache_flags,
        uintptr_t page_cache_size_kb,
        uint64_t vat_cache_time_ms,
        uintptr_t vat_cache_entries
    ) : BindDestr(kernel_build_custom(
            mem.invalidate(),
            page_cache_time_ms,
            page_cache_flags,
            page_cache_size_kb,
            vat_cache_time_ms,
            vat_cache_entries
        )) {}

    WRAP_FN_TYPE(CKernel, kernel, clone);
    WRAP_FN_TYPE_INVALIDATE(CCloneablePhysicalMemory, kernel, destroy);
    WRAP_FN(kernel, start_block);
    WRAP_FN(kernel, winver);
    WRAP_FN(kernel, winver_unmasked);
    WRAP_FN(kernel, eprocess_list);
    WRAP_FN(kernel, process_info_list);
    WRAP_FN_TYPE(CWin32ProcessInfo, kernel, kernel_process_info);
    WRAP_FN_TYPE(CWin32ProcessInfo, kernel, process_info_from_eprocess);
    WRAP_FN_TYPE(CWin32ProcessInfo, kernel, process_info);
    WRAP_FN_TYPE(CWin32ProcessInfo, kernel, process_info_pid);
    WRAP_FN_TYPE_INVALIDATE(CWin32Process, kernel, into_process);
    WRAP_FN_TYPE_INVALIDATE(CWin32Process, kernel, into_process_pid);
    WRAP_FN_TYPE_INVALIDATE(CWin32Process, kernel, into_kernel_process);

#ifndef NO_STL_CONTAINERS
    // Manual eprocess_list impl
    std::vector<Address> eprocess_vec(size_t max_size) {
        Address *buf = (Address *)malloc(sizeof(Address *) * max_size);
        std::vector<Address> ret;

        if (buf) {
            size_t size = kernel_eprocess_list(this->inner, buf, max_size);

            for (size_t i = 0; i < size; i++)
                ret.push_back(buf[i]);

            free(buf);
        }

        return ret;
    }

    std::vector<Address> eprocess_vec() {
        return this->eprocess_vec(AUTO_VEC_SIZE);
    }

    // Manual process_info_list impl
    std::vector<CWin32ProcessInfo> process_info_vec(size_t max_size) {
        Win32ProcessInfo **buf = (Win32ProcessInfo **)malloc(sizeof(Win32ProcessInfo *) * max_size);
        std::vector<CWin32ProcessInfo> ret;

        if (buf) {
            size_t size = kernel_process_info_list(this->inner, buf, max_size);

            for (size_t i = 0; i < size; i++)
                ret.push_back(CWin32ProcessInfo(buf[i]));

            free(buf);
        }

        return ret;
    }

    std::vector<CWin32ProcessInfo> process_info_vec() {
        return this->process_info_vec(AUTO_VEC_SIZE);
    }
#endif
};

// Extra constructors we couldn't define inside the classes
CWin32Process::CWin32Process(CKernel &kernel, Win32ProcessInfo *info)
    : BindDestr(process_with_kernel(kernel.invalidate(), info)) {}

#endif
