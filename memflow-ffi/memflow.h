#ifndef MEMFLOW_H
#define MEMFLOW_H

// Construct a typed slice for rust functions
#define REF_SLICE(ty, buf, len) ((struct CSliceRef_##ty){(buf), (len)})

// Constructs a typed mutable slice for rust functions
#define MUT_SLICE(ty, buf, len) ((struct CSliceMut_##ty){(buf), (len)})

// Constructs a slice from a string for rust functions
// Note that strlen() is optimized out for string literals here
#define STR(string) \
    REF_SLICE(u8, (const unsigned char *)string, strlen(string))

// Constructs a callback
#define CALLBACK(ty, ctx, func) \
    (struct Callback_c_void__##ty){(ctx), (bool (*)(void *, ty))(func)}

// Constructs a dynamic collect callback
//
// This callback will collect all elements into a buffer accessible within `(*name_data)`.
// It is the same buffer as `name_base.buf`, but cast into the correct type. The buffer must
// be freed with `free(3)`.
//
// Number of elements is accessible within `name_base.size`, alongside its capacity.
//
// After creation, this callback should not exit its scope.
#define COLLECT_CB(ty, name) \
    struct CollectBase name##_base = {}; \
    ty **name##_data = (ty **)&name##_base.buf; \
    Callback_c_void__##ty name = CALLBACK(ty, &name##_base, cb_collect_dynamic_##ty)

// Constructs a static collect callback
//
// This callback will collect all elements into the provided buffer up to given length.
//
// Any additional elements that do not fit will be skipped.
//
// Number of elements is accessible within `name_base.size`.
//
// After creation, this callback should not exit its scope.
#define COLLECT_CB_INTO(ty, name, data, len) \
    struct CollectBase name##_base = (struct CollectBase){ (void *)data, (size_t)len, 0 }; \
    ty **name##_data = (ty **)&name##_base.buf; \
    Callback_c_void__##ty name = CALLBACK(ty, &name##_base, cb_collect_static_##ty)

// Constructs a static collect callback (for arrays)
//
// This is the same as `COLLECT_CB_INTO`, but performs an automatic array size calculation.
//
// Number of elements is accessible within `name_base.size`.
//
// After creation, this callback should not exit its scope.
#define COLLECT_CB_INTO_ARR(ty, name, data) \
    COLLECT_CB_INTO(ty, name, data, sizeof(data) / sizeof(*data))

// Constructs a count callback
//
// This callback will simply count the number of elements encountered, and this value is
// accessible through `name_count` variable.
//
// After creation, this callback should not exit its scope.
#define COUNT_CB(ty, name) \
    size_t name##_count = 0; \
    Callback_c_void__##ty name = CALLBACK(ty, &name##_count, cb_count_##ty)

#define BUF_ITER_SPEC(ty, ty2, name, buf, len) \
    struct BufferIterator name##_base = (struct BufferIterator){(const void *)(const ty2 *)buf, len, 0, sizeof(ty2)}; \
    CIterator_##ty name = (CIterator_##ty){ &name##_base, (int32_t (*)(void *, ty2 *))buf_iter_next }

#define BUF_ITER_ARR_SPEC(ty, ty2, name, buf) BUF_ITER_SPEC(ty, ty2, name, buf, sizeof(buf) / sizeof(*buf))

#define BUF_ITER(ty, name, buf, len) \
    BUF_ITER_SPEC(ty, ty, name, buf, len)

#define BUF_ITER_ARR(ty, name, buf) BUF_ITER(ty, name, buf, sizeof(buf) / sizeof(*buf))

// Forward declarations for vtables and their wrappers
struct CGlueTraitObj_CBox_c_void_____CpuStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void______________CArc_c_void_____CpuStateRetTmp_CArc_c_void;
struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void;
struct CGlueTraitObj_CBox_c_void_____KeyboardStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardStateRetTmp_CArc_c_void______________CArc_c_void_____KeyboardStateRetTmp_CArc_c_void;
struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardStateRetTmp_CArc_c_void;
struct CGlueTraitObj_CBox_c_void_____KeyboardVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void______________CArc_c_void_____KeyboardRetTmp_CArc_c_void;
struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void;
struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void;
struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void;
struct ConnectorInstance_CBox_c_void_____CArc_c_void;
struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void;
struct IntoCpuState_CBox_c_void_____CArc_c_void;
struct IntoCpuStateContainer_CBox_c_void_____CArc_c_void;
struct IntoCpuState_CBox_c_void_____CArc_c_void;
struct IntoCpuStateContainer_CBox_c_void_____CArc_c_void;
struct ConnectorInstance_CBox_c_void_____CArc_c_void;
struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void;
struct OsInstance_CBox_c_void_____CArc_c_void;
struct OsInstanceContainer_CBox_c_void_____CArc_c_void;
struct OsInstance_CBox_c_void_____CArc_c_void;
struct OsInstanceContainer_CBox_c_void_____CArc_c_void;
struct OsInstance_CBox_c_void_____CArc_c_void;
struct OsInstanceContainer_CBox_c_void_____CArc_c_void;
struct IntoKeyboard_CBox_c_void_____CArc_c_void;
struct IntoKeyboardContainer_CBox_c_void_____CArc_c_void;
struct IntoKeyboard_CBox_c_void_____CArc_c_void;
struct IntoKeyboardContainer_CBox_c_void_____CArc_c_void;
struct OsInstance_CBox_c_void_____CArc_c_void;
struct OsInstanceContainer_CBox_c_void_____CArc_c_void;
struct OsInstance_CBox_c_void_____CArc_c_void;
struct OsInstanceContainer_CBox_c_void_____CArc_c_void;
struct OsInstance_CBox_c_void_____CArc_c_void;
struct OsInstanceContainer_CBox_c_void_____CArc_c_void;
struct ProcessInstance_CBox_c_void_____CArc_c_void;
struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void;
struct ProcessInstance_CBox_c_void_____CArc_c_void;
struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void;
struct ProcessInstance_CBox_c_void_____CArc_c_void;
struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void;
struct IntoProcessInstance_CBox_c_void_____CArc_c_void;
struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void;
struct IntoProcessInstance_CBox_c_void_____CArc_c_void;
struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void;
struct IntoProcessInstance_CBox_c_void_____CArc_c_void;
struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void;
struct IntoProcessInstance_CBox_c_void_____CArc_c_void;
struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void;
struct ConnectorInstance_CBox_c_void_____CArc_c_void;
struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void;

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
    Endianess_LittleEndian,
    /**
     * Big Endianess
     */
    Endianess_BigEndian,
};
#ifndef __cplusplus
typedef uint8_t Endianess;
#endif // __cplusplus

/**
 * An enum representing the available verbosity levels of the logger.
 *
 * Typical usage includes: checking if a certain `Level` is enabled with
 * [`log_enabled!`](macro.log_enabled.html), specifying the `Level` of
 * [`log!`](macro.log.html), and comparing a `Level` directly to a
 * [`LevelFilter`](enum.LevelFilter.html).
 */
enum Level
#ifdef __cplusplus
  : uintptr_t
#endif // __cplusplus
 {
    /**
     * The "error" level.
     *
     * Designates very serious errors.
     */
    Level_Error = 1,
    /**
     * The "warn" level.
     *
     * Designates hazardous situations.
     */
    Level_Warn,
    /**
     * The "info" level.
     *
     * Designates useful information.
     */
    Level_Info,
    /**
     * The "debug" level.
     *
     * Designates lower priority information.
     */
    Level_Debug,
    /**
     * The "trace" level.
     *
     * Designates very low priority, often extremely verbose, information.
     */
    Level_Trace,
};
#ifndef __cplusplus
typedef uintptr_t Level;
#endif // __cplusplus

/**
 * An enum representing the available verbosity level filters of the logger.
 *
 * A `LevelFilter` may be compared directly to a [`Level`]. Use this type
 * to get and set the maximum log level with [`max_level()`] and [`set_max_level`].
 *
 * [`Level`]: enum.Level.html
 * [`max_level()`]: fn.max_level.html
 * [`set_max_level`]: fn.set_max_level.html
 */
enum LevelFilter
#ifdef __cplusplus
  : uintptr_t
#endif // __cplusplus
 {
    /**
     * A level lower than all log levels.
     */
    LevelFilter_Off,
    /**
     * Corresponds to the `Error` log level.
     */
    LevelFilter_Error,
    /**
     * Corresponds to the `Warn` log level.
     */
    LevelFilter_Warn,
    /**
     * Corresponds to the `Info` log level.
     */
    LevelFilter_Info,
    /**
     * Corresponds to the `Debug` log level.
     */
    LevelFilter_Debug,
    /**
     * Corresponds to the `Trace` log level.
     */
    LevelFilter_Trace,
};
#ifndef __cplusplus
typedef uintptr_t LevelFilter;
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
 * ```no_run
 * use memflow::plugins::Inventory;
 * # use memflow::plugins::OsInstanceArcBox;
 * # use memflow::error::Result;
 * # fn test() -> Result<OsInstanceArcBox<'static>> {
 * let inventory = Inventory::scan();
 * inventory
 *   .builder()
 *   .connector("qemu")
 *   .os("win32")
 *   .build()
 * # }
 * # test().ok();
 * ```
 *
 * Nesting connectors and os plugins:
 * ```no_run
 * use memflow::plugins::{Inventory, Args};
 * # use memflow::error::Result;
 * # fn test() -> Result<()> {
 * let inventory = Inventory::scan();
 * let os = inventory
 *   .builder()
 *   .connector("qemu")
 *   .os("linux")
 *   .connector("qemu")
 *   .os("win32")
 *   .build();
 * # Ok(())
 * # }
 * # test().ok();
 * ```
 */
typedef struct Inventory Inventory;

/**
 * The largest target memory type
 * The following core rule is defined for these memory types:
 *
 * `PAGE_SIZE < usize <= umem`
 *
 * Where `PAGE_SIZE` is any lowest granularity page size, `usize` is the standard size type, and
 * `umem` is memflow's memory size type.
 *
 * This means that `usize` can always be safely cast to `umem`, while anything to do with page
 * sizes can be cast to `umem` safely,
 *
 */
typedef uint64_t umem;

/**
 * This type represents a address on the target system.
 * It internally holds a `umem` value but can also be used
 * when working in 32-bit environments.
 *
 * This type will not handle overflow for 32-bit or 64-bit addresses / lengths.
 */
typedef umem Address;
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
 * A address with an invalid value.
 *
 * # Examples
 *
 * ```
 * use memflow::types::Address;
 *
 * println!("address: {}", Address::INVALID);
 * ```
 */
#define Address_INVALID ~0

/**
 * Describes the type of a page using a bitflag.
 */
typedef uint8_t PageType;
/**
 * The page explicitly has no flags.
 */
#define PageType_NONE 0
/**
 * The page type is not known.
 */
#define PageType_UNKNOWN 1
/**
 * The page contains page table entries.
 */
#define PageType_PAGE_TABLE 2
/**
 * The page is a writeable page.
 */
#define PageType_WRITEABLE 4
/**
 * The page is read only.
 */
#define PageType_READ_ONLY 8
/**
 * The page is not executable.
 */
#define PageType_NOEXEC 16

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
 * A physical address with an invalid value.
 */
#define PhysicalAddress_INVALID (PhysicalAddress){ .address = Address_INVALID, .page_type = PageType_UNKNOWN, .page_size_log2 = 0 }

/**
 * FFI-safe box
 *
 * This box has a static self reference, alongside a custom drop function.
 *
 * The drop function can be called from anywhere, it will free on correct allocator internally.
 */
typedef struct CBox_c_void {
    void *instance;
    void (*drop_fn)(void*);
} CBox_c_void;

/**
 * FFI-Safe Arc
 *
 * This is an FFI-Safe equivalent of Arc<T> and Option<Arc<T>>.
 */
typedef struct CArc_c_void {
    const void *instance;
    const void *(*clone_fn)(const void*);
    void (*drop_fn)(const void*);
} CArc_c_void;

typedef struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void {
    struct CBox_c_void instance;
    struct CArc_c_void context;
} ConnectorInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait Clone.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct CloneVtbl_ConnectorInstanceContainer_CBox_c_void_____CArc_c_void {
    struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void (*clone)(const struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void *cont);
} CloneVtbl_ConnectorInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * Wrapper around mutable slices.
 *
 * This is meant as a safe type to pass across the FFI boundary with similar semantics as regular
 * slice. However, not all functionality is present, use the slice conversion functions.
 */
typedef struct CSliceMut_u8 {
    uint8_t *data;
    uintptr_t len;
} CSliceMut_u8;

/**
 * FFI-safe 3 element tuple.
 */
typedef struct CTup3_PhysicalAddress__Address__CSliceMut_u8 {
    struct PhysicalAddress _0;
    Address _1;
    struct CSliceMut_u8 _2;
} CTup3_PhysicalAddress__Address__CSliceMut_u8;

/**
 * MemData type for physical memory reads.
 */
typedef struct CTup3_PhysicalAddress__Address__CSliceMut_u8 PhysicalReadData;

/**
 * FFI compatible iterator.
 *
 * Any mutable reference to an iterator can be converted to a `CIterator`.
 *
 * `CIterator<T>` implements `Iterator<Item = T>`.
 *
 * # Examples
 *
 * Using [`AsCIterator`](AsCIterator) helper:
 *
 * ```
 * use cglue::iter::{CIterator, AsCIterator};
 *
 * extern "C" fn sum_all(iter: CIterator<usize>) -> usize {
 *     iter.sum()
 * }
 *
 * let mut iter = (0..10).map(|v| v * v);
 *
 * assert_eq!(sum_all(iter.as_citer()), 285);
 * ```
 *
 * Converting with `Into` trait:
 *
 * ```
 * use cglue::iter::{CIterator, AsCIterator};
 *
 * extern "C" fn sum_all(iter: CIterator<usize>) -> usize {
 *     iter.sum()
 * }
 *
 * let mut iter = (0..=10).map(|v| v * v);
 *
 * assert_eq!(sum_all((&mut iter).into()), 385);
 * ```
 */
typedef struct CIterator_PhysicalReadData {
    void *iter;
    int32_t (*func)(void*, PhysicalReadData *out);
} CIterator_PhysicalReadData;

/**
 * FFI-safe 2 element tuple.
 */
typedef struct CTup2_Address__CSliceMut_u8 {
    Address _0;
    struct CSliceMut_u8 _1;
} CTup2_Address__CSliceMut_u8;

typedef struct CTup2_Address__CSliceMut_u8 ReadData;

typedef struct Callback_c_void__ReadData {
    void *context;
    bool (*func)(void*, ReadData);
} Callback_c_void__ReadData;

typedef struct Callback_c_void__ReadData OpaqueCallback_ReadData;

/**
 * Data needed to perform memory operations.
 *
 * `inp` is an iterator containing
 */
typedef struct MemOps_PhysicalReadData__ReadData {
    struct CIterator_PhysicalReadData inp;
    OpaqueCallback_ReadData *out;
    OpaqueCallback_ReadData *out_fail;
} MemOps_PhysicalReadData__ReadData;

typedef struct MemOps_PhysicalReadData__ReadData PhysicalReadMemOps;

/**
 * Wrapper around const slices.
 *
 * This is meant as a safe type to pass across the FFI boundary with similar semantics as regular
 * slice. However, not all functionality is present, use the slice conversion functions.
 *
 * # Examples
 *
 * Simple conversion:
 *
 * ```
 * use cglue::slice::CSliceRef;
 *
 * let arr = [0, 5, 3, 2];
 *
 * let cslice = CSliceRef::from(&arr[..]);
 *
 * let slice = cslice.as_slice();
 *
 * assert_eq!(&arr, slice);
 * ```
 */
typedef struct CSliceRef_u8 {
    const uint8_t *data;
    uintptr_t len;
} CSliceRef_u8;

/**
 * FFI-safe 3 element tuple.
 */
typedef struct CTup3_PhysicalAddress__Address__CSliceRef_u8 {
    struct PhysicalAddress _0;
    Address _1;
    struct CSliceRef_u8 _2;
} CTup3_PhysicalAddress__Address__CSliceRef_u8;

/**
 * MemData type for physical memory writes.
 */
typedef struct CTup3_PhysicalAddress__Address__CSliceRef_u8 PhysicalWriteData;

/**
 * FFI compatible iterator.
 *
 * Any mutable reference to an iterator can be converted to a `CIterator`.
 *
 * `CIterator<T>` implements `Iterator<Item = T>`.
 *
 * # Examples
 *
 * Using [`AsCIterator`](AsCIterator) helper:
 *
 * ```
 * use cglue::iter::{CIterator, AsCIterator};
 *
 * extern "C" fn sum_all(iter: CIterator<usize>) -> usize {
 *     iter.sum()
 * }
 *
 * let mut iter = (0..10).map(|v| v * v);
 *
 * assert_eq!(sum_all(iter.as_citer()), 285);
 * ```
 *
 * Converting with `Into` trait:
 *
 * ```
 * use cglue::iter::{CIterator, AsCIterator};
 *
 * extern "C" fn sum_all(iter: CIterator<usize>) -> usize {
 *     iter.sum()
 * }
 *
 * let mut iter = (0..=10).map(|v| v * v);
 *
 * assert_eq!(sum_all((&mut iter).into()), 385);
 * ```
 */
typedef struct CIterator_PhysicalWriteData {
    void *iter;
    int32_t (*func)(void*, PhysicalWriteData *out);
} CIterator_PhysicalWriteData;

/**
 * FFI-safe 2 element tuple.
 */
typedef struct CTup2_Address__CSliceRef_u8 {
    Address _0;
    struct CSliceRef_u8 _1;
} CTup2_Address__CSliceRef_u8;

typedef struct CTup2_Address__CSliceRef_u8 WriteData;

typedef struct Callback_c_void__WriteData {
    void *context;
    bool (*func)(void*, WriteData);
} Callback_c_void__WriteData;

typedef struct Callback_c_void__WriteData OpaqueCallback_WriteData;

/**
 * Data needed to perform memory operations.
 *
 * `inp` is an iterator containing
 */
typedef struct MemOps_PhysicalWriteData__WriteData {
    struct CIterator_PhysicalWriteData inp;
    OpaqueCallback_WriteData *out;
    OpaqueCallback_WriteData *out_fail;
} MemOps_PhysicalWriteData__WriteData;

typedef struct MemOps_PhysicalWriteData__WriteData PhysicalWriteMemOps;

typedef struct PhysicalMemoryMetadata {
    Address max_address;
    umem real_size;
    bool readonly;
    uint32_t ideal_batch_size;
} PhysicalMemoryMetadata;

typedef struct PhysicalMemoryMapping {
    Address base;
    umem size;
    Address real_base;
} PhysicalMemoryMapping;

/**
 * Wrapper around const slices.
 *
 * This is meant as a safe type to pass across the FFI boundary with similar semantics as regular
 * slice. However, not all functionality is present, use the slice conversion functions.
 *
 * # Examples
 *
 * Simple conversion:
 *
 * ```
 * use cglue::slice::CSliceRef;
 *
 * let arr = [0, 5, 3, 2];
 *
 * let cslice = CSliceRef::from(&arr[..]);
 *
 * let slice = cslice.as_slice();
 *
 * assert_eq!(&arr, slice);
 * ```
 */
typedef struct CSliceRef_PhysicalMemoryMapping {
    const struct PhysicalMemoryMapping *data;
    uintptr_t len;
} CSliceRef_PhysicalMemoryMapping;

/**
 * FFI-safe 3 element tuple.
 */
typedef struct CTup3_Address__Address__CSliceMut_u8 {
    Address _0;
    Address _1;
    struct CSliceMut_u8 _2;
} CTup3_Address__Address__CSliceMut_u8;

/**
 * MemData type for regular memory reads.
 */
typedef struct CTup3_Address__Address__CSliceMut_u8 ReadDataRaw;

/**
 * FFI compatible iterator.
 *
 * Any mutable reference to an iterator can be converted to a `CIterator`.
 *
 * `CIterator<T>` implements `Iterator<Item = T>`.
 *
 * # Examples
 *
 * Using [`AsCIterator`](AsCIterator) helper:
 *
 * ```
 * use cglue::iter::{CIterator, AsCIterator};
 *
 * extern "C" fn sum_all(iter: CIterator<usize>) -> usize {
 *     iter.sum()
 * }
 *
 * let mut iter = (0..10).map(|v| v * v);
 *
 * assert_eq!(sum_all(iter.as_citer()), 285);
 * ```
 *
 * Converting with `Into` trait:
 *
 * ```
 * use cglue::iter::{CIterator, AsCIterator};
 *
 * extern "C" fn sum_all(iter: CIterator<usize>) -> usize {
 *     iter.sum()
 * }
 *
 * let mut iter = (0..=10).map(|v| v * v);
 *
 * assert_eq!(sum_all((&mut iter).into()), 385);
 * ```
 */
typedef struct CIterator_ReadDataRaw {
    void *iter;
    int32_t (*func)(void*, ReadDataRaw *out);
} CIterator_ReadDataRaw;

/**
 * Data needed to perform memory operations.
 *
 * `inp` is an iterator containing
 */
typedef struct MemOps_ReadDataRaw__ReadData {
    struct CIterator_ReadDataRaw inp;
    OpaqueCallback_ReadData *out;
    OpaqueCallback_ReadData *out_fail;
} MemOps_ReadDataRaw__ReadData;

typedef struct MemOps_ReadDataRaw__ReadData ReadRawMemOps;

/**
 * FFI-safe 3 element tuple.
 */
typedef struct CTup3_Address__Address__CSliceRef_u8 {
    Address _0;
    Address _1;
    struct CSliceRef_u8 _2;
} CTup3_Address__Address__CSliceRef_u8;

/**
 * MemData type for regular memory writes.
 */
typedef struct CTup3_Address__Address__CSliceRef_u8 WriteDataRaw;

/**
 * FFI compatible iterator.
 *
 * Any mutable reference to an iterator can be converted to a `CIterator`.
 *
 * `CIterator<T>` implements `Iterator<Item = T>`.
 *
 * # Examples
 *
 * Using [`AsCIterator`](AsCIterator) helper:
 *
 * ```
 * use cglue::iter::{CIterator, AsCIterator};
 *
 * extern "C" fn sum_all(iter: CIterator<usize>) -> usize {
 *     iter.sum()
 * }
 *
 * let mut iter = (0..10).map(|v| v * v);
 *
 * assert_eq!(sum_all(iter.as_citer()), 285);
 * ```
 *
 * Converting with `Into` trait:
 *
 * ```
 * use cglue::iter::{CIterator, AsCIterator};
 *
 * extern "C" fn sum_all(iter: CIterator<usize>) -> usize {
 *     iter.sum()
 * }
 *
 * let mut iter = (0..=10).map(|v| v * v);
 *
 * assert_eq!(sum_all((&mut iter).into()), 385);
 * ```
 */
typedef struct CIterator_WriteDataRaw {
    void *iter;
    int32_t (*func)(void*, WriteDataRaw *out);
} CIterator_WriteDataRaw;

/**
 * Data needed to perform memory operations.
 *
 * `inp` is an iterator containing
 */
typedef struct MemOps_WriteDataRaw__WriteData {
    struct CIterator_WriteDataRaw inp;
    OpaqueCallback_WriteData *out;
    OpaqueCallback_WriteData *out_fail;
} MemOps_WriteDataRaw__WriteData;

typedef struct MemOps_WriteDataRaw__WriteData WriteRawMemOps;

typedef struct MemoryViewMetadata {
    Address max_address;
    umem real_size;
    bool readonly;
    bool little_endian;
    uint8_t arch_bits;
} MemoryViewMetadata;

/**
 * FFI compatible iterator.
 *
 * Any mutable reference to an iterator can be converted to a `CIterator`.
 *
 * `CIterator<T>` implements `Iterator<Item = T>`.
 *
 * # Examples
 *
 * Using [`AsCIterator`](AsCIterator) helper:
 *
 * ```
 * use cglue::iter::{CIterator, AsCIterator};
 *
 * extern "C" fn sum_all(iter: CIterator<usize>) -> usize {
 *     iter.sum()
 * }
 *
 * let mut iter = (0..10).map(|v| v * v);
 *
 * assert_eq!(sum_all(iter.as_citer()), 285);
 * ```
 *
 * Converting with `Into` trait:
 *
 * ```
 * use cglue::iter::{CIterator, AsCIterator};
 *
 * extern "C" fn sum_all(iter: CIterator<usize>) -> usize {
 *     iter.sum()
 * }
 *
 * let mut iter = (0..=10).map(|v| v * v);
 *
 * assert_eq!(sum_all((&mut iter).into()), 385);
 * ```
 */
typedef struct CIterator_ReadData {
    void *iter;
    int32_t (*func)(void*, ReadData *out);
} CIterator_ReadData;

typedef OpaqueCallback_ReadData ReadCallback;

/**
 * Wrapper around mutable slices.
 *
 * This is meant as a safe type to pass across the FFI boundary with similar semantics as regular
 * slice. However, not all functionality is present, use the slice conversion functions.
 */
typedef struct CSliceMut_ReadData {
    ReadData *data;
    uintptr_t len;
} CSliceMut_ReadData;

/**
 * FFI compatible iterator.
 *
 * Any mutable reference to an iterator can be converted to a `CIterator`.
 *
 * `CIterator<T>` implements `Iterator<Item = T>`.
 *
 * # Examples
 *
 * Using [`AsCIterator`](AsCIterator) helper:
 *
 * ```
 * use cglue::iter::{CIterator, AsCIterator};
 *
 * extern "C" fn sum_all(iter: CIterator<usize>) -> usize {
 *     iter.sum()
 * }
 *
 * let mut iter = (0..10).map(|v| v * v);
 *
 * assert_eq!(sum_all(iter.as_citer()), 285);
 * ```
 *
 * Converting with `Into` trait:
 *
 * ```
 * use cglue::iter::{CIterator, AsCIterator};
 *
 * extern "C" fn sum_all(iter: CIterator<usize>) -> usize {
 *     iter.sum()
 * }
 *
 * let mut iter = (0..=10).map(|v| v * v);
 *
 * assert_eq!(sum_all((&mut iter).into()), 385);
 * ```
 */
typedef struct CIterator_WriteData {
    void *iter;
    int32_t (*func)(void*, WriteData *out);
} CIterator_WriteData;

typedef OpaqueCallback_WriteData WriteCallback;

/**
 * Wrapper around const slices.
 *
 * This is meant as a safe type to pass across the FFI boundary with similar semantics as regular
 * slice. However, not all functionality is present, use the slice conversion functions.
 *
 * # Examples
 *
 * Simple conversion:
 *
 * ```
 * use cglue::slice::CSliceRef;
 *
 * let arr = [0, 5, 3, 2];
 *
 * let cslice = CSliceRef::from(&arr[..]);
 *
 * let slice = cslice.as_slice();
 *
 * assert_eq!(&arr, slice);
 * ```
 */
typedef struct CSliceRef_WriteData {
    const WriteData *data;
    uintptr_t len;
} CSliceRef_WriteData;
/**
 * Base CGlue trait object for trait MemoryView.
 */
typedef struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void MemoryViewBase_CBox_c_void_____CArc_c_void;
/**
 * CtxBoxed CGlue trait object for trait MemoryView with context.
 */
typedef MemoryViewBase_CBox_c_void_____CArc_c_void MemoryViewBaseCtxBox_c_void__CArc_c_void;
/**
 * Boxed CGlue trait object for trait MemoryView with a [`CArc`](cglue::arc::CArc) reference counted context.
 */
typedef MemoryViewBaseCtxBox_c_void__CArc_c_void MemoryViewBaseArcBox_c_void__c_void;
/**
 * Opaque Boxed CGlue trait object for trait MemoryView with a [`CArc`](cglue::arc::CArc) reference counted context.
 */
typedef MemoryViewBaseArcBox_c_void__c_void MemoryViewArcBox;

/**
 * Simple CGlue trait object container.
 *
 * This is the simplest form of container, represented by an instance, clone context, and
 * temporary return context.
 *
 * `instance` value usually is either a reference, or a mutable reference, or a `CBox`, which
 * contains static reference to the instance, and a dedicated drop function for freeing resources.
 *
 * `context` is either `PhantomData` representing nothing, or typically a `CArc` that can be
 * cloned at will, reference counting some resource, like a `Library` for automatic unloading.
 *
 * `ret_tmp` is usually `PhantomData` representing nothing, unless the trait has functions that
 * return references to associated types, in which case space is reserved for wrapping structures.
 */
typedef struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void {
    struct CBox_c_void instance;
    CArc_c_void context;
} CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void;
/**
 * CGlue vtable for trait CpuState.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct CpuStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void {
    void (*pause)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void *cont);
    void (*resume)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void *cont);
} CpuStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void;
/**
 * Simple CGlue trait object.
 *
 * This is the simplest form of CGlue object, represented by a container and vtable for a single
 * trait.
 *
 * Container merely is a this pointer with some optional temporary return reference context.
 */
typedef struct CGlueTraitObj_CBox_c_void_____CpuStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void______________CArc_c_void_____CpuStateRetTmp_CArc_c_void {
    const struct CpuStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void *vtbl;
    struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void container;
} CGlueTraitObj_CBox_c_void_____CpuStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void______________CArc_c_void_____CpuStateRetTmp_CArc_c_void;

// Typedef for default container and context type
typedef struct CGlueTraitObj_CBox_c_void_____CpuStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void______________CArc_c_void_____CpuStateRetTmp_CArc_c_void CpuState;
/**
 * Base CGlue trait object for trait CpuState.
 */
typedef struct CGlueTraitObj_CBox_c_void_____CpuStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void______________CArc_c_void_____CpuStateRetTmp_CArc_c_void CpuStateBase_CBox_c_void_____CArc_c_void;
typedef struct IntoCpuStateContainer_CBox_c_void_____CArc_c_void {
    struct CBox_c_void instance;
    CArc_c_void context;
} IntoCpuStateContainer_CBox_c_void_____CArc_c_void;
/**
 * CGlue vtable for trait Clone.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct CloneVtbl_IntoCpuStateContainer_CBox_c_void_____CArc_c_void {
    struct IntoCpuStateContainer_CBox_c_void_____CArc_c_void (*clone)(const struct IntoCpuStateContainer_CBox_c_void_____CArc_c_void *cont);
} CloneVtbl_IntoCpuStateContainer_CBox_c_void_____CArc_c_void;
/**
 * CGlue vtable for trait CpuState.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct CpuStateVtbl_IntoCpuStateContainer_CBox_c_void_____CArc_c_void {
    void (*pause)(struct IntoCpuStateContainer_CBox_c_void_____CArc_c_void *cont);
    void (*resume)(struct IntoCpuStateContainer_CBox_c_void_____CArc_c_void *cont);
} CpuStateVtbl_IntoCpuStateContainer_CBox_c_void_____CArc_c_void;
/**
 * Trait group potentially implementing `:: cglue :: ext :: core :: clone :: Clone < > + CpuState < >` traits.
 *
 * Optional traits are not implemented here, however. There are numerous conversion
 * functions available for safely retrieving a concrete collection of traits.
 *
 * `check_impl_` functions allow to check if the object implements the wanted traits.
 *
 * `into_impl_` functions consume the object and produce a new final structure that
 * keeps only the required information.
 *
 * `cast_impl_` functions merely check and transform the object into a type that can
 *be transformed back into `IntoCpuState` without losing data.
 *
 * `as_ref_`, and `as_mut_` functions obtain references to safe objects, but do not
 * perform any memory transformations either. They are the safest to use, because
 * there is no risk of accidentally consuming the whole object.
 */
typedef struct IntoCpuState_CBox_c_void_____CArc_c_void {
    const struct CloneVtbl_IntoCpuStateContainer_CBox_c_void_____CArc_c_void *vtbl_clone;
    const struct CpuStateVtbl_IntoCpuStateContainer_CBox_c_void_____CArc_c_void *vtbl_cpustate;
    struct IntoCpuStateContainer_CBox_c_void_____CArc_c_void container;
} IntoCpuState_CBox_c_void_____CArc_c_void;

// Typedef for default container and context type
typedef struct IntoCpuState_CBox_c_void_____CArc_c_void IntoCpuState;
/**
 * CGlue vtable for trait ConnectorCpuState.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct ConnectorCpuStateVtbl_ConnectorInstanceContainer_CBox_c_void_____CArc_c_void {
    int32_t (*cpu_state)(struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                         CpuStateBase_CBox_c_void_____CArc_c_void *ok_out);
    int32_t (*into_cpu_state)(struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void cont,
                              struct IntoCpuState_CBox_c_void_____CArc_c_void *ok_out);
} ConnectorCpuStateVtbl_ConnectorInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * Trait group potentially implementing `:: cglue :: ext :: core :: clone :: Clone < > + PhysicalMemory < > + ConnectorCpuState < >` traits.
 *
 * Optional traits are not implemented here, however. There are numerous conversion
 * functions available for safely retrieving a concrete collection of traits.
 *
 * `check_impl_` functions allow to check if the object implements the wanted traits.
 *
 * `into_impl_` functions consume the object and produce a new final structure that
 * keeps only the required information.
 *
 * `cast_impl_` functions merely check and transform the object into a type that can
 *be transformed back into `ConnectorInstance` without losing data.
 *
 * `as_ref_`, and `as_mut_` functions obtain references to safe objects, but do not
 * perform any memory transformations either. They are the safest to use, because
 * there is no risk of accidentally consuming the whole object.
 */
typedef struct ConnectorInstance_CBox_c_void_____CArc_c_void {
    const struct CloneVtbl_ConnectorInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_clone;
    const struct PhysicalMemoryVtbl_ConnectorInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_physicalmemory;
    const struct ConnectorCpuStateVtbl_ConnectorInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_connectorcpustate;
    struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void container;
} ConnectorInstance_CBox_c_void_____CArc_c_void;

// Typedef for default container and context type
typedef struct ConnectorInstance_CBox_c_void_____CArc_c_void ConnectorInstance;

typedef struct ConnectorInstance_CBox_c_void_____CArc_c_void ConnectorInstanceBaseCtxBox_c_void__CArc_c_void;

typedef ConnectorInstanceBaseCtxBox_c_void__CArc_c_void ConnectorInstanceBaseArcBox_c_void__c_void;

typedef ConnectorInstanceBaseArcBox_c_void__c_void ConnectorInstanceArcBox;

typedef ConnectorInstanceArcBox MuConnectorInstanceArcBox;

typedef struct OsInstanceContainer_CBox_c_void_____CArc_c_void {
    struct CBox_c_void instance;
    struct CArc_c_void context;
} OsInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait Clone.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct CloneVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void {
    struct OsInstanceContainer_CBox_c_void_____CArc_c_void (*clone)(const struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont);
} CloneVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void;

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

/**
 * Exit code of a process
 */
typedef int32_t ExitCode;

/**
 * The state of a process
 *
 * # Remarks
 *
 * In case the exit code isn't known ProcessState::Unknown is set.
 */
typedef enum ProcessState_Tag {
    ProcessState_Unknown,
    ProcessState_Alive,
    ProcessState_Dead,
} ProcessState_Tag;

typedef struct ProcessState {
    ProcessState_Tag tag;
    union {
        struct {
            ExitCode dead;
        };
    };
} ProcessState;

/**
 * Wrapper around null-terminated C-style strings.
 *
 * Analog to Rust's `String`, [`ReprCString`] owns the underlying data.
 */
typedef char *ReprCString;

typedef enum ArchitectureIdent_Tag {
    /**
     * Unknown architecture. Could be third-party implemented. memflow knows how to work on them,
     * but is unable to instantiate them.
     */
    ArchitectureIdent_Unknown,
    /**
     * X86 with specified bitness and address extensions
     *
     * First argument - `bitness` controls whether it's 32, or 64 bit variant.
     * Second argument - `address_extensions` control whether address extensions are
     * enabled (PAE on x32, or LA57 on x64). Warning: LA57 is currently unsupported.
     */
    ArchitectureIdent_X86,
    /**
     * Arm 64-bit architecture with specified page size
     *
     * Valid page sizes are 4kb, 16kb, 64kb. Only 4kb is supported at the moment
     */
    ArchitectureIdent_AArch64,
} ArchitectureIdent_Tag;

typedef struct ArchitectureIdent_X86_Body {
    uint8_t _0;
    bool _1;
} ArchitectureIdent_X86_Body;

typedef struct ArchitectureIdent {
    ArchitectureIdent_Tag tag;
    union {
        struct {
            uintptr_t unknown;
        };
        ArchitectureIdent_X86_Body x86;
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
     * The current status of the process at the time when this process info was fetched.
     *
     * # Remarks
     *
     * This field is highly volatile and can be re-checked with the [`Process::state()`] function.
     */
    struct ProcessState state;
    /**
     * Name of the process.
     */
    ReprCString name;
    /**
     * Path of the process binary
     */
    ReprCString path;
    /**
     * Command line the process was started with.
     */
    ReprCString command_line;
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
    /**
     * Directory Table Base
     *
     * # Remarks
     *
     * These fields contain the translation base used to translate virtual memory addresses into physical memory addresses.
     * On x86 systems only `dtb1` is set because only one dtb is used.
     * On arm systems both `dtb1` and `dtb2` are set to their corresponding values.
     */
    Address dtb1;
    Address dtb2;
} ProcessInfo;

typedef struct Callback_c_void__ProcessInfo {
    void *context;
    bool (*func)(void*, struct ProcessInfo);
} Callback_c_void__ProcessInfo;

typedef struct Callback_c_void__ProcessInfo OpaqueCallback_ProcessInfo;

typedef OpaqueCallback_ProcessInfo ProcessInfoCallback;

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
    umem size;
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

typedef struct Callback_c_void__ModuleInfo {
    void *context;
    bool (*func)(void*, struct ModuleInfo);
} Callback_c_void__ModuleInfo;

typedef struct Callback_c_void__ModuleInfo OpaqueCallback_ModuleInfo;

typedef OpaqueCallback_ModuleInfo ModuleInfoCallback;

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
    umem offset;
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
    umem offset;
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
    umem size;
} SectionInfo;

typedef struct Callback_c_void__SectionInfo {
    void *context;
    bool (*func)(void*, struct SectionInfo);
} Callback_c_void__SectionInfo;

typedef struct Callback_c_void__SectionInfo OpaqueCallback_SectionInfo;

typedef OpaqueCallback_SectionInfo SectionCallback;

typedef int64_t imem;

/**
 * FFI-safe 3 element tuple.
 */
typedef struct CTup3_Address__umem__PageType {
    Address _0;
    umem _1;
    PageType _2;
} CTup3_Address__umem__PageType;

typedef struct CTup3_Address__umem__PageType MemoryRange;

typedef struct Callback_c_void__MemoryRange {
    void *context;
    bool (*func)(void*, MemoryRange);
} Callback_c_void__MemoryRange;

typedef struct Callback_c_void__MemoryRange OpaqueCallback_MemoryRange;

typedef OpaqueCallback_MemoryRange MemoryRangeCallback;

/**
 * FFI-safe 2 element tuple.
 */
typedef struct CTup2_Address__umem {
    Address _0;
    umem _1;
} CTup2_Address__umem;

typedef struct CTup2_Address__umem VtopRange;

/**
 * Wrapper around const slices.
 *
 * This is meant as a safe type to pass across the FFI boundary with similar semantics as regular
 * slice. However, not all functionality is present, use the slice conversion functions.
 *
 * # Examples
 *
 * Simple conversion:
 *
 * ```
 * use cglue::slice::CSliceRef;
 *
 * let arr = [0, 5, 3, 2];
 *
 * let cslice = CSliceRef::from(&arr[..]);
 *
 * let slice = cslice.as_slice();
 *
 * assert_eq!(&arr, slice);
 * ```
 */
typedef struct CSliceRef_VtopRange {
    const VtopRange *data;
    uintptr_t len;
} CSliceRef_VtopRange;

/**
 * Virtual page range information with physical mappings used for callbacks
 */
typedef struct VirtualTranslation {
    Address in_virtual;
    umem size;
    struct PhysicalAddress out_physical;
} VirtualTranslation;

typedef struct Callback_c_void__VirtualTranslation {
    void *context;
    bool (*func)(void*, struct VirtualTranslation);
} Callback_c_void__VirtualTranslation;

typedef struct Callback_c_void__VirtualTranslation OpaqueCallback_VirtualTranslation;

typedef OpaqueCallback_VirtualTranslation VirtualTranslationCallback;

typedef struct VirtualTranslationFail {
    Address from;
    umem size;
} VirtualTranslationFail;

typedef struct Callback_c_void__VirtualTranslationFail {
    void *context;
    bool (*func)(void*, struct VirtualTranslationFail);
} Callback_c_void__VirtualTranslationFail;

typedef struct Callback_c_void__VirtualTranslationFail OpaqueCallback_VirtualTranslationFail;

typedef OpaqueCallback_VirtualTranslationFail VirtualTranslationFailCallback;

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
    umem page_size;
} Page;
/**
 * A page object that is invalid.
 */
#define Page_INVALID (Page){ .page_type = PageType_UNKNOWN, .page_base = Address_INVALID, .page_size = 0 }

/**
 * FFI-safe Option.
 *
 * This type is not really meant for general use, but rather as a last-resort conversion for type
 * wrapping.
 *
 * Typical workflow would include temporarily converting into/from COption.
 */
typedef enum COption_Address_Tag {
    COption_Address_None_Address,
    COption_Address_Some_Address,
} COption_Address_Tag;

typedef struct COption_Address {
    COption_Address_Tag tag;
    union {
        struct {
            Address some;
        };
    };
} COption_Address;

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
    umem size;
    /**
     * System architecture
     */
    struct ArchitectureIdent arch;
} OsInfo;

/**
 * CGlue vtable for trait Os.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct OsVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void {
    int32_t (*process_address_list_callback)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                             AddressCallback callback);
    int32_t (*process_info_list_callback)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                          ProcessInfoCallback callback);
    int32_t (*process_info_by_address)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                       Address address,
                                       struct ProcessInfo *ok_out);
    int32_t (*process_info_by_name)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                    struct CSliceRef_u8 name,
                                    struct ProcessInfo *ok_out);
    int32_t (*process_info_by_pid)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                   Pid pid,
                                   struct ProcessInfo *ok_out);
    int32_t (*process_by_info)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                               struct ProcessInfo info,
                               struct ProcessInstance_CBox_c_void_____CArc_c_void *ok_out);
    int32_t (*into_process_by_info)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void cont,
                                    struct ProcessInfo info,
                                    struct IntoProcessInstance_CBox_c_void_____CArc_c_void *ok_out);
    int32_t (*process_by_address)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                  Address addr,
                                  struct ProcessInstance_CBox_c_void_____CArc_c_void *ok_out);
    int32_t (*process_by_name)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                               struct CSliceRef_u8 name,
                               struct ProcessInstance_CBox_c_void_____CArc_c_void *ok_out);
    int32_t (*process_by_pid)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              Pid pid,
                              struct ProcessInstance_CBox_c_void_____CArc_c_void *ok_out);
    int32_t (*into_process_by_address)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void cont,
                                       Address addr,
                                       struct IntoProcessInstance_CBox_c_void_____CArc_c_void *ok_out);
    int32_t (*into_process_by_name)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void cont,
                                    struct CSliceRef_u8 name,
                                    struct IntoProcessInstance_CBox_c_void_____CArc_c_void *ok_out);
    int32_t (*into_process_by_pid)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void cont,
                                   Pid pid,
                                   struct IntoProcessInstance_CBox_c_void_____CArc_c_void *ok_out);
    int32_t (*module_address_list_callback)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                            AddressCallback callback);
    int32_t (*module_list_callback)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                    ModuleInfoCallback callback);
    int32_t (*module_by_address)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                 Address address,
                                 struct ModuleInfo *ok_out);
    int32_t (*module_by_name)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              struct CSliceRef_u8 name,
                              struct ModuleInfo *ok_out);
    int32_t (*primary_module_address)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                      Address *ok_out);
    int32_t (*primary_module)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              struct ModuleInfo *ok_out);
    int32_t (*module_import_list_callback)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                           const struct ModuleInfo *info,
                                           ImportCallback callback);
    int32_t (*module_export_list_callback)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                           const struct ModuleInfo *info,
                                           ExportCallback callback);
    int32_t (*module_section_list_callback)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                            const struct ModuleInfo *info,
                                            SectionCallback callback);
    int32_t (*module_import_by_name)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                     const struct ModuleInfo *info,
                                     struct CSliceRef_u8 name,
                                     struct ImportInfo *ok_out);
    int32_t (*module_export_by_name)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                     const struct ModuleInfo *info,
                                     struct CSliceRef_u8 name,
                                     struct ExportInfo *ok_out);
    int32_t (*module_section_by_name)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                      const struct ModuleInfo *info,
                                      struct CSliceRef_u8 name,
                                      struct SectionInfo *ok_out);
    const struct OsInfo *(*info)(const struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont);
} OsVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait MemoryView.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct MemoryViewVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void {
    int32_t (*read_raw_iter)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                             ReadRawMemOps data);
    int32_t (*write_raw_iter)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              WriteRawMemOps data);
    struct MemoryViewMetadata (*metadata)(const struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont);
    int32_t (*read_iter)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                         struct CIterator_ReadData inp,
                         ReadCallback *out,
                         ReadCallback *out_fail);
    int32_t (*read_raw_list)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                             struct CSliceMut_ReadData data);
    int32_t (*read_raw_into)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                             Address addr,
                             struct CSliceMut_u8 out);
    int32_t (*write_iter)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                          struct CIterator_WriteData inp,
                          WriteCallback *out,
                          WriteCallback *out_fail);
    int32_t (*write_raw_list)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              struct CSliceRef_WriteData data);
    int32_t (*write_raw)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                         Address addr,
                         struct CSliceRef_u8 data);
} MemoryViewVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * Simple CGlue trait object container.
 *
 * This is the simplest form of container, represented by an instance, clone context, and
 * temporary return context.
 *
 * `instance` value usually is either a reference, or a mutable reference, or a `CBox`, which
 * contains static reference to the instance, and a dedicated drop function for freeing resources.
 *
 * `context` is either `PhantomData` representing nothing, or typically a `CArc` that can be
 * cloned at will, reference counting some resource, like a `Library` for automatic unloading.
 *
 * `ret_tmp` is usually `PhantomData` representing nothing, unless the trait has functions that
 * return references to associated types, in which case space is reserved for wrapping structures.
 */
typedef struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void {
    struct CBox_c_void instance;
    CArc_c_void context;
} CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void;
/**
 * Simple CGlue trait object container.
 *
 * This is the simplest form of container, represented by an instance, clone context, and
 * temporary return context.
 *
 * `instance` value usually is either a reference, or a mutable reference, or a `CBox`, which
 * contains static reference to the instance, and a dedicated drop function for freeing resources.
 *
 * `context` is either `PhantomData` representing nothing, or typically a `CArc` that can be
 * cloned at will, reference counting some resource, like a `Library` for automatic unloading.
 *
 * `ret_tmp` is usually `PhantomData` representing nothing, unless the trait has functions that
 * return references to associated types, in which case space is reserved for wrapping structures.
 */
typedef struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardStateRetTmp_CArc_c_void {
    struct CBox_c_void instance;
    CArc_c_void context;
} CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardStateRetTmp_CArc_c_void;
/**
 * CGlue vtable for trait KeyboardState.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct KeyboardStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardStateRetTmp_CArc_c_void {
    bool (*is_down)(const struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardStateRetTmp_CArc_c_void *cont,
                    int32_t vk);
} KeyboardStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardStateRetTmp_CArc_c_void;
/**
 * Simple CGlue trait object.
 *
 * This is the simplest form of CGlue object, represented by a container and vtable for a single
 * trait.
 *
 * Container merely is a this pointer with some optional temporary return reference context.
 */
typedef struct CGlueTraitObj_CBox_c_void_____KeyboardStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardStateRetTmp_CArc_c_void______________CArc_c_void_____KeyboardStateRetTmp_CArc_c_void {
    const struct KeyboardStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardStateRetTmp_CArc_c_void *vtbl;
    struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardStateRetTmp_CArc_c_void container;
} CGlueTraitObj_CBox_c_void_____KeyboardStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardStateRetTmp_CArc_c_void______________CArc_c_void_____KeyboardStateRetTmp_CArc_c_void;

// Typedef for default container and context type
typedef struct CGlueTraitObj_CBox_c_void_____KeyboardStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardStateRetTmp_CArc_c_void______________CArc_c_void_____KeyboardStateRetTmp_CArc_c_void KeyboardState;
/**
 * Base CGlue trait object for trait KeyboardState.
 */
typedef struct CGlueTraitObj_CBox_c_void_____KeyboardStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardStateRetTmp_CArc_c_void______________CArc_c_void_____KeyboardStateRetTmp_CArc_c_void KeyboardStateBase_CBox_c_void_____CArc_c_void;
/**
 * CGlue vtable for trait Keyboard.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct KeyboardVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void {
    bool (*is_down)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void *cont,
                    int32_t vk);
    void (*set_down)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void *cont,
                     int32_t vk,
                     bool down);
    int32_t (*state)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void *cont,
                     KeyboardStateBase_CBox_c_void_____CArc_c_void *ok_out);
} KeyboardVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void;
/**
 * Simple CGlue trait object.
 *
 * This is the simplest form of CGlue object, represented by a container and vtable for a single
 * trait.
 *
 * Container merely is a this pointer with some optional temporary return reference context.
 */
typedef struct CGlueTraitObj_CBox_c_void_____KeyboardVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void______________CArc_c_void_____KeyboardRetTmp_CArc_c_void {
    const struct KeyboardVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void *vtbl;
    struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void container;
} CGlueTraitObj_CBox_c_void_____KeyboardVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void______________CArc_c_void_____KeyboardRetTmp_CArc_c_void;

// Typedef for default container and context type
typedef struct CGlueTraitObj_CBox_c_void_____KeyboardVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void______________CArc_c_void_____KeyboardRetTmp_CArc_c_void Keyboard;
/**
 * Base CGlue trait object for trait Keyboard.
 */
typedef struct CGlueTraitObj_CBox_c_void_____KeyboardVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void______________CArc_c_void_____KeyboardRetTmp_CArc_c_void KeyboardBase_CBox_c_void_____CArc_c_void;
typedef struct IntoKeyboardContainer_CBox_c_void_____CArc_c_void {
    struct CBox_c_void instance;
    CArc_c_void context;
} IntoKeyboardContainer_CBox_c_void_____CArc_c_void;
/**
 * CGlue vtable for trait Clone.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct CloneVtbl_IntoKeyboardContainer_CBox_c_void_____CArc_c_void {
    struct IntoKeyboardContainer_CBox_c_void_____CArc_c_void (*clone)(const struct IntoKeyboardContainer_CBox_c_void_____CArc_c_void *cont);
} CloneVtbl_IntoKeyboardContainer_CBox_c_void_____CArc_c_void;
/**
 * CGlue vtable for trait Keyboard.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct KeyboardVtbl_IntoKeyboardContainer_CBox_c_void_____CArc_c_void {
    bool (*is_down)(struct IntoKeyboardContainer_CBox_c_void_____CArc_c_void *cont, int32_t vk);
    void (*set_down)(struct IntoKeyboardContainer_CBox_c_void_____CArc_c_void *cont,
                     int32_t vk,
                     bool down);
    int32_t (*state)(struct IntoKeyboardContainer_CBox_c_void_____CArc_c_void *cont,
                     KeyboardStateBase_CBox_c_void_____CArc_c_void *ok_out);
} KeyboardVtbl_IntoKeyboardContainer_CBox_c_void_____CArc_c_void;
/**
 * Trait group potentially implementing `:: cglue :: ext :: core :: clone :: Clone < > + Keyboard < >` traits.
 *
 * Optional traits are not implemented here, however. There are numerous conversion
 * functions available for safely retrieving a concrete collection of traits.
 *
 * `check_impl_` functions allow to check if the object implements the wanted traits.
 *
 * `into_impl_` functions consume the object and produce a new final structure that
 * keeps only the required information.
 *
 * `cast_impl_` functions merely check and transform the object into a type that can
 *be transformed back into `IntoKeyboard` without losing data.
 *
 * `as_ref_`, and `as_mut_` functions obtain references to safe objects, but do not
 * perform any memory transformations either. They are the safest to use, because
 * there is no risk of accidentally consuming the whole object.
 */
typedef struct IntoKeyboard_CBox_c_void_____CArc_c_void {
    const struct CloneVtbl_IntoKeyboardContainer_CBox_c_void_____CArc_c_void *vtbl_clone;
    const struct KeyboardVtbl_IntoKeyboardContainer_CBox_c_void_____CArc_c_void *vtbl_keyboard;
    struct IntoKeyboardContainer_CBox_c_void_____CArc_c_void container;
} IntoKeyboard_CBox_c_void_____CArc_c_void;

// Typedef for default container and context type
typedef struct IntoKeyboard_CBox_c_void_____CArc_c_void IntoKeyboard;
/**
 * CGlue vtable for trait OsKeyboard.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct OsKeyboardVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void {
    int32_t (*keyboard)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                        KeyboardBase_CBox_c_void_____CArc_c_void *ok_out);
    int32_t (*into_keyboard)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void cont,
                             struct IntoKeyboard_CBox_c_void_____CArc_c_void *ok_out);
} OsKeyboardVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait PhysicalMemory.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct PhysicalMemoryVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void {
    int32_t (*phys_read_raw_iter)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                  PhysicalReadMemOps data);
    int32_t (*phys_write_raw_iter)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                   PhysicalWriteMemOps data);
    struct PhysicalMemoryMetadata (*metadata)(const struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont);
    void (*set_mem_map)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                        struct CSliceRef_PhysicalMemoryMapping _mem_map);
    MemoryViewBase_CBox_c_void_____CArc_c_void (*into_phys_view)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void cont);
    MemoryViewBase_CBox_c_void_____CArc_c_void (*phys_view)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont);
} PhysicalMemoryVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait VirtualTranslate.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct VirtualTranslateVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void {
    void (*virt_to_phys_list)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              struct CSliceRef_VtopRange addrs,
                              VirtualTranslationCallback out,
                              VirtualTranslationFailCallback out_fail);
    void (*virt_to_phys_range)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                               Address start,
                               Address end,
                               VirtualTranslationCallback out);
    void (*virt_translation_map_range)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                       Address start,
                                       Address end,
                                       VirtualTranslationCallback out);
    void (*virt_page_map_range)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                imem gap_size,
                                Address start,
                                Address end,
                                MemoryRangeCallback out);
    int32_t (*virt_to_phys)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                            Address address,
                            struct PhysicalAddress *ok_out);
    int32_t (*virt_page_info)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              Address addr,
                              struct Page *ok_out);
    void (*virt_translation_map)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                 VirtualTranslationCallback out);
    struct COption_Address (*phys_to_virt)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                           Address phys);
    void (*virt_page_map)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                          imem gap_size,
                          MemoryRangeCallback out);
} VirtualTranslateVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * Trait group potentially implementing `:: cglue :: ext :: core :: clone :: Clone < > + Os < > + MemoryView < > + OsKeyboard < > + PhysicalMemory < > + VirtualTranslate < >` traits.
 *
 * Optional traits are not implemented here, however. There are numerous conversion
 * functions available for safely retrieving a concrete collection of traits.
 *
 * `check_impl_` functions allow to check if the object implements the wanted traits.
 *
 * `into_impl_` functions consume the object and produce a new final structure that
 * keeps only the required information.
 *
 * `cast_impl_` functions merely check and transform the object into a type that can
 *be transformed back into `OsInstance` without losing data.
 *
 * `as_ref_`, and `as_mut_` functions obtain references to safe objects, but do not
 * perform any memory transformations either. They are the safest to use, because
 * there is no risk of accidentally consuming the whole object.
 */
typedef struct OsInstance_CBox_c_void_____CArc_c_void {
    const struct CloneVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_clone;
    const struct OsVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_os;
    const struct MemoryViewVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_memoryview;
    const struct OsKeyboardVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_oskeyboard;
    const struct PhysicalMemoryVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_physicalmemory;
    const struct VirtualTranslateVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_virtualtranslate;
    struct OsInstanceContainer_CBox_c_void_____CArc_c_void container;
} OsInstance_CBox_c_void_____CArc_c_void;

// Typedef for default container and context type
typedef struct OsInstance_CBox_c_void_____CArc_c_void OsInstance;

typedef struct OsInstance_CBox_c_void_____CArc_c_void OsInstanceBaseCtxBox_c_void__CArc_c_void;

typedef OsInstanceBaseCtxBox_c_void__CArc_c_void OsInstanceBaseArcBox_c_void__c_void;

typedef OsInstanceBaseArcBox_c_void__c_void OsInstanceArcBox;

typedef OsInstanceArcBox MuOsInstanceArcBox;

typedef struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void {
    struct CBox_c_void instance;
    struct CArc_c_void context;
} ProcessInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait MemoryView.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct MemoryViewVtbl_ProcessInstanceContainer_CBox_c_void_____CArc_c_void {
    int32_t (*read_raw_iter)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                             ReadRawMemOps data);
    int32_t (*write_raw_iter)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              WriteRawMemOps data);
    struct MemoryViewMetadata (*metadata)(const struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont);
    int32_t (*read_iter)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                         struct CIterator_ReadData inp,
                         ReadCallback *out,
                         ReadCallback *out_fail);
    int32_t (*read_raw_list)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                             struct CSliceMut_ReadData data);
    int32_t (*read_raw_into)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                             Address addr,
                             struct CSliceMut_u8 out);
    int32_t (*write_iter)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                          struct CIterator_WriteData inp,
                          WriteCallback *out,
                          WriteCallback *out_fail);
    int32_t (*write_raw_list)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              struct CSliceRef_WriteData data);
    int32_t (*write_raw)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                         Address addr,
                         struct CSliceRef_u8 data);
} MemoryViewVtbl_ProcessInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait Process.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct ProcessVtbl_ProcessInstanceContainer_CBox_c_void_____CArc_c_void {
    struct ProcessState (*state)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont);
    int32_t (*set_dtb)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                       Address dtb1,
                       Address dtb2);
    int32_t (*module_address_list_callback)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                            const struct ArchitectureIdent *target_arch,
                                            ModuleAddressCallback callback);
    int32_t (*module_list_callback)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                    const struct ArchitectureIdent *target_arch,
                                    ModuleInfoCallback callback);
    int32_t (*module_by_address)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                 Address address,
                                 struct ArchitectureIdent architecture,
                                 struct ModuleInfo *ok_out);
    int32_t (*module_by_name_arch)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                   struct CSliceRef_u8 name,
                                   const struct ArchitectureIdent *architecture,
                                   struct ModuleInfo *ok_out);
    int32_t (*module_by_name)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              struct CSliceRef_u8 name,
                              struct ModuleInfo *ok_out);
    int32_t (*primary_module_address)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                      Address *ok_out);
    int32_t (*primary_module)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              struct ModuleInfo *ok_out);
    int32_t (*module_import_list_callback)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                           const struct ModuleInfo *info,
                                           ImportCallback callback);
    int32_t (*module_export_list_callback)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                           const struct ModuleInfo *info,
                                           ExportCallback callback);
    int32_t (*module_section_list_callback)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                            const struct ModuleInfo *info,
                                            SectionCallback callback);
    int32_t (*module_import_by_name)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                     const struct ModuleInfo *info,
                                     struct CSliceRef_u8 name,
                                     struct ImportInfo *ok_out);
    int32_t (*module_export_by_name)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                     const struct ModuleInfo *info,
                                     struct CSliceRef_u8 name,
                                     struct ExportInfo *ok_out);
    int32_t (*module_section_by_name)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                      const struct ModuleInfo *info,
                                      struct CSliceRef_u8 name,
                                      struct SectionInfo *ok_out);
    const struct ProcessInfo *(*info)(const struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont);
    void (*mapped_mem_range)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                             imem gap_size,
                             Address start,
                             Address end,
                             MemoryRangeCallback out);
    void (*mapped_mem)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                       imem gap_size,
                       MemoryRangeCallback out);
} ProcessVtbl_ProcessInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait VirtualTranslate.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct VirtualTranslateVtbl_ProcessInstanceContainer_CBox_c_void_____CArc_c_void {
    void (*virt_to_phys_list)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              struct CSliceRef_VtopRange addrs,
                              VirtualTranslationCallback out,
                              VirtualTranslationFailCallback out_fail);
    void (*virt_to_phys_range)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                               Address start,
                               Address end,
                               VirtualTranslationCallback out);
    void (*virt_translation_map_range)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                       Address start,
                                       Address end,
                                       VirtualTranslationCallback out);
    void (*virt_page_map_range)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                imem gap_size,
                                Address start,
                                Address end,
                                MemoryRangeCallback out);
    int32_t (*virt_to_phys)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                            Address address,
                            struct PhysicalAddress *ok_out);
    int32_t (*virt_page_info)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              Address addr,
                              struct Page *ok_out);
    void (*virt_translation_map)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                 VirtualTranslationCallback out);
    struct COption_Address (*phys_to_virt)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                           Address phys);
    void (*virt_page_map)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                          imem gap_size,
                          MemoryRangeCallback out);
} VirtualTranslateVtbl_ProcessInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * Trait group potentially implementing `MemoryView < > + Process < > + VirtualTranslate < >` traits.
 *
 * Optional traits are not implemented here, however. There are numerous conversion
 * functions available for safely retrieving a concrete collection of traits.
 *
 * `check_impl_` functions allow to check if the object implements the wanted traits.
 *
 * `into_impl_` functions consume the object and produce a new final structure that
 * keeps only the required information.
 *
 * `cast_impl_` functions merely check and transform the object into a type that can
 *be transformed back into `ProcessInstance` without losing data.
 *
 * `as_ref_`, and `as_mut_` functions obtain references to safe objects, but do not
 * perform any memory transformations either. They are the safest to use, because
 * there is no risk of accidentally consuming the whole object.
 */
typedef struct ProcessInstance_CBox_c_void_____CArc_c_void {
    const struct MemoryViewVtbl_ProcessInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_memoryview;
    const struct ProcessVtbl_ProcessInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_process;
    const struct VirtualTranslateVtbl_ProcessInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_virtualtranslate;
    struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void container;
} ProcessInstance_CBox_c_void_____CArc_c_void;

// Typedef for default container and context type
typedef struct ProcessInstance_CBox_c_void_____CArc_c_void ProcessInstance;

typedef struct ProcessInstance_CBox_c_void_____CArc_c_void ProcessInstanceBaseCtxBox_c_void__CArc_c_void;

typedef ProcessInstanceBaseCtxBox_c_void__CArc_c_void ProcessInstanceBaseArcBox_c_void__c_void;

typedef ProcessInstanceBaseArcBox_c_void__c_void ProcessInstanceArcBox;

typedef struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void {
    struct CBox_c_void instance;
    struct CArc_c_void context;
} IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait Clone.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct CloneVtbl_IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void {
    struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void (*clone)(const struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont);
} CloneVtbl_IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait MemoryView.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct MemoryViewVtbl_IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void {
    int32_t (*read_raw_iter)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                             ReadRawMemOps data);
    int32_t (*write_raw_iter)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              WriteRawMemOps data);
    struct MemoryViewMetadata (*metadata)(const struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont);
    int32_t (*read_iter)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                         struct CIterator_ReadData inp,
                         ReadCallback *out,
                         ReadCallback *out_fail);
    int32_t (*read_raw_list)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                             struct CSliceMut_ReadData data);
    int32_t (*read_raw_into)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                             Address addr,
                             struct CSliceMut_u8 out);
    int32_t (*write_iter)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                          struct CIterator_WriteData inp,
                          WriteCallback *out,
                          WriteCallback *out_fail);
    int32_t (*write_raw_list)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              struct CSliceRef_WriteData data);
    int32_t (*write_raw)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                         Address addr,
                         struct CSliceRef_u8 data);
} MemoryViewVtbl_IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait Process.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct ProcessVtbl_IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void {
    struct ProcessState (*state)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont);
    int32_t (*set_dtb)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                       Address dtb1,
                       Address dtb2);
    int32_t (*module_address_list_callback)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                            const struct ArchitectureIdent *target_arch,
                                            ModuleAddressCallback callback);
    int32_t (*module_list_callback)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                    const struct ArchitectureIdent *target_arch,
                                    ModuleInfoCallback callback);
    int32_t (*module_by_address)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                 Address address,
                                 struct ArchitectureIdent architecture,
                                 struct ModuleInfo *ok_out);
    int32_t (*module_by_name_arch)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                   struct CSliceRef_u8 name,
                                   const struct ArchitectureIdent *architecture,
                                   struct ModuleInfo *ok_out);
    int32_t (*module_by_name)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              struct CSliceRef_u8 name,
                              struct ModuleInfo *ok_out);
    int32_t (*primary_module_address)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                      Address *ok_out);
    int32_t (*primary_module)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              struct ModuleInfo *ok_out);
    int32_t (*module_import_list_callback)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                           const struct ModuleInfo *info,
                                           ImportCallback callback);
    int32_t (*module_export_list_callback)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                           const struct ModuleInfo *info,
                                           ExportCallback callback);
    int32_t (*module_section_list_callback)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                            const struct ModuleInfo *info,
                                            SectionCallback callback);
    int32_t (*module_import_by_name)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                     const struct ModuleInfo *info,
                                     struct CSliceRef_u8 name,
                                     struct ImportInfo *ok_out);
    int32_t (*module_export_by_name)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                     const struct ModuleInfo *info,
                                     struct CSliceRef_u8 name,
                                     struct ExportInfo *ok_out);
    int32_t (*module_section_by_name)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                      const struct ModuleInfo *info,
                                      struct CSliceRef_u8 name,
                                      struct SectionInfo *ok_out);
    const struct ProcessInfo *(*info)(const struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont);
    void (*mapped_mem_range)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                             imem gap_size,
                             Address start,
                             Address end,
                             MemoryRangeCallback out);
    void (*mapped_mem)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                       imem gap_size,
                       MemoryRangeCallback out);
} ProcessVtbl_IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait VirtualTranslate.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct VirtualTranslateVtbl_IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void {
    void (*virt_to_phys_list)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              struct CSliceRef_VtopRange addrs,
                              VirtualTranslationCallback out,
                              VirtualTranslationFailCallback out_fail);
    void (*virt_to_phys_range)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                               Address start,
                               Address end,
                               VirtualTranslationCallback out);
    void (*virt_translation_map_range)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                       Address start,
                                       Address end,
                                       VirtualTranslationCallback out);
    void (*virt_page_map_range)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                imem gap_size,
                                Address start,
                                Address end,
                                MemoryRangeCallback out);
    int32_t (*virt_to_phys)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                            Address address,
                            struct PhysicalAddress *ok_out);
    int32_t (*virt_page_info)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                              Address addr,
                              struct Page *ok_out);
    void (*virt_translation_map)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                 VirtualTranslationCallback out);
    struct COption_Address (*phys_to_virt)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                           Address phys);
    void (*virt_page_map)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                          imem gap_size,
                          MemoryRangeCallback out);
} VirtualTranslateVtbl_IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * Trait group potentially implementing `:: cglue :: ext :: core :: clone :: Clone < > + MemoryView < > + Process < > + VirtualTranslate < >` traits.
 *
 * Optional traits are not implemented here, however. There are numerous conversion
 * functions available for safely retrieving a concrete collection of traits.
 *
 * `check_impl_` functions allow to check if the object implements the wanted traits.
 *
 * `into_impl_` functions consume the object and produce a new final structure that
 * keeps only the required information.
 *
 * `cast_impl_` functions merely check and transform the object into a type that can
 *be transformed back into `IntoProcessInstance` without losing data.
 *
 * `as_ref_`, and `as_mut_` functions obtain references to safe objects, but do not
 * perform any memory transformations either. They are the safest to use, because
 * there is no risk of accidentally consuming the whole object.
 */
typedef struct IntoProcessInstance_CBox_c_void_____CArc_c_void {
    const struct CloneVtbl_IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_clone;
    const struct MemoryViewVtbl_IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_memoryview;
    const struct ProcessVtbl_IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_process;
    const struct VirtualTranslateVtbl_IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_virtualtranslate;
    struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void container;
} IntoProcessInstance_CBox_c_void_____CArc_c_void;

// Typedef for default container and context type
typedef struct IntoProcessInstance_CBox_c_void_____CArc_c_void IntoProcessInstance;

typedef struct IntoProcessInstance_CBox_c_void_____CArc_c_void IntoProcessInstanceBaseCtxBox_c_void__CArc_c_void;

typedef IntoProcessInstanceBaseCtxBox_c_void__CArc_c_void IntoProcessInstanceBaseArcBox_c_void__c_void;

typedef IntoProcessInstanceBaseArcBox_c_void__c_void IntoProcessInstanceArcBox;

/**
 * Simple CGlue trait object container.
 *
 * This is the simplest form of container, represented by an instance, clone context, and
 * temporary return context.
 *
 * `instance` value usually is either a reference, or a mutable reference, or a `CBox`, which
 * contains static reference to the instance, and a dedicated drop function for freeing resources.
 *
 * `context` is either `PhantomData` representing nothing, or typically a `CArc` that can be
 * cloned at will, reference counting some resource, like a `Library` for automatic unloading.
 *
 * `ret_tmp` is usually `PhantomData` representing nothing, unless the trait has functions that
 * return references to associated types, in which case space is reserved for wrapping structures.
 */
typedef struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void {
    struct CBox_c_void instance;
    struct CArc_c_void context;
} CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void;

/**
 * CGlue vtable for trait MemoryView.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void {
    int32_t (*read_raw_iter)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void *cont,
                             ReadRawMemOps data);
    int32_t (*write_raw_iter)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void *cont,
                              WriteRawMemOps data);
    struct MemoryViewMetadata (*metadata)(const struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void *cont);
    int32_t (*read_iter)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void *cont,
                         struct CIterator_ReadData inp,
                         ReadCallback *out,
                         ReadCallback *out_fail);
    int32_t (*read_raw_list)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void *cont,
                             struct CSliceMut_ReadData data);
    int32_t (*read_raw_into)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void *cont,
                             Address addr,
                             struct CSliceMut_u8 out);
    int32_t (*write_iter)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void *cont,
                          struct CIterator_WriteData inp,
                          WriteCallback *out,
                          WriteCallback *out_fail);
    int32_t (*write_raw_list)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void *cont,
                              struct CSliceRef_WriteData data);
    int32_t (*write_raw)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void *cont,
                         Address addr,
                         struct CSliceRef_u8 data);
} MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void;

/**
 * Simple CGlue trait object.
 *
 * This is the simplest form of CGlue object, represented by a container and vtable for a single
 * trait.
 *
 * Container merely is a this pointer with some optional temporary return reference context.
 */
typedef struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void {
    const struct MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void *vtbl;
    struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void container;
} CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void;

// Typedef for default container and context type
typedef struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void MemoryView;/**
 * CGlue vtable for trait PhysicalMemory.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct PhysicalMemoryVtbl_ConnectorInstanceContainer_CBox_c_void_____CArc_c_void {
    int32_t (*phys_read_raw_iter)(struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                  PhysicalReadMemOps data);
    int32_t (*phys_write_raw_iter)(struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                                   PhysicalWriteMemOps data);
    struct PhysicalMemoryMetadata (*metadata)(const struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void *cont);
    void (*set_mem_map)(struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void *cont,
                        struct CSliceRef_PhysicalMemoryMapping _mem_map);
    MemoryViewBase_CBox_c_void_____CArc_c_void (*into_phys_view)(struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void cont);
    MemoryViewBase_CBox_c_void_____CArc_c_void (*phys_view)(struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void *cont);
} PhysicalMemoryVtbl_ConnectorInstanceContainer_CBox_c_void_____CArc_c_void;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

extern const struct ArchitectureObj *X86_32;

extern const struct ArchitectureObj *X86_32_PAE;

extern const struct ArchitectureObj *X86_64;

/**
 * Initialize logging with selected logging level.
 */
void log_init(LevelFilter level_filter);

/**
 * Logs a error message via log::error!
 *
 * # Safety
 *
 * The provided string must be a valid null-terminated char array.
 */
void log_error(const char *s);

/**
 * Logs a warning message via log::warn!
 *
 * # Safety
 *
 * The provided string must be a valid null-terminated char array.
 */
void log_warn(const char *s);

/**
 * Logs a info message via log::info!
 *
 * # Safety
 *
 * The provided string must be a valid null-terminated char array.
 */
void log_info(const char *s);

/**
 * Logs a debug message via log::debug!
 *
 * # Safety
 *
 * The provided string must be a valid null-terminated char array.
 */
void log_debug(const char *s);

/**
 * Logs a trace message via log::trace!
 *
 * # Safety
 *
 * The provided string must be a valid null-terminated char array.
 */
void log_trace(const char *s);

/**
 * Logs an error code with custom log level.
 */
void log_errorcode(Level level, int32_t error);

/**
 * Logs an error with debug log level.
 */
void log_debug_errorcode(int32_t error);

/**
 * Sets new maximum log level.
 *
 * If `inventory` is supplied, the log level is also updated within all plugin instances. However,
 * if it is not supplied, plugins will not have their log levels updated, potentially leading to
 * lower performance, or less logging than expected.
 */
void log_set_max_level(LevelFilter level_filter, const struct Inventory *inventory);

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
                                   MuConnectorInstanceArcBox *out);

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
                            ConnectorInstanceArcBox *mem,
                            MuOsInstanceArcBox *out);

/**
 * Free a os plugin
 *
 * # Safety
 *
 * `os` must point to a valid `OsInstance` that was created using one of the provided
 * functions.
 */
void os_drop(OsInstanceArcBox *os);

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
void connector_clone(const ConnectorInstanceArcBox *conn, MuConnectorInstanceArcBox *out);

/**
 * Free a connector instance
 *
 * # Safety
 *
 * `conn` has to point to a valid [`ConnectorInstance`](ConnectorInstanceArcBox) created by one of the provided
 * functions.
 *
 * There has to be no instance of `PhysicalMemory` created from the input `conn`, because they
 * will become invalid.
 */
void connector_drop(ConnectorInstanceArcBox *conn);

/**
 * Free a connector inventory
 *
 * # Safety
 *
 * `inv` must point to a valid `Inventory` that was created using one of the provided
 * functions.
 */
void inventory_free(struct Inventory *inv);

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
static CArc_c_void ctx_arc_clone(CArc_c_void *self) {
    CArc_c_void ret = *self;
    ret.instance = self->clone_fn(self->instance);
    return ret;
}

void ctx_arc_drop(CArc_c_void *self) {
    if (self->drop_fn && self->instance) self->drop_fn(self->instance);
}
void cont_box_drop(CBox_c_void *self) {
    if (self->drop_fn && self->instance) self->drop_fn(self->instance);
}

static inline void mf_pause(void *self)  {
(((struct CGlueTraitObj_CBox_c_void_____CpuStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void______________CArc_c_void_____CpuStateRetTmp_CArc_c_void *)self)->vtbl)->pause(&((struct CGlueTraitObj_CBox_c_void_____CpuStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void______________CArc_c_void_____CpuStateRetTmp_CArc_c_void *)self)->container);

}

static inline void mf_resume(void *self)  {
(((struct CGlueTraitObj_CBox_c_void_____CpuStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void______________CArc_c_void_____CpuStateRetTmp_CArc_c_void *)self)->vtbl)->resume(&((struct CGlueTraitObj_CBox_c_void_____CpuStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void______________CArc_c_void_____CpuStateRetTmp_CArc_c_void *)self)->container);

}

static inline void mf_cpustate_drop(struct CGlueTraitObj_CBox_c_void_____CpuStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____CpuStateRetTmp_CArc_c_void______________CArc_c_void_____CpuStateRetTmp_CArc_c_void self)  {
    cont_box_drop(&self.container.instance);
    ctx_arc_drop(&self.container.context);

}

static inline bool mf_keyboardstate_is_down(const void *self, int32_t vk)  {
    bool __ret = (((const struct CGlueTraitObj_CBox_c_void_____KeyboardStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardStateRetTmp_CArc_c_void______________CArc_c_void_____KeyboardStateRetTmp_CArc_c_void *)self)->vtbl)->is_down(&((const struct CGlueTraitObj_CBox_c_void_____KeyboardStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardStateRetTmp_CArc_c_void______________CArc_c_void_____KeyboardStateRetTmp_CArc_c_void *)self)->container, vk);
    return __ret;
}

static inline void mf_keyboardstate_drop(struct CGlueTraitObj_CBox_c_void_____KeyboardStateVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardStateRetTmp_CArc_c_void______________CArc_c_void_____KeyboardStateRetTmp_CArc_c_void self)  {
    cont_box_drop(&self.container.instance);
    ctx_arc_drop(&self.container.context);

}

static inline bool mf_keyboard_is_down(void *self, int32_t vk)  {
    bool __ret = (((struct CGlueTraitObj_CBox_c_void_____KeyboardVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void______________CArc_c_void_____KeyboardRetTmp_CArc_c_void *)self)->vtbl)->is_down(&((struct CGlueTraitObj_CBox_c_void_____KeyboardVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void______________CArc_c_void_____KeyboardRetTmp_CArc_c_void *)self)->container, vk);
    return __ret;
}

static inline void mf_set_down(void *self, int32_t vk, bool down)  {
(((struct CGlueTraitObj_CBox_c_void_____KeyboardVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void______________CArc_c_void_____KeyboardRetTmp_CArc_c_void *)self)->vtbl)->set_down(&((struct CGlueTraitObj_CBox_c_void_____KeyboardVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void______________CArc_c_void_____KeyboardRetTmp_CArc_c_void *)self)->container, vk, down);

}

static inline int32_t mf_state(void *self, KeyboardStateBase_CBox_c_void_____CArc_c_void * ok_out)  {
    int32_t __ret = (((struct CGlueTraitObj_CBox_c_void_____KeyboardVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void______________CArc_c_void_____KeyboardRetTmp_CArc_c_void *)self)->vtbl)->state(&((struct CGlueTraitObj_CBox_c_void_____KeyboardVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void______________CArc_c_void_____KeyboardRetTmp_CArc_c_void *)self)->container, ok_out);
    return __ret;
}

static inline void mf_keyboard_drop(struct CGlueTraitObj_CBox_c_void_____KeyboardVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____KeyboardRetTmp_CArc_c_void______________CArc_c_void_____KeyboardRetTmp_CArc_c_void self)  {
    cont_box_drop(&self.container.instance);
    ctx_arc_drop(&self.container.context);

}

static inline int32_t mf_read_raw_iter(void *self, ReadRawMemOps data)  {
    int32_t __ret = (((struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void *)self)->vtbl)->read_raw_iter(&((struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void *)self)->container, data);
    return __ret;
}

static inline int32_t mf_write_raw_iter(void *self, WriteRawMemOps data)  {
    int32_t __ret = (((struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void *)self)->vtbl)->write_raw_iter(&((struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void *)self)->container, data);
    return __ret;
}

static inline struct MemoryViewMetadata mf_metadata(const void *self)  {
    struct MemoryViewMetadata __ret = (((const struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void *)self)->vtbl)->metadata(&((const struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void *)self)->container);
    return __ret;
}

static inline int32_t mf_read_iter(void *self, struct CIterator_ReadData inp, ReadCallback * out, ReadCallback * out_fail)  {
    int32_t __ret = (((struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void *)self)->vtbl)->read_iter(&((struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void *)self)->container, inp, out, out_fail);
    return __ret;
}

static inline int32_t mf_read_raw_list(void *self, struct CSliceMut_ReadData data)  {
    int32_t __ret = (((struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void *)self)->vtbl)->read_raw_list(&((struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void *)self)->container, data);
    return __ret;
}

static inline int32_t mf_read_raw_into(void *self, Address addr, struct CSliceMut_u8 out)  {
    int32_t __ret = (((struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void *)self)->vtbl)->read_raw_into(&((struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void *)self)->container, addr, out);
    return __ret;
}

static inline int32_t mf_write_iter(void *self, struct CIterator_WriteData inp, WriteCallback * out, WriteCallback * out_fail)  {
    int32_t __ret = (((struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void *)self)->vtbl)->write_iter(&((struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void *)self)->container, inp, out, out_fail);
    return __ret;
}

static inline int32_t mf_write_raw_list(void *self, struct CSliceRef_WriteData data)  {
    int32_t __ret = (((struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void *)self)->vtbl)->write_raw_list(&((struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void *)self)->container, data);
    return __ret;
}

static inline int32_t mf_write_raw(void *self, Address addr, struct CSliceRef_u8 data)  {
    int32_t __ret = (((struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void *)self)->vtbl)->write_raw(&((struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void *)self)->container, addr, data);
    return __ret;
}

static inline void mf_memoryview_drop(struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void______________CArc_c_void_____MemoryViewRetTmp_CArc_c_void self)  {
    cont_box_drop(&self.container.instance);
    ctx_arc_drop(&self.container.context);

}

static inline struct ConnectorInstance_CBox_c_void_____CArc_c_void mf_connectorinstance_clone(const void *self)  {
    struct ConnectorInstance_CBox_c_void_____CArc_c_void __ret;
    __ret.container = (((const struct ConnectorInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_clone)->clone(&((const struct ConnectorInstance_CBox_c_void_____CArc_c_void *)self)->container);
    return __ret;
}

static inline void mf_connectorinstance_drop(struct ConnectorInstance_CBox_c_void_____CArc_c_void self)  {
    cont_box_drop(&self.container.instance);
    ctx_arc_drop(&self.container.context);

}

static inline struct IntoCpuState_CBox_c_void_____CArc_c_void mf_intocpustate_clone(const void *self)  {
    struct IntoCpuState_CBox_c_void_____CArc_c_void __ret;
    __ret.container = (((const struct IntoCpuState_CBox_c_void_____CArc_c_void *)self)->vtbl_clone)->clone(&((const struct IntoCpuState_CBox_c_void_____CArc_c_void *)self)->container);
    return __ret;
}

static inline void mf_intocpustate_drop(struct IntoCpuState_CBox_c_void_____CArc_c_void self)  {
    cont_box_drop(&self.container.instance);
    ctx_arc_drop(&self.container.context);

}

static inline void mf_intocpustate_pause(void *self)  {
(((struct IntoCpuState_CBox_c_void_____CArc_c_void *)self)->vtbl_cpustate)->pause(&((struct IntoCpuState_CBox_c_void_____CArc_c_void *)self)->container);

}

static inline void mf_intocpustate_resume(void *self)  {
(((struct IntoCpuState_CBox_c_void_____CArc_c_void *)self)->vtbl_cpustate)->resume(&((struct IntoCpuState_CBox_c_void_____CArc_c_void *)self)->container);

}

static inline int32_t mf_connectorinstance_cpu_state(void *self, CpuStateBase_CBox_c_void_____CArc_c_void * ok_out)  {
    int32_t __ret = (((struct ConnectorInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_connectorcpustate)->cpu_state(&((struct ConnectorInstance_CBox_c_void_____CArc_c_void *)self)->container, ok_out);
    return __ret;
}

static inline int32_t mf_connectorinstance_into_cpu_state(struct ConnectorInstance_CBox_c_void_____CArc_c_void self, struct IntoCpuState_CBox_c_void_____CArc_c_void * ok_out)  {
    CArc_c_void ___ctx = ctx_arc_clone(&self.container.context);
    int32_t __ret = (self.vtbl_connectorcpustate)->into_cpu_state(self.container, ok_out);
    ctx_arc_drop(&___ctx);
    return __ret;
}

static inline struct OsInstance_CBox_c_void_____CArc_c_void mf_osinstance_clone(const void *self)  {
    struct OsInstance_CBox_c_void_____CArc_c_void __ret;
    __ret.container = (((const struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_clone)->clone(&((const struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container);
    return __ret;
}

static inline void mf_osinstance_drop(struct OsInstance_CBox_c_void_____CArc_c_void self)  {
    cont_box_drop(&self.container.instance);
    ctx_arc_drop(&self.container.context);

}

static inline int32_t mf_osinstance_process_address_list_callback(void *self, AddressCallback callback)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->process_address_list_callback(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, callback);
    return __ret;
}

static inline int32_t mf_osinstance_process_info_list_callback(void *self, ProcessInfoCallback callback)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->process_info_list_callback(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, callback);
    return __ret;
}

static inline int32_t mf_osinstance_process_info_by_address(void *self, Address address, struct ProcessInfo * ok_out)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->process_info_by_address(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, address, ok_out);
    return __ret;
}

static inline int32_t mf_osinstance_process_info_by_name(void *self, struct CSliceRef_u8 name, struct ProcessInfo * ok_out)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->process_info_by_name(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, name, ok_out);
    return __ret;
}

static inline int32_t mf_osinstance_process_info_by_pid(void *self, Pid pid, struct ProcessInfo * ok_out)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->process_info_by_pid(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, pid, ok_out);
    return __ret;
}

static inline int32_t mf_osinstance_process_by_info(void *self, struct ProcessInfo info, struct ProcessInstance_CBox_c_void_____CArc_c_void * ok_out)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->process_by_info(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, info, ok_out);
    return __ret;
}

static inline int32_t mf_osinstance_into_process_by_info(struct OsInstance_CBox_c_void_____CArc_c_void self, struct ProcessInfo info, struct IntoProcessInstance_CBox_c_void_____CArc_c_void * ok_out)  {
    CArc_c_void ___ctx = ctx_arc_clone(&self.container.context);
    int32_t __ret = (self.vtbl_os)->into_process_by_info(self.container, info, ok_out);
    ctx_arc_drop(&___ctx);
    return __ret;
}

static inline int32_t mf_osinstance_process_by_address(void *self, Address addr, struct ProcessInstance_CBox_c_void_____CArc_c_void * ok_out)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->process_by_address(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, addr, ok_out);
    return __ret;
}

static inline int32_t mf_osinstance_process_by_name(void *self, struct CSliceRef_u8 name, struct ProcessInstance_CBox_c_void_____CArc_c_void * ok_out)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->process_by_name(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, name, ok_out);
    return __ret;
}

static inline int32_t mf_osinstance_process_by_pid(void *self, Pid pid, struct ProcessInstance_CBox_c_void_____CArc_c_void * ok_out)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->process_by_pid(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, pid, ok_out);
    return __ret;
}

static inline int32_t mf_osinstance_into_process_by_address(struct OsInstance_CBox_c_void_____CArc_c_void self, Address addr, struct IntoProcessInstance_CBox_c_void_____CArc_c_void * ok_out)  {
    CArc_c_void ___ctx = ctx_arc_clone(&self.container.context);
    int32_t __ret = (self.vtbl_os)->into_process_by_address(self.container, addr, ok_out);
    ctx_arc_drop(&___ctx);
    return __ret;
}

static inline int32_t mf_osinstance_into_process_by_name(struct OsInstance_CBox_c_void_____CArc_c_void self, struct CSliceRef_u8 name, struct IntoProcessInstance_CBox_c_void_____CArc_c_void * ok_out)  {
    CArc_c_void ___ctx = ctx_arc_clone(&self.container.context);
    int32_t __ret = (self.vtbl_os)->into_process_by_name(self.container, name, ok_out);
    ctx_arc_drop(&___ctx);
    return __ret;
}

static inline int32_t mf_osinstance_into_process_by_pid(struct OsInstance_CBox_c_void_____CArc_c_void self, Pid pid, struct IntoProcessInstance_CBox_c_void_____CArc_c_void * ok_out)  {
    CArc_c_void ___ctx = ctx_arc_clone(&self.container.context);
    int32_t __ret = (self.vtbl_os)->into_process_by_pid(self.container, pid, ok_out);
    ctx_arc_drop(&___ctx);
    return __ret;
}

static inline int32_t mf_osinstance_module_address_list_callback(void *self, AddressCallback callback)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->module_address_list_callback(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, callback);
    return __ret;
}

static inline int32_t mf_osinstance_module_list_callback(void *self, ModuleInfoCallback callback)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->module_list_callback(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, callback);
    return __ret;
}

static inline int32_t mf_osinstance_module_by_address(void *self, Address address, struct ModuleInfo * ok_out)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->module_by_address(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, address, ok_out);
    return __ret;
}

static inline int32_t mf_osinstance_module_by_name(void *self, struct CSliceRef_u8 name, struct ModuleInfo * ok_out)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->module_by_name(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, name, ok_out);
    return __ret;
}

static inline int32_t mf_osinstance_primary_module_address(void *self, Address * ok_out)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->primary_module_address(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, ok_out);
    return __ret;
}

static inline int32_t mf_osinstance_primary_module(void *self, struct ModuleInfo * ok_out)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->primary_module(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, ok_out);
    return __ret;
}

static inline int32_t mf_osinstance_module_import_list_callback(void *self, const struct ModuleInfo * info, ImportCallback callback)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->module_import_list_callback(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, info, callback);
    return __ret;
}

static inline int32_t mf_osinstance_module_export_list_callback(void *self, const struct ModuleInfo * info, ExportCallback callback)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->module_export_list_callback(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, info, callback);
    return __ret;
}

static inline int32_t mf_osinstance_module_section_list_callback(void *self, const struct ModuleInfo * info, SectionCallback callback)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->module_section_list_callback(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, info, callback);
    return __ret;
}

static inline int32_t mf_osinstance_module_import_by_name(void *self, const struct ModuleInfo * info, struct CSliceRef_u8 name, struct ImportInfo * ok_out)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->module_import_by_name(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, info, name, ok_out);
    return __ret;
}

static inline int32_t mf_osinstance_module_export_by_name(void *self, const struct ModuleInfo * info, struct CSliceRef_u8 name, struct ExportInfo * ok_out)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->module_export_by_name(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, info, name, ok_out);
    return __ret;
}

static inline int32_t mf_osinstance_module_section_by_name(void *self, const struct ModuleInfo * info, struct CSliceRef_u8 name, struct SectionInfo * ok_out)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->module_section_by_name(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, info, name, ok_out);
    return __ret;
}

static inline const struct OsInfo * mf_osinstance_info(const void *self)  {
    const struct OsInfo * __ret = (((const struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_os)->info(&((const struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container);
    return __ret;
}

static inline int32_t mf_osinstance_read_raw_iter(void *self, ReadRawMemOps data)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->read_raw_iter(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, data);
    return __ret;
}

static inline int32_t mf_osinstance_write_raw_iter(void *self, WriteRawMemOps data)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->write_raw_iter(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, data);
    return __ret;
}

static inline struct MemoryViewMetadata mf_osinstance_metadata(const void *self)  {
    struct MemoryViewMetadata __ret = (((const struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->metadata(&((const struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container);
    return __ret;
}

static inline int32_t mf_osinstance_read_iter(void *self, struct CIterator_ReadData inp, ReadCallback * out, ReadCallback * out_fail)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->read_iter(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, inp, out, out_fail);
    return __ret;
}

static inline int32_t mf_osinstance_read_raw_list(void *self, struct CSliceMut_ReadData data)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->read_raw_list(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, data);
    return __ret;
}

static inline int32_t mf_osinstance_read_raw_into(void *self, Address addr, struct CSliceMut_u8 out)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->read_raw_into(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, addr, out);
    return __ret;
}

static inline int32_t mf_osinstance_write_iter(void *self, struct CIterator_WriteData inp, WriteCallback * out, WriteCallback * out_fail)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->write_iter(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, inp, out, out_fail);
    return __ret;
}

static inline int32_t mf_osinstance_write_raw_list(void *self, struct CSliceRef_WriteData data)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->write_raw_list(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, data);
    return __ret;
}

static inline int32_t mf_osinstance_write_raw(void *self, Address addr, struct CSliceRef_u8 data)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->write_raw(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, addr, data);
    return __ret;
}

static inline struct IntoKeyboard_CBox_c_void_____CArc_c_void mf_intokeyboard_clone(const void *self)  {
    struct IntoKeyboard_CBox_c_void_____CArc_c_void __ret;
    __ret.container = (((const struct IntoKeyboard_CBox_c_void_____CArc_c_void *)self)->vtbl_clone)->clone(&((const struct IntoKeyboard_CBox_c_void_____CArc_c_void *)self)->container);
    return __ret;
}

static inline void mf_intokeyboard_drop(struct IntoKeyboard_CBox_c_void_____CArc_c_void self)  {
    cont_box_drop(&self.container.instance);
    ctx_arc_drop(&self.container.context);

}

static inline bool mf_intokeyboard_is_down(void *self, int32_t vk)  {
    bool __ret = (((struct IntoKeyboard_CBox_c_void_____CArc_c_void *)self)->vtbl_keyboard)->is_down(&((struct IntoKeyboard_CBox_c_void_____CArc_c_void *)self)->container, vk);
    return __ret;
}

static inline void mf_intokeyboard_set_down(void *self, int32_t vk, bool down)  {
(((struct IntoKeyboard_CBox_c_void_____CArc_c_void *)self)->vtbl_keyboard)->set_down(&((struct IntoKeyboard_CBox_c_void_____CArc_c_void *)self)->container, vk, down);

}

static inline int32_t mf_intokeyboard_state(void *self, KeyboardStateBase_CBox_c_void_____CArc_c_void * ok_out)  {
    int32_t __ret = (((struct IntoKeyboard_CBox_c_void_____CArc_c_void *)self)->vtbl_keyboard)->state(&((struct IntoKeyboard_CBox_c_void_____CArc_c_void *)self)->container, ok_out);
    return __ret;
}

static inline int32_t mf_osinstance_keyboard(void *self, KeyboardBase_CBox_c_void_____CArc_c_void * ok_out)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_oskeyboard)->keyboard(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, ok_out);
    return __ret;
}

static inline int32_t mf_osinstance_into_keyboard(struct OsInstance_CBox_c_void_____CArc_c_void self, struct IntoKeyboard_CBox_c_void_____CArc_c_void * ok_out)  {
    CArc_c_void ___ctx = ctx_arc_clone(&self.container.context);
    int32_t __ret = (self.vtbl_oskeyboard)->into_keyboard(self.container, ok_out);
    ctx_arc_drop(&___ctx);
    return __ret;
}

static inline int32_t mf_osinstance_phys_read_raw_iter(void *self, PhysicalReadMemOps data)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_physicalmemory)->phys_read_raw_iter(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, data);
    return __ret;
}

static inline int32_t mf_osinstance_phys_write_raw_iter(void *self, PhysicalWriteMemOps data)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_physicalmemory)->phys_write_raw_iter(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, data);
    return __ret;
}

static inline void mf_osinstance_set_mem_map(void *self, struct CSliceRef_PhysicalMemoryMapping _mem_map)  {
(((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_physicalmemory)->set_mem_map(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, _mem_map);

}

static inline MemoryViewBase_CBox_c_void_____CArc_c_void mf_osinstance_into_phys_view(struct OsInstance_CBox_c_void_____CArc_c_void self)  {
    CArc_c_void ___ctx = ctx_arc_clone(&self.container.context);
    MemoryViewBase_CBox_c_void_____CArc_c_void __ret = (self.vtbl_physicalmemory)->into_phys_view(self.container);
    ctx_arc_drop(&___ctx);
    return __ret;
}

static inline MemoryViewBase_CBox_c_void_____CArc_c_void mf_osinstance_phys_view(void *self)  {
    MemoryViewBase_CBox_c_void_____CArc_c_void __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_physicalmemory)->phys_view(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container);
    return __ret;
}

static inline void mf_osinstance_virt_to_phys_list(void *self, struct CSliceRef_VtopRange addrs, VirtualTranslationCallback out, VirtualTranslationFailCallback out_fail)  {
(((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_to_phys_list(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, addrs, out, out_fail);

}

static inline void mf_osinstance_virt_to_phys_range(void *self, Address start, Address end, VirtualTranslationCallback out)  {
(((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_to_phys_range(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, start, end, out);

}

static inline void mf_osinstance_virt_translation_map_range(void *self, Address start, Address end, VirtualTranslationCallback out)  {
(((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_translation_map_range(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, start, end, out);

}

static inline void mf_osinstance_virt_page_map_range(void *self, imem gap_size, Address start, Address end, MemoryRangeCallback out)  {
(((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_page_map_range(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, gap_size, start, end, out);

}

static inline int32_t mf_osinstance_virt_to_phys(void *self, Address address, struct PhysicalAddress * ok_out)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_to_phys(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, address, ok_out);
    return __ret;
}

static inline int32_t mf_osinstance_virt_page_info(void *self, Address addr, struct Page * ok_out)  {
    int32_t __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_page_info(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, addr, ok_out);
    return __ret;
}

static inline void mf_osinstance_virt_translation_map(void *self, VirtualTranslationCallback out)  {
(((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_translation_map(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, out);

}

static inline struct COption_Address mf_osinstance_phys_to_virt(void *self, Address phys)  {
    struct COption_Address __ret = (((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->phys_to_virt(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, phys);
    return __ret;
}

static inline void mf_osinstance_virt_page_map(void *self, imem gap_size, MemoryRangeCallback out)  {
(((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_page_map(&((struct OsInstance_CBox_c_void_____CArc_c_void *)self)->container, gap_size, out);

}

static inline int32_t mf_processinstance_read_raw_iter(void *self, ReadRawMemOps data)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->read_raw_iter(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, data);
    return __ret;
}

static inline int32_t mf_processinstance_write_raw_iter(void *self, WriteRawMemOps data)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->write_raw_iter(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, data);
    return __ret;
}

static inline struct MemoryViewMetadata mf_processinstance_metadata(const void *self)  {
    struct MemoryViewMetadata __ret = (((const struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->metadata(&((const struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container);
    return __ret;
}

static inline int32_t mf_processinstance_read_iter(void *self, struct CIterator_ReadData inp, ReadCallback * out, ReadCallback * out_fail)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->read_iter(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, inp, out, out_fail);
    return __ret;
}

static inline int32_t mf_processinstance_read_raw_list(void *self, struct CSliceMut_ReadData data)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->read_raw_list(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, data);
    return __ret;
}

static inline int32_t mf_processinstance_read_raw_into(void *self, Address addr, struct CSliceMut_u8 out)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->read_raw_into(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, addr, out);
    return __ret;
}

static inline int32_t mf_processinstance_write_iter(void *self, struct CIterator_WriteData inp, WriteCallback * out, WriteCallback * out_fail)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->write_iter(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, inp, out, out_fail);
    return __ret;
}

static inline int32_t mf_processinstance_write_raw_list(void *self, struct CSliceRef_WriteData data)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->write_raw_list(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, data);
    return __ret;
}

static inline int32_t mf_processinstance_write_raw(void *self, Address addr, struct CSliceRef_u8 data)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->write_raw(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, addr, data);
    return __ret;
}

static inline void mf_processinstance_drop(struct ProcessInstance_CBox_c_void_____CArc_c_void self)  {
    cont_box_drop(&self.container.instance);
    ctx_arc_drop(&self.container.context);

}

static inline struct ProcessState mf_processinstance_state(void *self)  {
    struct ProcessState __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->state(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container);
    return __ret;
}

static inline int32_t mf_processinstance_set_dtb(void *self, Address dtb1, Address dtb2)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->set_dtb(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, dtb1, dtb2);
    return __ret;
}

static inline int32_t mf_processinstance_module_address_list_callback(void *self, const struct ArchitectureIdent * target_arch, ModuleAddressCallback callback)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_address_list_callback(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, target_arch, callback);
    return __ret;
}

static inline int32_t mf_processinstance_module_list_callback(void *self, const struct ArchitectureIdent * target_arch, ModuleInfoCallback callback)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_list_callback(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, target_arch, callback);
    return __ret;
}

static inline int32_t mf_processinstance_module_by_address(void *self, Address address, struct ArchitectureIdent architecture, struct ModuleInfo * ok_out)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_by_address(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, address, architecture, ok_out);
    return __ret;
}

static inline int32_t mf_processinstance_module_by_name_arch(void *self, struct CSliceRef_u8 name, const struct ArchitectureIdent * architecture, struct ModuleInfo * ok_out)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_by_name_arch(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, name, architecture, ok_out);
    return __ret;
}

static inline int32_t mf_processinstance_module_by_name(void *self, struct CSliceRef_u8 name, struct ModuleInfo * ok_out)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_by_name(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, name, ok_out);
    return __ret;
}

static inline int32_t mf_processinstance_primary_module_address(void *self, Address * ok_out)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->primary_module_address(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, ok_out);
    return __ret;
}

static inline int32_t mf_processinstance_primary_module(void *self, struct ModuleInfo * ok_out)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->primary_module(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, ok_out);
    return __ret;
}

static inline int32_t mf_processinstance_module_import_list_callback(void *self, const struct ModuleInfo * info, ImportCallback callback)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_import_list_callback(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, info, callback);
    return __ret;
}

static inline int32_t mf_processinstance_module_export_list_callback(void *self, const struct ModuleInfo * info, ExportCallback callback)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_export_list_callback(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, info, callback);
    return __ret;
}

static inline int32_t mf_processinstance_module_section_list_callback(void *self, const struct ModuleInfo * info, SectionCallback callback)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_section_list_callback(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, info, callback);
    return __ret;
}

static inline int32_t mf_processinstance_module_import_by_name(void *self, const struct ModuleInfo * info, struct CSliceRef_u8 name, struct ImportInfo * ok_out)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_import_by_name(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, info, name, ok_out);
    return __ret;
}

static inline int32_t mf_processinstance_module_export_by_name(void *self, const struct ModuleInfo * info, struct CSliceRef_u8 name, struct ExportInfo * ok_out)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_export_by_name(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, info, name, ok_out);
    return __ret;
}

static inline int32_t mf_processinstance_module_section_by_name(void *self, const struct ModuleInfo * info, struct CSliceRef_u8 name, struct SectionInfo * ok_out)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_section_by_name(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, info, name, ok_out);
    return __ret;
}

static inline const struct ProcessInfo * mf_processinstance_info(const void *self)  {
    const struct ProcessInfo * __ret = (((const struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->info(&((const struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container);
    return __ret;
}

static inline void mf_processinstance_mapped_mem_range(void *self, imem gap_size, Address start, Address end, MemoryRangeCallback out)  {
(((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->mapped_mem_range(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, gap_size, start, end, out);

}

static inline void mf_processinstance_mapped_mem(void *self, imem gap_size, MemoryRangeCallback out)  {
(((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->mapped_mem(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, gap_size, out);

}

static inline void mf_processinstance_virt_to_phys_list(void *self, struct CSliceRef_VtopRange addrs, VirtualTranslationCallback out, VirtualTranslationFailCallback out_fail)  {
(((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_to_phys_list(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, addrs, out, out_fail);

}

static inline void mf_processinstance_virt_to_phys_range(void *self, Address start, Address end, VirtualTranslationCallback out)  {
(((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_to_phys_range(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, start, end, out);

}

static inline void mf_processinstance_virt_translation_map_range(void *self, Address start, Address end, VirtualTranslationCallback out)  {
(((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_translation_map_range(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, start, end, out);

}

static inline void mf_processinstance_virt_page_map_range(void *self, imem gap_size, Address start, Address end, MemoryRangeCallback out)  {
(((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_page_map_range(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, gap_size, start, end, out);

}

static inline int32_t mf_processinstance_virt_to_phys(void *self, Address address, struct PhysicalAddress * ok_out)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_to_phys(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, address, ok_out);
    return __ret;
}

static inline int32_t mf_processinstance_virt_page_info(void *self, Address addr, struct Page * ok_out)  {
    int32_t __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_page_info(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, addr, ok_out);
    return __ret;
}

static inline void mf_processinstance_virt_translation_map(void *self, VirtualTranslationCallback out)  {
(((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_translation_map(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, out);

}

static inline struct COption_Address mf_processinstance_phys_to_virt(void *self, Address phys)  {
    struct COption_Address __ret = (((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->phys_to_virt(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, phys);
    return __ret;
}

static inline void mf_processinstance_virt_page_map(void *self, imem gap_size, MemoryRangeCallback out)  {
(((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_page_map(&((struct ProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, gap_size, out);

}

static inline struct IntoProcessInstance_CBox_c_void_____CArc_c_void mf_intoprocessinstance_clone(const void *self)  {
    struct IntoProcessInstance_CBox_c_void_____CArc_c_void __ret;
    __ret.container = (((const struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_clone)->clone(&((const struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container);
    return __ret;
}

static inline void mf_intoprocessinstance_drop(struct IntoProcessInstance_CBox_c_void_____CArc_c_void self)  {
    cont_box_drop(&self.container.instance);
    ctx_arc_drop(&self.container.context);

}

static inline int32_t mf_intoprocessinstance_read_raw_iter(void *self, ReadRawMemOps data)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->read_raw_iter(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, data);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_write_raw_iter(void *self, WriteRawMemOps data)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->write_raw_iter(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, data);
    return __ret;
}

static inline struct MemoryViewMetadata mf_intoprocessinstance_metadata(const void *self)  {
    struct MemoryViewMetadata __ret = (((const struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->metadata(&((const struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_read_iter(void *self, struct CIterator_ReadData inp, ReadCallback * out, ReadCallback * out_fail)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->read_iter(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, inp, out, out_fail);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_read_raw_list(void *self, struct CSliceMut_ReadData data)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->read_raw_list(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, data);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_read_raw_into(void *self, Address addr, struct CSliceMut_u8 out)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->read_raw_into(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, addr, out);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_write_iter(void *self, struct CIterator_WriteData inp, WriteCallback * out, WriteCallback * out_fail)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->write_iter(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, inp, out, out_fail);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_write_raw_list(void *self, struct CSliceRef_WriteData data)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->write_raw_list(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, data);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_write_raw(void *self, Address addr, struct CSliceRef_u8 data)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_memoryview)->write_raw(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, addr, data);
    return __ret;
}

static inline struct ProcessState mf_intoprocessinstance_state(void *self)  {
    struct ProcessState __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->state(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_set_dtb(void *self, Address dtb1, Address dtb2)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->set_dtb(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, dtb1, dtb2);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_module_address_list_callback(void *self, const struct ArchitectureIdent * target_arch, ModuleAddressCallback callback)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_address_list_callback(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, target_arch, callback);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_module_list_callback(void *self, const struct ArchitectureIdent * target_arch, ModuleInfoCallback callback)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_list_callback(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, target_arch, callback);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_module_by_address(void *self, Address address, struct ArchitectureIdent architecture, struct ModuleInfo * ok_out)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_by_address(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, address, architecture, ok_out);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_module_by_name_arch(void *self, struct CSliceRef_u8 name, const struct ArchitectureIdent * architecture, struct ModuleInfo * ok_out)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_by_name_arch(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, name, architecture, ok_out);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_module_by_name(void *self, struct CSliceRef_u8 name, struct ModuleInfo * ok_out)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_by_name(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, name, ok_out);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_primary_module_address(void *self, Address * ok_out)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->primary_module_address(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, ok_out);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_primary_module(void *self, struct ModuleInfo * ok_out)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->primary_module(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, ok_out);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_module_import_list_callback(void *self, const struct ModuleInfo * info, ImportCallback callback)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_import_list_callback(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, info, callback);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_module_export_list_callback(void *self, const struct ModuleInfo * info, ExportCallback callback)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_export_list_callback(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, info, callback);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_module_section_list_callback(void *self, const struct ModuleInfo * info, SectionCallback callback)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_section_list_callback(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, info, callback);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_module_import_by_name(void *self, const struct ModuleInfo * info, struct CSliceRef_u8 name, struct ImportInfo * ok_out)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_import_by_name(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, info, name, ok_out);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_module_export_by_name(void *self, const struct ModuleInfo * info, struct CSliceRef_u8 name, struct ExportInfo * ok_out)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_export_by_name(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, info, name, ok_out);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_module_section_by_name(void *self, const struct ModuleInfo * info, struct CSliceRef_u8 name, struct SectionInfo * ok_out)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->module_section_by_name(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, info, name, ok_out);
    return __ret;
}

static inline const struct ProcessInfo * mf_intoprocessinstance_info(const void *self)  {
    const struct ProcessInfo * __ret = (((const struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->info(&((const struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container);
    return __ret;
}

static inline void mf_intoprocessinstance_mapped_mem_range(void *self, imem gap_size, Address start, Address end, MemoryRangeCallback out)  {
(((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->mapped_mem_range(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, gap_size, start, end, out);

}

static inline void mf_intoprocessinstance_mapped_mem(void *self, imem gap_size, MemoryRangeCallback out)  {
(((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_process)->mapped_mem(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, gap_size, out);

}

static inline void mf_intoprocessinstance_virt_to_phys_list(void *self, struct CSliceRef_VtopRange addrs, VirtualTranslationCallback out, VirtualTranslationFailCallback out_fail)  {
(((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_to_phys_list(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, addrs, out, out_fail);

}

static inline void mf_intoprocessinstance_virt_to_phys_range(void *self, Address start, Address end, VirtualTranslationCallback out)  {
(((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_to_phys_range(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, start, end, out);

}

static inline void mf_intoprocessinstance_virt_translation_map_range(void *self, Address start, Address end, VirtualTranslationCallback out)  {
(((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_translation_map_range(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, start, end, out);

}

static inline void mf_intoprocessinstance_virt_page_map_range(void *self, imem gap_size, Address start, Address end, MemoryRangeCallback out)  {
(((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_page_map_range(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, gap_size, start, end, out);

}

static inline int32_t mf_intoprocessinstance_virt_to_phys(void *self, Address address, struct PhysicalAddress * ok_out)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_to_phys(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, address, ok_out);
    return __ret;
}

static inline int32_t mf_intoprocessinstance_virt_page_info(void *self, Address addr, struct Page * ok_out)  {
    int32_t __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_page_info(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, addr, ok_out);
    return __ret;
}

static inline void mf_intoprocessinstance_virt_translation_map(void *self, VirtualTranslationCallback out)  {
(((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_translation_map(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, out);

}

static inline struct COption_Address mf_intoprocessinstance_phys_to_virt(void *self, Address phys)  {
    struct COption_Address __ret = (((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->phys_to_virt(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, phys);
    return __ret;
}

static inline void mf_intoprocessinstance_virt_page_map(void *self, imem gap_size, MemoryRangeCallback out)  {
(((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_virtualtranslate)->virt_page_map(&((struct IntoProcessInstance_CBox_c_void_____CArc_c_void *)self)->container, gap_size, out);

}

static inline int32_t mf_connectorinstance_phys_read_raw_iter(void *self, PhysicalReadMemOps data)  {
    int32_t __ret = (((struct ConnectorInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_physicalmemory)->phys_read_raw_iter(&((struct ConnectorInstance_CBox_c_void_____CArc_c_void *)self)->container, data);
    return __ret;
}

static inline int32_t mf_connectorinstance_phys_write_raw_iter(void *self, PhysicalWriteMemOps data)  {
    int32_t __ret = (((struct ConnectorInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_physicalmemory)->phys_write_raw_iter(&((struct ConnectorInstance_CBox_c_void_____CArc_c_void *)self)->container, data);
    return __ret;
}

static inline struct PhysicalMemoryMetadata mf_connectorinstance_metadata(const void *self)  {
    struct PhysicalMemoryMetadata __ret = (((const struct ConnectorInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_physicalmemory)->metadata(&((const struct ConnectorInstance_CBox_c_void_____CArc_c_void *)self)->container);
    return __ret;
}

static inline void mf_connectorinstance_set_mem_map(void *self, struct CSliceRef_PhysicalMemoryMapping _mem_map)  {
(((struct ConnectorInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_physicalmemory)->set_mem_map(&((struct ConnectorInstance_CBox_c_void_____CArc_c_void *)self)->container, _mem_map);

}

static inline MemoryViewBase_CBox_c_void_____CArc_c_void mf_connectorinstance_into_phys_view(struct ConnectorInstance_CBox_c_void_____CArc_c_void self)  {
    CArc_c_void ___ctx = ctx_arc_clone(&self.container.context);
    MemoryViewBase_CBox_c_void_____CArc_c_void __ret = (self.vtbl_physicalmemory)->into_phys_view(self.container);
    ctx_arc_drop(&___ctx);
    return __ret;
}

static inline MemoryViewBase_CBox_c_void_____CArc_c_void mf_connectorinstance_phys_view(void *self)  {
    MemoryViewBase_CBox_c_void_____CArc_c_void __ret = (((struct ConnectorInstance_CBox_c_void_____CArc_c_void *)self)->vtbl_physicalmemory)->phys_view(&((struct ConnectorInstance_CBox_c_void_____CArc_c_void *)self)->container);
    return __ret;
}

struct CollectBase {
    /* Pointer to array of data */
    char *buf;
    /* Capacity of the buffer (in elements) */
    size_t capacity;
    /* Current size of the buffer (in elements) */
    size_t size;
};

// For memcpy
#include <string.h>

static bool cb_collect_static_base(struct CollectBase *ctx, size_t elem_size, void *info) {

    if (ctx->size < ctx->capacity) {
        memcpy(ctx->buf + elem_size * ctx->size++, info, elem_size);
    }

    return ctx->size < ctx->capacity;
}

static bool cb_collect_dynamic_base(struct CollectBase *ctx, size_t elem_size, void *info) {

    if (!ctx->buf || ctx->size >= ctx->capacity) {
        size_t new_capacity = ctx->buf ? ctx->capacity * 2 : 64;
        char *buf = (char *)realloc(ctx->buf, elem_size * new_capacity);
        if (buf) {
            ctx->buf = buf;
            ctx->capacity = new_capacity;
        }
    }

    if (!ctx->buf || ctx->size >= ctx->capacity) return false;

    memcpy(ctx->buf + elem_size * ctx->size++, info, elem_size);

    return true;
}

struct BufferIterator {
    /* Pointer to the data buffer */
    const char *buf;
    /* Number of elements in the buffer */
    size_t size;
    /* Current element index */
    size_t i;
    /* Size of the data element */
    size_t sz_elem;
};

static bool buf_iter_next(struct BufferIterator *iter, void *out) {
    if (iter->i >= iter->size) return 1;
    memcpy(out, iter->buf + iter->i++ * iter->sz_elem, iter->sz_elem);
    return 0;
}

static inline bool cb_collect_static_ReadData(struct CollectBase *ctx, ReadData info) {
    return cb_collect_static_base(ctx, sizeof(ReadData), &info);
}

static inline bool cb_collect_dynamic_ReadData(struct CollectBase *ctx, ReadData info) {
    return cb_collect_dynamic_base(ctx, sizeof(ReadData), &info);
}

static inline bool cb_count_ReadData(size_t *cnt, ReadData info) {
    return ++(*cnt);
}

static inline bool cb_collect_static_WriteData(struct CollectBase *ctx, WriteData info) {
    return cb_collect_static_base(ctx, sizeof(WriteData), &info);
}

static inline bool cb_collect_dynamic_WriteData(struct CollectBase *ctx, WriteData info) {
    return cb_collect_dynamic_base(ctx, sizeof(WriteData), &info);
}

static inline bool cb_count_WriteData(size_t *cnt, WriteData info) {
    return ++(*cnt);
}

static inline bool cb_collect_static_Address(struct CollectBase *ctx, Address info) {
    return cb_collect_static_base(ctx, sizeof(Address), &info);
}

static inline bool cb_collect_dynamic_Address(struct CollectBase *ctx, Address info) {
    return cb_collect_dynamic_base(ctx, sizeof(Address), &info);
}

static inline bool cb_count_Address(size_t *cnt, Address info) {
    return ++(*cnt);
}

static inline bool cb_collect_static_ProcessInfo(struct CollectBase *ctx, ProcessInfo info) {
    return cb_collect_static_base(ctx, sizeof(ProcessInfo), &info);
}

static inline bool cb_collect_dynamic_ProcessInfo(struct CollectBase *ctx, ProcessInfo info) {
    return cb_collect_dynamic_base(ctx, sizeof(ProcessInfo), &info);
}

static inline bool cb_count_ProcessInfo(size_t *cnt, ProcessInfo info) {
    return ++(*cnt);
}

static inline bool cb_collect_static_ModuleAddressInfo(struct CollectBase *ctx, ModuleAddressInfo info) {
    return cb_collect_static_base(ctx, sizeof(ModuleAddressInfo), &info);
}

static inline bool cb_collect_dynamic_ModuleAddressInfo(struct CollectBase *ctx, ModuleAddressInfo info) {
    return cb_collect_dynamic_base(ctx, sizeof(ModuleAddressInfo), &info);
}

static inline bool cb_count_ModuleAddressInfo(size_t *cnt, ModuleAddressInfo info) {
    return ++(*cnt);
}

static inline bool cb_collect_static_ModuleInfo(struct CollectBase *ctx, ModuleInfo info) {
    return cb_collect_static_base(ctx, sizeof(ModuleInfo), &info);
}

static inline bool cb_collect_dynamic_ModuleInfo(struct CollectBase *ctx, ModuleInfo info) {
    return cb_collect_dynamic_base(ctx, sizeof(ModuleInfo), &info);
}

static inline bool cb_count_ModuleInfo(size_t *cnt, ModuleInfo info) {
    return ++(*cnt);
}

static inline bool cb_collect_static_ImportInfo(struct CollectBase *ctx, ImportInfo info) {
    return cb_collect_static_base(ctx, sizeof(ImportInfo), &info);
}

static inline bool cb_collect_dynamic_ImportInfo(struct CollectBase *ctx, ImportInfo info) {
    return cb_collect_dynamic_base(ctx, sizeof(ImportInfo), &info);
}

static inline bool cb_count_ImportInfo(size_t *cnt, ImportInfo info) {
    return ++(*cnt);
}

static inline bool cb_collect_static_ExportInfo(struct CollectBase *ctx, ExportInfo info) {
    return cb_collect_static_base(ctx, sizeof(ExportInfo), &info);
}

static inline bool cb_collect_dynamic_ExportInfo(struct CollectBase *ctx, ExportInfo info) {
    return cb_collect_dynamic_base(ctx, sizeof(ExportInfo), &info);
}

static inline bool cb_count_ExportInfo(size_t *cnt, ExportInfo info) {
    return ++(*cnt);
}

static inline bool cb_collect_static_SectionInfo(struct CollectBase *ctx, SectionInfo info) {
    return cb_collect_static_base(ctx, sizeof(SectionInfo), &info);
}

static inline bool cb_collect_dynamic_SectionInfo(struct CollectBase *ctx, SectionInfo info) {
    return cb_collect_dynamic_base(ctx, sizeof(SectionInfo), &info);
}

static inline bool cb_count_SectionInfo(size_t *cnt, SectionInfo info) {
    return ++(*cnt);
}

static inline bool cb_collect_static_MemoryRange(struct CollectBase *ctx, MemoryRange info) {
    return cb_collect_static_base(ctx, sizeof(MemoryRange), &info);
}

static inline bool cb_collect_dynamic_MemoryRange(struct CollectBase *ctx, MemoryRange info) {
    return cb_collect_dynamic_base(ctx, sizeof(MemoryRange), &info);
}

static inline bool cb_count_MemoryRange(size_t *cnt, MemoryRange info) {
    return ++(*cnt);
}

static inline bool cb_collect_static_VirtualTranslation(struct CollectBase *ctx, VirtualTranslation info) {
    return cb_collect_static_base(ctx, sizeof(VirtualTranslation), &info);
}

static inline bool cb_collect_dynamic_VirtualTranslation(struct CollectBase *ctx, VirtualTranslation info) {
    return cb_collect_dynamic_base(ctx, sizeof(VirtualTranslation), &info);
}

static inline bool cb_count_VirtualTranslation(size_t *cnt, VirtualTranslation info) {
    return ++(*cnt);
}

static inline bool cb_collect_static_VirtualTranslationFail(struct CollectBase *ctx, VirtualTranslationFail info) {
    return cb_collect_static_base(ctx, sizeof(VirtualTranslationFail), &info);
}

static inline bool cb_collect_dynamic_VirtualTranslationFail(struct CollectBase *ctx, VirtualTranslationFail info) {
    return cb_collect_dynamic_base(ctx, sizeof(VirtualTranslationFail), &info);
}

static inline bool cb_count_VirtualTranslationFail(size_t *cnt, VirtualTranslationFail info) {
    return ++(*cnt);
}


#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* MEMFLOW_H */
