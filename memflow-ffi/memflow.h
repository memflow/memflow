#ifndef MEMFLOW_H
#define MEMFLOW_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
typedef void *Library;

/**
 * Identifies the byte order of a architecture
 *
 * This enum is used when reading/writing to/from the memory of a target system.
 * The memory will be automatically converted to the endianess memflow is currently running on.
 *
 * See the [wikipedia article](https://en.wikipedia.org/wiki/Endianness) for more information on the subject.
 */
enum Endianess
#ifdef __cplusplus
  : uint8_t
#endif // __cplusplus
 {
    /**
     * Little Endianess
     */
    LittleEndian,
    /**
     * Big Endianess
     */
    BigEndian,
};
#ifndef __cplusplus
typedef uint8_t Endianess;
#endif // __cplusplus

typedef struct ArchitectureObj ArchitectureObj;

/**
 * The core of the plugin system
 *
 * It scans system directories and collects valid memflow plugins. They can then be instantiated
 * easily. The reason the libraries are collected is to allow for reuse, and save performance
 *
 * # Examples
 *
 * Creating a OS instance, the recommended way:
 *
 * ```
 * use memflow::plugins::Inventory;
 * # use memflow::error::Result;
 * # use memflow::plugins::OsInstance;
 * # fn test() -> Result<OsInstance> {
 * let inventory = Inventory::scan();
 * inventory
 *   .builder()
 *   .connector("qemu_procfs")
 *   .os("win32")
 *   .build()
 * # }
 * # test().ok();
 * ```
 *
 * Nesting connectors and os plugins:
 * ```
 * use memflow::plugins::{Inventory, Args};
 * # use memflow::error::Result;
 * # fn test() -> Result<()> {
 * let inventory = Inventory::scan();
 * let os = inventory
 *   .builder()
 *   .connector("qemu_procfs")
 *   .os("linux")
 *   .connector("qemu_procfs")
 *   .os("win32")
 *   .build();
 * # Ok(())
 * # }
 * # test().ok();
 * ```
 */
typedef struct Inventory Inventory;

typedef struct Option______Library Option______Library;

typedef struct PhysicalReadData PhysicalReadData;

typedef struct PhysicalWriteData PhysicalWriteData;

typedef struct VirtualMemoryObj VirtualMemoryObj;

typedef struct VirtualReadData VirtualReadData;

typedef struct VirtualWriteData VirtualWriteData;

/**
 * This type represents a address on the target system.
 * It internally holds a `u64` value but can also be used
 * when working in 32-bit environments.
 *
 * This type will not handle overflow for 32-bit or 64-bit addresses / lengths.
 */
typedef uint64_t Address;
/**
 * A address with the value of zero.
 *
 * # Examples
 *
 * ```
 * use memflow::types::Address;
 *
 * println!("address: {}", Address::NULL);
 * ```
 */
#define Address_NULL 0

/**
 * Describes the type of a page using a bitflag.
 */
typedef uint8_t PageType;
/**
 * The page explicitly has no flags.
 */
#define PageType_NONE (uint8_t)0
/**
 * The page type is not known.
 */
#define PageType_UNKNOWN (uint8_t)1
/**
 * The page contains page table entries.
 */
#define PageType_PAGE_TABLE (uint8_t)2
/**
 * The page is a writeable page.
 */
#define PageType_WRITEABLE (uint8_t)4
/**
 * The page is read only.
 */
#define PageType_READ_ONLY (uint8_t)8
/**
 * The page is not executable.
 */
#define PageType_NOEXEC (uint8_t)16

/**
 * This type represents a wrapper over a [address](address/index.html)
 * with additional information about the containing page in the physical memory domain.
 *
 * This type will mostly be used by the [virtual to physical address translation](todo.html).
 * When a physical address is translated from a virtual address the additional information
 * about the allocated page the virtual address points to can be obtained from this structure.
 *
 * Most architectures have support multiple page sizes (see [huge pages](todo.html))
 * which will be represented by the containing `page` of the `PhysicalAddress` struct.
 */
typedef struct PhysicalAddress {
    Address address;
    PageType page_type;
    uint8_t page_size_log2;
} PhysicalAddress;

/**
 * Utility typedef for better cbindgen
 *
 * TODO: remove when fixed in cbindgen
 */
typedef void *pvoid;

/**
 * Generic function for cloning past FFI boundary
 */
typedef struct GenericCloneTable_c_void {
    pvoid (*clone)(const void *thisptr);
} GenericCloneTable_c_void;

/**
 * Base table for most objects that are cloneable and droppable.
 */
typedef struct GenericBaseTable_c_void {
    struct GenericCloneTable_c_void clone;
    void (*drop)(void *thisptr);
} GenericBaseTable_c_void;

/**
 * Opaque version of `GenericBaseTable` for FFI purposes
 */
typedef struct GenericBaseTable_c_void OpaqueBaseTable;

typedef struct PhysicalMemoryMetadata {
    uintptr_t size;
    bool readonly;
} PhysicalMemoryMetadata;

typedef struct PhysicalMemoryMapping {
    Address base;
    uintptr_t size;
    Address real_base;
} PhysicalMemoryMapping;

typedef struct PhysicalMemoryFunctionTable_c_void {
    int32_t (*phys_read_raw_list)(void *phys_mem, struct PhysicalReadData *read_data, uintptr_t read_data_count);
    int32_t (*phys_write_raw_list)(void *phys_mem, const struct PhysicalWriteData *write_data, uintptr_t write_data_count);
    struct PhysicalMemoryMetadata (*metadata)(const void *phys_mem);
    void (*set_mem_map)(void *phys_mem, const struct PhysicalMemoryMapping *mem_maps, uintptr_t mem_maps_count);
} PhysicalMemoryFunctionTable_c_void;

typedef struct PhysicalMemoryFunctionTable_c_void OpaquePhysicalMemoryFunctionTable;

typedef struct COptArc_Library {
    const Library *inner;
    struct Option______Library (*clone_fn)(struct Option______Library);
    void (*drop_fn)(struct Option______Library*);
} COptArc_Library;

typedef struct CpuStateFunctionTable_c_void {
    void (*drop)(void *thisptr);
} CpuStateFunctionTable_c_void;

typedef struct CpuStateFunctionTable_c_void OpaqueCpuStateFunctionTable;

typedef struct PluginCpuState {
    void *instance;
    OpaqueCpuStateFunctionTable vtable;
    struct COptArc_Library library;
} PluginCpuState;

typedef struct PluginCpuState MuPluginCpuState;

/**
 * Opaque version of `GenericCloneTable` for FFI purposes
 */
typedef struct GenericCloneTable_c_void OpaqueCloneTable;

typedef struct ArcPluginCpuState {
    struct PluginCpuState inner;
    OpaqueCloneTable clone;
} ArcPluginCpuState;

typedef struct ArcPluginCpuState MuArcPluginCpuState;

typedef struct ConnectorCpuStateFunctionTable_c_void__c_void {
    int32_t (*cpu_state)(void *os, struct COptArc_Library lib, MuPluginCpuState *out);
    int32_t (*into_cpu_state)(void *os, struct COptArc_Library lib, MuArcPluginCpuState *out);
} ConnectorCpuStateFunctionTable_c_void__c_void;

typedef struct ConnectorCpuStateFunctionTable_c_void__c_void OpaqueConnectorCpuStateFunctionTable;

typedef struct ConnectorFunctionTable {
    /**
     * The vtable for object creation and cloning
     */
    const OpaqueBaseTable *base;
    /**
     * The vtable for all physical memory function calls to the connector.
     */
    const OpaquePhysicalMemoryFunctionTable *phys;
    const OpaqueConnectorCpuStateFunctionTable *cpu_state;
} ConnectorFunctionTable;

/**
 * Describes initialized connector instance
 *
 * This structure is returned by `Connector`. It is needed to maintain reference
 * counts to the loaded connector library.
 */
typedef struct ConnectorInstance {
    void *instance;
    struct ConnectorFunctionTable vtable;
    /**
     * Internal library arc.
     *
     * This will keep the library loaded in memory as long as the connector instance is alive.
     * This has to be the last member of the struct so the library will be unloaded _after_
     * the instance is destroyed.
     *
     * If the library is unloaded prior to the instance this will lead to a SIGSEGV.
     */
    struct COptArc_Library library;
} ConnectorInstance;

typedef struct ConnectorInstance MuConnectorInstance;

typedef struct Callback_c_void__Address {
    void *context;
    bool (*func)(void*, Address);
} Callback_c_void__Address;

typedef struct Callback_c_void__Address OpaqueCallback_Address;

typedef OpaqueCallback_Address AddressCallback;

/**
 * Type meant for process IDs
 *
 * If there is a case where Pid can be over 32-bit limit, or negative, please open an issue, we
 * would love to see that.
 */
typedef uint32_t Pid;

typedef int8_t *ReprCString;

typedef enum ArchitectureIdent_Tag {
    /**
     * Unknown architecture. Could be third-party implemented. memflow knows how to work on them,
     * but is unable to instantiate them.
     */
    Unknown,
    /**
     * X86 with specified bitness and address extensions
     *
     * First argument - `bitness` controls whether it's 32, or 64 bit variant.
     * Second argument - `address_extensions` control whether address extensions are
     * enabled (PAE on x32, or LA57 on x64). Warning: LA57 is currently unsupported.
     */
    X86,
    /**
     * Arm 64-bit architecture with specified page size
     *
     * Valid page sizes are 4kb, 16kb, 64kb. Only 4kb is supported at the moment
     */
    AArch64,
} ArchitectureIdent_Tag;

typedef struct X86_Body {
    uint8_t _0;
    bool _1;
} X86_Body;

typedef struct ArchitectureIdent {
    ArchitectureIdent_Tag tag;
    union {
        X86_Body x86;
        struct {
            uintptr_t a_arch64;
        };
    };
} ArchitectureIdent;

/**
 * Process information structure
 *
 * This structure implements basic process information. Architectures are provided both of the
 * system, and of the process.
 */
typedef struct ProcessInfo {
    /**
     * The base address of this process.
     *
     * # Remarks
     *
     * On Windows this will be the address of the [`_EPROCESS`](https://www.nirsoft.net/kernel_struct/vista/EPROCESS.html) structure.
     */
    Address address;
    /**
     * ID of this process.
     */
    Pid pid;
    /**
     * Name of the process.
     */
    ReprCString name;
    /**
     * System architecture of the target system.
     */
    struct ArchitectureIdent sys_arch;
    /**
     * Process architecture
     *
     * # Remarks
     *
     * Specifically on 64-bit systems this could be different
     * to the `sys_arch` in case the process is an emulated 32-bit process.
     *
     * On windows this technique is called [`WOW64`](https://docs.microsoft.com/en-us/windows/win32/winprog64/wow64-implementation-details).
     */
    struct ArchitectureIdent proc_arch;
} ProcessInfo;

typedef struct ProcessInfo MuProcessInfo;

typedef const struct ArchitectureIdent *OptionArchitectureIdent;

/**
 * Pair of address and architecture used for callbacks
 */
typedef struct ModuleAddressInfo {
    Address address;
    struct ArchitectureIdent arch;
} ModuleAddressInfo;

typedef struct Callback_c_void__ModuleAddressInfo {
    void *context;
    bool (*func)(void*, struct ModuleAddressInfo);
} Callback_c_void__ModuleAddressInfo;

typedef struct Callback_c_void__ModuleAddressInfo OpaqueCallback_ModuleAddressInfo;

typedef OpaqueCallback_ModuleAddressInfo ModuleAddressCallback;

/**
 * Module information structure
 */
typedef struct ModuleInfo {
    /**
     * Returns the address of the module header.
     *
     * # Remarks
     *
     * On Windows this will be the address where the [`PEB`](https://docs.microsoft.com/en-us/windows/win32/api/winternl/ns-winternl-peb) entry is stored.
     */
    Address address;
    /**
     * The base address of the parent process.
     *
     * # Remarks
     *
     * This field is analog to the `ProcessInfo::address` field.
     */
    Address parent_process;
    /**
     * The actual base address of this module.
     *
     * # Remarks
     *
     * The base address is contained in the virtual address range of the process
     * this module belongs to.
     */
    Address base;
    /**
     * Size of the module
     */
    uintptr_t size;
    /**
     * Name of the module
     */
    ReprCString name;
    /**
     * Path of the module
     */
    ReprCString path;
    /**
     * Architecture of the module
     *
     * # Remarks
     *
     * Emulated processes often have 2 separate lists of modules, one visible to the emulated
     * context (e.g. all 32-bit modules in a WoW64 process), and the other for all native modules
     * needed to support the process emulation. This should be equal to either
     * `ProcessInfo::proc_arch`, or `ProcessInfo::sys_arch` of the parent process.
     */
    struct ArchitectureIdent arch;
} ModuleInfo;

typedef struct ModuleInfo MuModuleInfo;

typedef Address MuAddress;

/**
 * Import information structure
 */
typedef struct ImportInfo {
    /**
     * Name of the import
     */
    ReprCString name;
    /**
     * Offset of this import from the containing modules base address
     */
    uintptr_t offset;
} ImportInfo;

typedef struct Callback_c_void__ImportInfo {
    void *context;
    bool (*func)(void*, struct ImportInfo);
} Callback_c_void__ImportInfo;

typedef struct Callback_c_void__ImportInfo OpaqueCallback_ImportInfo;

typedef OpaqueCallback_ImportInfo ImportCallback;

/**
 * Export information structure
 */
typedef struct ExportInfo {
    /**
     * Name of the export
     */
    ReprCString name;
    /**
     * Offset of this export from the containing modules base address
     */
    uintptr_t offset;
} ExportInfo;

typedef struct Callback_c_void__ExportInfo {
    void *context;
    bool (*func)(void*, struct ExportInfo);
} Callback_c_void__ExportInfo;

typedef struct Callback_c_void__ExportInfo OpaqueCallback_ExportInfo;

typedef OpaqueCallback_ExportInfo ExportCallback;

/**
 * Section information structure
 */
typedef struct SectionInfo {
    /**
     * Name of the section
     */
    ReprCString name;
    /**
     * Virtual address of this section (essentially module_info.base + virtual_address)
     */
    Address base;
    /**
     * Size of this section
     */
    uintptr_t size;
} SectionInfo;

typedef struct Callback_c_void__SectionInfo {
    void *context;
    bool (*func)(void*, struct SectionInfo);
} Callback_c_void__SectionInfo;

typedef struct Callback_c_void__SectionInfo OpaqueCallback_SectionInfo;

typedef OpaqueCallback_SectionInfo SectionCallback;

typedef struct ProcessFunctionTable_c_void {
    int32_t (*module_address_list_callback)(void *process, OptionArchitectureIdent target_arch, ModuleAddressCallback callback);
    int32_t (*module_by_address)(void *process, Address address, struct ArchitectureIdent architecture, MuModuleInfo *out);
    int32_t (*primary_module_address)(void *process, MuAddress *out);
    int32_t (*module_import_list_callback)(void *process, const struct ModuleInfo *info, ImportCallback callback);
    int32_t (*module_export_list_callback)(void *process, const struct ModuleInfo *info, ExportCallback callback);
    int32_t (*module_section_list_callback)(void *process, const struct ModuleInfo *info, SectionCallback callback);
    const struct ProcessInfo *(*info)(const void *process);
    void *(*virt_mem)(void *process);
    void (*drop)(void *thisptr);
} ProcessFunctionTable_c_void;

typedef struct ProcessFunctionTable_c_void OpaqueProcessFunctionTable;

/**
 * A `Page` holds information about a memory page.
 *
 * More information about paging can be found [here](https://en.wikipedia.org/wiki/Paging).
 */
typedef struct Page {
    /**
     * Contains the page type (see above).
     */
    PageType page_type;
    /**
     * Contains the base address of this page.
     */
    Address page_base;
    /**
     * Contains the size of this page.
     */
    uintptr_t page_size;
} Page;

typedef struct Page MuPage;

typedef struct TranslationChunk {
    Address _0;
    uintptr_t _1;
    struct PhysicalAddress _2;
} TranslationChunk;

typedef struct Callback_c_void__TranslationChunk {
    void *context;
    bool (*func)(void*, struct TranslationChunk);
} Callback_c_void__TranslationChunk;

typedef struct Callback_c_void__TranslationChunk OpaqueCallback_TranslationChunk;

typedef OpaqueCallback_TranslationChunk TranslationMapCallback;

typedef struct PageMapChunk {
    Address _0;
    uintptr_t _1;
} PageMapChunk;

typedef struct Callback_c_void__PageMapChunk {
    void *context;
    bool (*func)(void*, struct PageMapChunk);
} Callback_c_void__PageMapChunk;

typedef struct Callback_c_void__PageMapChunk OpaqueCallback_PageMapChunk;

typedef OpaqueCallback_PageMapChunk PageMapCallback;

typedef struct VirtualMemoryFunctionTable_c_void {
    int32_t (*virt_read_raw_list)(void *virt_mem, struct VirtualReadData *read_data, uintptr_t read_data_count);
    int32_t (*virt_write_raw_list)(void *virt_mem, const struct VirtualWriteData *write_data, uintptr_t write_data_count);
    int32_t (*virt_page_info)(void *virt_mem, Address addr, MuPage *out);
    void (*virt_translation_map_range)(void *virt_mem, Address start, Address end, TranslationMapCallback out);
    void (*virt_page_map_range)(void *virt_mem, uintptr_t gap_size, Address start, Address end, PageMapCallback out);
} VirtualMemoryFunctionTable_c_void;

typedef struct VirtualMemoryFunctionTable_c_void OpaqueVirtualMemoryFunctionTable;

typedef struct VirtualMemoryInstance {
    void *instance;
    const OpaqueVirtualMemoryFunctionTable *vtable;
} VirtualMemoryInstance;

typedef struct PluginProcess {
    void *instance;
    OpaqueProcessFunctionTable vtable;
    struct VirtualMemoryInstance virt_mem;
} PluginProcess;

typedef struct PluginProcess MuPluginProcess;

typedef struct ArcPluginProcess {
    struct PluginProcess inner;
    OpaqueCloneTable clone;
    struct COptArc_Library library;
} ArcPluginProcess;

typedef struct ArcPluginProcess MuArcPluginProcess;

/**
 * Information block about OS
 *
 * This provides some basic information about the OS in question. `base`, and `size` may be
 * omitted in some circumstances (lack of kernel, or privileges). But architecture should always
 * be correct.
 */
typedef struct OsInfo {
    /**
     * Base address of the OS kernel
     */
    Address base;
    /**
     * Size of the OS kernel
     */
    uintptr_t size;
    /**
     * System architecture
     */
    struct ArchitectureIdent arch;
} OsInfo;

typedef struct OsFunctionTable_c_void__c_void {
    int32_t (*process_address_list_callback)(void *os, AddressCallback callback);
    int32_t (*process_info_by_address)(void *os, Address address, MuProcessInfo *out);
    int32_t (*process_by_info)(void *os, struct ProcessInfo info, MuPluginProcess *out);
    int32_t (*into_process_by_info)(void *os, struct ProcessInfo info, struct COptArc_Library lib, MuArcPluginProcess *out);
    int32_t (*module_address_list_callback)(void *os, AddressCallback callback);
    int32_t (*module_by_address)(void *os, Address address, MuModuleInfo *out);
    const struct OsInfo *(*info)(const void *os);
    void *(*phys_mem)(void *os);
    void *(*virt_mem)(void *os);
} OsFunctionTable_c_void__c_void;

typedef struct OsFunctionTable_c_void__c_void OpaqueOsFunctionTable;

typedef struct KeyboardStateFunctionTable_c_void {
    int32_t (*is_down)(const void *keyboard_state, int32_t vk);
    void (*set_down)(void *keyboard_state, int32_t vk, int32_t down);
    void (*drop)(void *thisptr);
} KeyboardStateFunctionTable_c_void;

typedef struct KeyboardStateFunctionTable_c_void OpaqueKeyboardStateFunctionTable;

typedef struct ArcPluginKeyboardState {
    void *instance;
    OpaqueKeyboardStateFunctionTable vtable;
    OpaqueCloneTable clone;
    struct COptArc_Library library;
} ArcPluginKeyboardState;

typedef struct ArcPluginKeyboardState MuArcPluginKeyboardState;

typedef struct KeyboardFunctionTable_c_void {
    int32_t (*state)(void *keyboard, struct COptArc_Library lib, MuArcPluginKeyboardState *out);
    int32_t (*set_state)(void *keyboard, const struct ArcPluginKeyboardState *state);
    void (*drop)(void *thisptr);
} KeyboardFunctionTable_c_void;

typedef struct KeyboardFunctionTable_c_void OpaqueKeyboardFunctionTable;

typedef struct PluginKeyboard {
    void *instance;
    OpaqueKeyboardFunctionTable vtable;
    struct COptArc_Library library;
} PluginKeyboard;

typedef struct PluginKeyboard MuPluginKeyboard;

typedef struct ArcPluginKeyboard {
    struct PluginKeyboard inner;
    OpaqueCloneTable clone;
} ArcPluginKeyboard;

typedef struct ArcPluginKeyboard MuArcPluginKeyboard;

typedef struct OsKeyboardFunctionTable_c_void__c_void {
    int32_t (*keyboard)(void *os, struct COptArc_Library lib, MuPluginKeyboard *out);
    int32_t (*into_keyboard)(void *os, struct COptArc_Library lib, MuArcPluginKeyboard *out);
} OsKeyboardFunctionTable_c_void__c_void;

typedef struct OsKeyboardFunctionTable_c_void__c_void OpaqueOsKeyboardFunctionTable;

typedef struct OsLayerFunctionTable {
    /**
     * The vtable for object creation and cloning
     */
    const OpaqueBaseTable *base;
    /**
     * The vtable for all os functions
     */
    const OpaqueOsFunctionTable *os;
    /**
     * The vtable for the keyboard access if available
     */
    const OpaqueOsKeyboardFunctionTable *keyboard;
} OsLayerFunctionTable;

typedef struct PhysicalMemoryInstance {
    void *instance;
    const OpaquePhysicalMemoryFunctionTable *vtable;
} PhysicalMemoryInstance;

/**
 * Describes a FFI safe option
 */
typedef enum COption_PhysicalMemoryInstance_Tag {
    None_PhysicalMemoryInstance,
    Some_PhysicalMemoryInstance,
} COption_PhysicalMemoryInstance_Tag;

typedef struct COption_PhysicalMemoryInstance {
    COption_PhysicalMemoryInstance_Tag tag;
    union {
        struct {
            struct PhysicalMemoryInstance some;
        };
    };
} COption_PhysicalMemoryInstance;

/**
 * Describes a FFI safe option
 */
typedef enum COption_VirtualMemoryInstance_Tag {
    None_VirtualMemoryInstance,
    Some_VirtualMemoryInstance,
} COption_VirtualMemoryInstance_Tag;

typedef struct COption_VirtualMemoryInstance {
    COption_VirtualMemoryInstance_Tag tag;
    union {
        struct {
            struct VirtualMemoryInstance some;
        };
    };
} COption_VirtualMemoryInstance;

/**
 * Describes initialized os instance
 *
 * This structure is returned by `OS`. It is needed to maintain reference
 * counts to the loaded plugin library.
 */
typedef struct OsInstance {
    void *instance;
    struct OsLayerFunctionTable vtable;
    /**
     * Internal library arc.
     *
     * This will keep the library loaded in memory as long as the os instance is alive.
     * This has to be the last member of the struct so the library will be unloaded _after_
     * the instance is destroyed.
     *
     * If the library is unloaded prior to the instance this will lead to a SIGSEGV.
     */
    struct COptArc_Library library;
    /**
     * Internal physical / virtual memory instances for borrowing
     */
    struct COption_PhysicalMemoryInstance phys_mem;
    struct COption_VirtualMemoryInstance virt_mem;
} OsInstance;

typedef struct OsInstance MuOsInstance;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

extern const struct ArchitectureObj *X86_32;

extern const struct ArchitectureObj *X86_32_PAE;

extern const struct ArchitectureObj *X86_64;

void log_init(int32_t level_num);

/**
 * Helper to convert `Address` to a `PhysicalAddress`
 *
 * This will create a `PhysicalAddress` with `UNKNOWN` PageType.
 */
struct PhysicalAddress addr_to_paddr(Address address);

/**
 * Create a new connector inventory
 *
 * This function will try to find connectors using PATH environment variable
 *
 * Note that all functions go through each directories, and look for a `memflow` directory,
 * and search for libraries in those.
 *
 * # Safety
 *
 * Inventory is inherently unsafe, because it loads shared libraries which can not be
 * guaranteed to be safe.
 */
struct Inventory *inventory_scan(void);

/**
 * Create a new inventory with custom path string
 *
 * # Safety
 *
 * `path` must be a valid null terminated string
 */
struct Inventory *inventory_scan_path(const char *path);

/**
 * Add a directory to an existing inventory
 *
 * # Safety
 *
 * `dir` must be a valid null terminated string
 */
int32_t inventory_add_dir(struct Inventory *inv, const char *dir);

/**
 * Create a connector with given arguments
 *
 * This creates an instance of `ConnectorInstance`.
 *
 * This instance needs to be dropped using `connector_drop`.
 *
 * # Arguments
 *
 * * `name` - name of the connector to use
 * * `args` - arguments to be passed to the connector upon its creation
 *
 * # Safety
 *
 * Both `name`, and `args` must be valid null terminated strings.
 *
 * Any error strings returned by the connector must not be outputed after the connector gets
 * freed, because that operation could cause the underlying shared library to get unloaded.
 */
int32_t inventory_create_connector(struct Inventory *inv,
                                   const char *name,
                                   const char *args,
                                   MuConnectorInstance *out);

/**
 * Create a OS instance with given arguments
 *
 * This creates an instance of `KernelInstance`.
 *
 * This instance needs to be freed using `os_drop`.
 *
 * # Arguments
 *
 * * `name` - name of the OS to use
 * * `args` - arguments to be passed to the connector upon its creation
 * * `mem` - a previously initialized connector instance
 * * `out` - a valid memory location that will contain the resulting os-instance
 *
 * # Remarks
 *
 * The `mem` connector instance is being _moved_ into the os layer.
 * This means upon calling `os_drop` it is not unnecessary to call `connector_drop` anymore.
 *
 * # Safety
 *
 * Both `name`, and `args` must be valid null terminated strings.
 *
 * Any error strings returned by the connector must not be outputed after the connector gets
 * freed, because that operation could cause the underlying shared library to get unloaded.
 */
int32_t inventory_create_os(struct Inventory *inv,
                            const char *name,
                            const char *args,
                            struct ConnectorInstance mem,
                            MuOsInstance *out);

/**
 * Free a os plugin
 *
 * # Safety
 *
 * `os` must point to a valid `OsInstance` that was created using one of the provided
 * functions.
 */
void os_drop(struct OsInstance *os);

/**
 * Clone a connector
 *
 * This method is useful when needing to perform multithreaded operations, as a connector is not
 * guaranteed to be thread safe. Every single cloned instance also needs to be dropped using
 * `connector_drop`.
 *
 * # Safety
 *
 * `conn` has to point to a a valid `CloneablePhysicalMemory` created by one of the provided
 * functions.
 */
void connector_clone(const struct ConnectorInstance *conn, MuConnectorInstance *out);

/**
 * Free a connector instance
 *
 * # Safety
 *
 * `conn` has to point to a valid [`ConnectorInstance`] created by one of the provided
 * functions.
 *
 * There has to be no instance of `PhysicalMemory` created from the input `conn`, because they
 * will become invalid.
 */
void connector_drop(struct ConnectorInstance *conn);

/**
 * Free a connector inventory
 *
 * # Safety
 *
 * `inv` must point to a valid `Inventory` that was created using one of the provided
 * functions.
 */
void inventory_free(struct Inventory *inv);

/**
 * Read a list of values
 *
 * This will perform `len` physical memory reads on the provided `data`. Using lists is preferable
 * for performance, because then the underlying connectors can batch those operations.
 *
 * # Safety
 *
 * `data` must be a valid array of `PhysicalReadData` with the length of at least `len`
 */
int32_t phys_read_raw_list(struct ConnectorInstance *mem,
                           struct PhysicalReadData *data,
                           uintptr_t len);

/**
 * Write a list of values
 *
 * This will perform `len` physical memory writes on the provided `data`. Using lists is preferable
 * for performance, because then the underlying connectors can batch those operations.
 *
 * # Safety
 *
 * `data` must be a valid array of `PhysicalWriteData` with the length of at least `len`
 */
int32_t phys_write_raw_list(struct ConnectorInstance *mem,
                            const struct PhysicalWriteData *data,
                            uintptr_t len);

/**
 * Retrieve metadata about the physical memory object
 */
struct PhysicalMemoryMetadata phys_metadata(const struct ConnectorInstance *mem);

/**
 * Read a single value into `out` from a provided `PhysicalAddress`
 *
 * # Safety
 *
 * `out` must be a valid pointer to a data buffer of at least `len` size.
 */
int32_t phys_read_raw(struct ConnectorInstance *mem,
                      struct PhysicalAddress addr,
                      uint8_t *out,
                      uintptr_t len);

/**
 * Read a single 32-bit value from a provided `PhysicalAddress`
 */
uint32_t phys_read_u32(struct ConnectorInstance *mem, struct PhysicalAddress addr);

/**
 * Read a single 64-bit value from a provided `PhysicalAddress`
 */
uint64_t phys_read_u64(struct ConnectorInstance *mem, struct PhysicalAddress addr);

/**
 * Write a single value from `input` into a provided `PhysicalAddress`
 *
 * # Safety
 *
 * `input` must be a valid pointer to a data buffer of at least `len` size.
 */
int32_t phys_write_raw(struct ConnectorInstance *mem,
                       struct PhysicalAddress addr,
                       const uint8_t *input,
                       uintptr_t len);

/**
 * Write a single 32-bit value into a provided `PhysicalAddress`
 */
int32_t phys_write_u32(struct ConnectorInstance *mem, struct PhysicalAddress addr, uint32_t val);

/**
 * Write a single 64-bit value into a provided `PhysicalAddress`
 */
int32_t phys_write_u64(struct ConnectorInstance *mem, struct PhysicalAddress addr, uint64_t val);

/**
 * Free a virtual memory object reference
 *
 * This function frees the reference to a virtual memory object.
 *
 * # Safety
 *
 * `mem` must be a valid reference to a virtual memory object.
 */
void virt_free(struct VirtualMemoryObj *mem);

/**
 * Read a list of values
 *
 * This will perform `len` virtual memory reads on the provided `data`. Using lists is preferable
 * for performance, because then the underlying connectors can batch those operations, and virtual
 * translation function can cut down on read operations.
 *
 * # Safety
 *
 * `data` must be a valid array of `VirtualReadData` with the length of at least `len`
 */
int32_t virt_read_raw_list(struct VirtualMemoryObj *mem,
                           struct VirtualReadData *data,
                           uintptr_t len);

/**
 * Write a list of values
 *
 * This will perform `len` virtual memory writes on the provided `data`. Using lists is preferable
 * for performance, because then the underlying connectors can batch those operations, and virtual
 * translation function can cut down on read operations.
 *
 * # Safety
 *
 * `data` must be a valid array of `VirtualWriteData` with the length of at least `len`
 */
int32_t virt_write_raw_list(struct VirtualMemoryObj *mem,
                            const struct VirtualWriteData *data,
                            uintptr_t len);

/**
 * Read a single value into `out` from a provided `Address`
 *
 * # Safety
 *
 * `out` must be a valid pointer to a data buffer of at least `len` size.
 */
int32_t virt_read_raw_into(struct VirtualMemoryObj *mem, Address addr, uint8_t *out, uintptr_t len);

/**
 * Read a single 32-bit value from a provided `Address`
 */
uint32_t virt_read_u32(struct VirtualMemoryObj *mem, Address addr);

/**
 * Read a single 64-bit value from a provided `Address`
 */
uint64_t virt_read_u64(struct VirtualMemoryObj *mem, Address addr);

/**
 * Write a single value from `input` into a provided `Address`
 *
 * # Safety
 *
 * `input` must be a valid pointer to a data buffer of at least `len` size.
 */
int32_t virt_write_raw(struct VirtualMemoryObj *mem,
                       Address addr,
                       const uint8_t *input,
                       uintptr_t len);

/**
 * Write a single 32-bit value into a provided `Address`
 */
int32_t virt_write_u32(struct VirtualMemoryObj *mem, Address addr, uint32_t val);

/**
 * Write a single 64-bit value into a provided `Address`
 */
int32_t virt_write_u64(struct VirtualMemoryObj *mem, Address addr, uint64_t val);

/**
 * Returns a reference to the [`PhysicalMemory`] object this OS uses.
 * The [`PhysicalMemory`] usually is just the Connector this OS was intitialized with.
 *
 * If no connector is used `null` is returned.
 *
 * # Safety
 *
 * `os` must point to a valid `OsInstance` that was created using one of the provided
 * functions.
 */
struct PhysicalMemoryInstance *os_phys_mem(struct OsInstance *os);

/**
 * Returns a reference to the [`VirtualMemory`] object this OS uses.
 *
 * If no [`VirtualMemory`] object is used `null` is returned.
 *
 * # Safety
 *
 * `os` must point to a valid `OsInstance` that was created using one of the provided
 * functions.
 */
struct VirtualMemoryInstance *os_virt_mem(struct OsInstance *os);

uint8_t arch_bits(const struct ArchitectureObj *arch);

Endianess arch_endianess(const struct ArchitectureObj *arch);

uintptr_t arch_page_size(const struct ArchitectureObj *arch);

uintptr_t arch_size_addr(const struct ArchitectureObj *arch);

uint8_t arch_address_space_bits(const struct ArchitectureObj *arch);

/**
 * Free an architecture reference
 *
 * # Safety
 *
 * `arch` must be a valid heap allocated reference created by one of the API's functions.
 */
void arch_free(struct ArchitectureObj *arch);

bool is_x86_arch(const struct ArchitectureObj *arch);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* MEMFLOW_H */
