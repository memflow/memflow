# Release notes
All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## 0.1.5
- Added memflow::prelude::v1 and memflow_win32::prelude::v1 modules
- Added new fields to FFI
- Added C++ bindings for the FFI
- Fixed core errors not displaying the full error message when wrapped in a win32 error
- Changed windows inventory search path from [user]/.local/lib/memflow to [user]/Documents/memflow

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
