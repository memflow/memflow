#ifndef MEMFLOW_H
#define MEMFLOW_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
typedef void *Arc_Library;

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

typedef struct Inventory Inventory;

typedef struct PhysicalReadData PhysicalReadData;

typedef struct PhysicalWriteData PhysicalWriteData;

typedef struct VirtualMemoryObj VirtualMemoryObj;

typedef struct VirtualReadData VirtualReadData;

typedef struct VirtualWriteData VirtualWriteData;

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
     * ARM 64-bit architecture with specified page size
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

typedef void *pvoid;

typedef struct GenericBaseTable_c_void__OptionMut_c_void {
    pvoid (*create)(const char *args, pvoid obj, int32_t log_level);
    pvoid (*clone)(const void *this);
    void (*drop)(void *this);
} GenericBaseTable_c_void__OptionMut_c_void;

typedef struct GenericBaseTable_c_void__OptionMut_c_void OpaqueBaseTable_OptionMut_c_void;

typedef OpaqueBaseTable_OptionMut_c_void ConnectorBaseTable;

typedef struct PhysicalMemoryMetadata {
    uintptr_t size;
    bool readonly;
} PhysicalMemoryMetadata;

typedef struct PhysicalMemoryFunctionTable_c_void {
    int32_t (*phys_read_raw_list)(void *phys_mem, struct PhysicalReadData *read_data, uintptr_t read_data_count);
    int32_t (*phys_write_raw_list)(void *phys_mem, const struct PhysicalWriteData *write_data, uintptr_t write_data_count);
    struct PhysicalMemoryMetadata (*metadata)(const void *phys_mem);
} PhysicalMemoryFunctionTable_c_void;

typedef struct PhysicalMemoryFunctionTable_c_void OpaquePhysicalMemoryFunctionTable;

typedef struct ConnectorFunctionTable {
    /**
     * The vtable for object creation and cloning
     */
    ConnectorBaseTable base;
    /**
     * The vtable for all physical memory function calls to the connector.
     */
    OpaquePhysicalMemoryFunctionTable phys;
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
    Arc_Library library;
} ConnectorInstance;

typedef OpaqueBaseTable_OptionMut_c_void OSBaseTable;

typedef struct Callback_c_void__c_void__Address {
    void *context;
    bool (*func)(void*, void*, Address);
} Callback_c_void__c_void__Address;

typedef struct Callback_c_void__c_void__Address OpaqueCallback_c_void__Address;

typedef OpaqueCallback_c_void__Address AddressCallback_c_void;

typedef uint32_t PID;

typedef int8_t *ReprCStr;

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
    PID pid;
    /**
     * Name of the process.
     */
    ReprCStr name;
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

typedef struct PluginProcessRef {
    void *instance;
} PluginProcessRef;

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
    ReprCStr name;
    /**
     * Path of the module
     */
    ReprCStr path;
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

typedef struct KernelInfo {
    /**
     * Base address of the kernel
     */
    Address base;
    /**
     * Size of the kernel
     */
    uintptr_t size;
    /**
     * System architecture
     */
    struct ArchitectureIdent arch;
} KernelInfo;

typedef struct KernelFunctionTable_c_void {
    int32_t (*process_address_list_callback)(void *kernel, AddressCallback_c_void callback);
    int32_t (*process_info_by_address)(void *kernel, Address address, struct ProcessInfo *out);
    int32_t (*process_by_info)(void *kernel, struct ProcessInfo info, struct PluginProcessRef *out);
    int32_t (*module_address_list_callback)(void *kernel, AddressCallback_c_void callback);
    int32_t (*module_by_address)(void *kernel, Address address, struct ModuleInfo *out);
    const struct KernelInfo *(*info)(const void *kernel);
} KernelFunctionTable_c_void;

typedef struct KernelFunctionTable_c_void OpaqueKernelFunctionTable;

typedef struct VirtualMemoryFunctionTable {
    int32_t (*virt_read_raw_list)(void *virt_mem, struct VirtualReadData *read_data, uintptr_t read_data_count);
    int32_t (*virt_write_raw_list)(void *virt_mem, const struct VirtualWriteData *write_data, uintptr_t write_data_count);
} VirtualMemoryFunctionTable;

typedef struct OSLayerFunctionTable {
    /**
     * The vtable for object creation and cloning
     */
    OSBaseTable base;
    /**
     * The vtable for all kernel functions
     */
    OpaqueKernelFunctionTable kernel;
    /**
     * The vtable for all physical memory access if available
     */
    const OpaquePhysicalMemoryFunctionTable *phys;
    /**
     * The vtable for all virtual memory access if available
     */
    const struct VirtualMemoryFunctionTable *virt;
} OSLayerFunctionTable;

/**
 * Describes initialized kernel instance
 *
 * This structure is returned by `Kernel`. It is needed to maintain reference
 * counts to the loaded connector library.
 */
typedef struct KernelInstance {
    void *instance;
    struct OSLayerFunctionTable vtable;
    /**
     * Internal library arc.
     *
     * This will keep the library loaded in memory as long as the kernel instance is alive.
     * This has to be the last member of the struct so the library will be unloaded _after_
     * the instance is destroyed.
     *
     * If the library is unloaded prior to the instance this will lead to a SIGSEGV.
     */
    Arc_Library library;
} KernelInstance;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

extern const struct ArchitectureObj *X86_32;

extern const struct ArchitectureObj *X86_32_PAE;

extern const struct ArchitectureObj *X86_64;

void gone(const struct ArchitectureIdent *_arch);

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
 * This instance needs to be freed using `connector_free`.
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
struct ConnectorInstance *inventory_create_connector(struct Inventory *inv,
                                                     const char *name,
                                                     const char *args);

/**
 * Create a OS instance with given arguments
 *
 * This creates an instance of `KernelInstance`.
 *
 * This instance needs to be freed using `os_free`.
 *
 * # Arguments
 *
 * * `name` - name of the OS to use
 * * `args` - arguments to be passed to the connector upon its creation
 *
 * # Safety
 *
 * Both `name`, and `args` must be valid null terminated strings.
 *
 * Any error strings returned by the connector must not be outputed after the connector gets
 * freed, because that operation could cause the underlying shared library to get unloaded.
 */
struct KernelInstance *inventory_create_os(struct Inventory *inv,
                                           const char *name,
                                           const char *args);

/**
 * Clone a connector
 *
 * This method is useful when needing to perform multithreaded operations, as a connector is not
 * guaranteed to be thread safe. Every single cloned instance also needs to be freed using
 * `connector_free`.
 *
 * # Safety
 *
 * `conn` has to point to a a valid `CloneablePhysicalMemory` created by one of the provided
 * functions.
 */
struct ConnectorInstance *connector_clone(const struct ConnectorInstance *conn);

/**
 * Free a connector instance
 *
 * # Safety
 *
 * `conn` has to point to a valid `ConnectorInstance` created by one of the provided
 * functions.
 *
 * There has to be no instance of `PhysicalMemory` created from the input `conn`, because they
 * will become invalid.
 */
void connector_free(struct ConnectorInstance *conn);

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

uint8_t arch_bits(const struct ArchitectureObj *arch);

Endianess arch_endianess(const struct ArchitectureObj *arch);

uintptr_t page_size(const struct ArchitectureObj *arch);

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
