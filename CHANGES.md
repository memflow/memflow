# Release notes
All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## 0.2.0-beta11
- Added dtb1 and dtb2 fields to ProcessInfo structure
- Added a function to the process trait which allows overriding dtb1/dtb2 with a custom value

## 0.2.0-beta10
- Removed all 'Inner' types and replaced them with GAT - this also shifts memflow to a minimum rust version of 1.70.0
- Improved cache configuration when using plugins (usage: --connector kvm:::cache=true,cache_size=1kb,cache_time=10,cache_page_size=1000 where size and page_size is specified in hex)
- Added DelayedPhysicalMemory middleware (usage: --connector kvm:::delay=200 where delay is specified in microseconds)
- Added PhysicalMemoryMetrics middleware (usage: --connector kvm:::metrics=true)
- Updated FileIoMemory constructor with a default identity mapped memory mapping.
- Rewrote argument parser to properly handle quotes in complex arguments.

## 0.2.0-beta9
## 0.2.0-beta8
- Hotfix for new bumpalo release

## 0.2.0-beta7
- Unified and simplified plugin proc macros and updated their documentation

## 0.2.0-beta6
- Added additional export/import/section helpers
- Dependency updates

## 0.2.0-beta5
- Cleaned up plugin search paths and matched them with memflowup
- Improved error messages
- Plugins are resolved to their canonical path before adding
- Added VirtualTranslate as optional trait on Os
- Updated to latest cglue

## 0.2.0-beta4
- Added missing functions to retrieve exports/imports/sections from kernel modules
- Added functions to retrieve primary kernel module

## 0.2.0-beta3
- Allow for PhysicalMemoryView to fill in gaps with zeros
## 0.2.0-beta2
- Memory API and Address rework

## 0.2.0-beta1
- Entirely new cglue based plugin architecture and various other major improvements

## 0.1.5
- Added memflow::prelude::v1 and memflow_win32::prelude::v1 modules
- Added new fields to FFI
- Improved consistency of these function names in C FFI: `phys_read_raw` -> `phys_read_raw_into`, `page_size` -> `arch_page_size`.
- Added C++ bindings for the FFI
- Fixed core errors not displaying the full error message when wrapped in a win32 error
- Changed windows inventory search path from [user]/.local/lib/memflow to [user]/Documents/memflow
- Added {PWD} to inventory search path

Transitioning from C FFI to C++ FFI:
- `memflow.h`, and `memflow_win32.h` become `memflow_cpp.h`, and `memflow_win32_cpp.h`.
  - The headers still depend on `memflow.h`, and `memflow_win32.h`. They are just wrappers for safety, and ergonomics.
- Types transition from `Type *` to `CType`. Every `CType` include automatic object destruction, so there is no need for the `type_free` methods.
- `CType` contains a `Type *` inside. The pointer can still be `null`. Checking whether object is valid is still the same: `if (CType != NULL)`
- Methods are implemented as class members. Most methods loose their prefix. The change looks like this: `process_module_info(Win32Process *process, const char *name)` becomes `CWin32Process::module_info(this, const char *name)`.
  - Calling methods changes into calling a function on the object, instead of with the object. Example: `process_module_info(proc, "ntdll.dll")` becomes `proc.module_info("ntdll.dll")`.
  - Exception to this are `virt`, and `phys` read/write functions. They do not loose their prefix, because they do have the prefix in the Rust library. So, `virt_read_u64(mem, addr)` becomes `mem.virt_read_u64(addr)`.
- There are extra convenience functions that utilize STL's `string`, and `vector` containers. Getting process/module names, and lists becomes much simpler.

## 0.1.4
- Removed namespaces in FFI headers and unused dependencies
- Fixed connector errors not being shown properly
- Added `main_module_info()` helper function which retrieves the main module of a process
- Added the DLL path to the Win32ModuleInfo structure
- Fixed duplicated connectors being added to the inventory multiple times
- Renamed and deprecated the `ConnectorInventory::try_new()` and `ConnectorInventory::with_path()` functions. The new function names are `ConnectorInventory::scan()` and `ConnectorInventory::scan_path()`
- Added a `available_connectors()` function to the ConnectorInventory which returns all connectors that have been found on the system.
- Added a fallback signature for windows 10 for the win32 keyboard implementation in case the PE Header of the win32kbase.sys is paged out
- Added a `MemoryMap::open()` function to load a memory map in TOML format
