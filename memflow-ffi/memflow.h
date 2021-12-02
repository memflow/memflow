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
    Endianess_LittleEndian,
    /**
     * Big Endianess
     */
    Endianess_BigEndian,
};
#ifndef __cplusplus
typedef uint8_t Endianess;
#endif // __cplusplus

typedef struct ArchitectureObj ArchitectureObj;

/**
 * Type definition for temporary return value wrapping storage.
 *
 * The trait does not use return wrapping, thus is a typedef to `PhantomData`.
 *
 * Note that `cbindgen` will generate wrong structures for this type. It is important
 * to go inside the generated headers and fix it - all RetTmp structures without a
 * body should be completely deleted, both as types, and as fields in the
 * groups/objects. If C++11 templates are generated, it is important to define a
 * custom type for CGlueTraitObj that does not have `ret_tmp` defined, and change all
 * type aliases of this trait to use that particular structure.
 */
typedef struct CloneRetTmp_CArc_c_void CloneRetTmp_CArc_c_void;

/**
 * Type definition for temporary return value wrapping storage.
 *
 * The trait does not use return wrapping, thus is a typedef to `PhantomData`.
 *
 * Note that `cbindgen` will generate wrong structures for this type. It is important
 * to go inside the generated headers and fix it - all RetTmp structures without a
 * body should be completely deleted, both as types, and as fields in the
 * groups/objects. If C++11 templates are generated, it is important to define a
 * custom type for CGlueTraitObj that does not have `ret_tmp` defined, and change all
 * type aliases of this trait to use that particular structure.
 */
typedef struct CloneRetTmp_Context CloneRetTmp_Context;

/**
 * Type definition for temporary return value wrapping storage.
 *
 * The trait does not use return wrapping, thus is a typedef to `PhantomData`.
 *
 * Note that `cbindgen` will generate wrong structures for this type. It is important
 * to go inside the generated headers and fix it - all RetTmp structures without a
 * body should be completely deleted, both as types, and as fields in the
 * groups/objects. If C++11 templates are generated, it is important to define a
 * custom type for CGlueTraitObj that does not have `ret_tmp` defined, and change all
 * type aliases of this trait to use that particular structure.
 */
typedef struct ConnectorCpuStateInnerRetTmp_CArc_c_void ConnectorCpuStateInnerRetTmp_CArc_c_void;

/**
 * Type definition for temporary return value wrapping storage.
 *
 * The trait does not use return wrapping, thus is a typedef to `PhantomData`.
 *
 * Note that `cbindgen` will generate wrong structures for this type. It is important
 * to go inside the generated headers and fix it - all RetTmp structures without a
 * body should be completely deleted, both as types, and as fields in the
 * groups/objects. If C++11 templates are generated, it is important to define a
 * custom type for CGlueTraitObj that does not have `ret_tmp` defined, and change all
 * type aliases of this trait to use that particular structure.
 */
typedef struct CpuStateRetTmp_Context CpuStateRetTmp_Context;

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
 * # use memflow::error::Result;
 * # use memflow::os::OsInstanceArcBox;
 * # fn test() -> Result<OsInstanceArcBox<'static>> {
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
 * ```no_run
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

/**
 * Type definition for temporary return value wrapping storage.
 *
 * The trait does not use return wrapping, thus is a typedef to `PhantomData`.
 *
 * Note that `cbindgen` will generate wrong structures for this type. It is important
 * to go inside the generated headers and fix it - all RetTmp structures without a
 * body should be completely deleted, both as types, and as fields in the
 * groups/objects. If C++11 templates are generated, it is important to define a
 * custom type for CGlueTraitObj that does not have `ret_tmp` defined, and change all
 * type aliases of this trait to use that particular structure.
 */
typedef struct KeyboardRetTmp_Context KeyboardRetTmp_Context;

/**
 * Type definition for temporary return value wrapping storage.
 *
 * The trait does not use return wrapping, thus is a typedef to `PhantomData`.
 *
 * Note that `cbindgen` will generate wrong structures for this type. It is important
 * to go inside the generated headers and fix it - all RetTmp structures without a
 * body should be completely deleted, both as types, and as fields in the
 * groups/objects. If C++11 templates are generated, it is important to define a
 * custom type for CGlueTraitObj that does not have `ret_tmp` defined, and change all
 * type aliases of this trait to use that particular structure.
 */
typedef struct KeyboardStateRetTmp_Context KeyboardStateRetTmp_Context;

/**
 * Type definition for temporary return value wrapping storage.
 *
 * The trait does not use return wrapping, thus is a typedef to `PhantomData`.
 *
 * Note that `cbindgen` will generate wrong structures for this type. It is important
 * to go inside the generated headers and fix it - all RetTmp structures without a
 * body should be completely deleted, both as types, and as fields in the
 * groups/objects. If C++11 templates are generated, it is important to define a
 * custom type for CGlueTraitObj that does not have `ret_tmp` defined, and change all
 * type aliases of this trait to use that particular structure.
 */
typedef struct MemoryViewRetTmp_CArc_c_void MemoryViewRetTmp_CArc_c_void;

/**
 * Type definition for temporary return value wrapping storage.
 *
 * The trait does not use return wrapping, thus is a typedef to `PhantomData`.
 *
 * Note that `cbindgen` will generate wrong structures for this type. It is important
 * to go inside the generated headers and fix it - all RetTmp structures without a
 * body should be completely deleted, both as types, and as fields in the
 * groups/objects. If C++11 templates are generated, it is important to define a
 * custom type for CGlueTraitObj that does not have `ret_tmp` defined, and change all
 * type aliases of this trait to use that particular structure.
 */
typedef struct MemoryViewRetTmp_Context MemoryViewRetTmp_Context;

/**
 * Type definition for temporary return value wrapping storage.
 *
 * The trait does not use return wrapping, thus is a typedef to `PhantomData`.
 *
 * Note that `cbindgen` will generate wrong structures for this type. It is important
 * to go inside the generated headers and fix it - all RetTmp structures without a
 * body should be completely deleted, both as types, and as fields in the
 * groups/objects. If C++11 templates are generated, it is important to define a
 * custom type for CGlueTraitObj that does not have `ret_tmp` defined, and change all
 * type aliases of this trait to use that particular structure.
 */
typedef struct OsInnerRetTmp_CArc_c_void OsInnerRetTmp_CArc_c_void;

/**
 * Type definition for temporary return value wrapping storage.
 *
 * The trait does not use return wrapping, thus is a typedef to `PhantomData`.
 *
 * Note that `cbindgen` will generate wrong structures for this type. It is important
 * to go inside the generated headers and fix it - all RetTmp structures without a
 * body should be completely deleted, both as types, and as fields in the
 * groups/objects. If C++11 templates are generated, it is important to define a
 * custom type for CGlueTraitObj that does not have `ret_tmp` defined, and change all
 * type aliases of this trait to use that particular structure.
 */
typedef struct OsKeyboardInnerRetTmp_CArc_c_void OsKeyboardInnerRetTmp_CArc_c_void;

/**
 * Type definition for temporary return value wrapping storage.
 *
 * The trait does not use return wrapping, thus is a typedef to `PhantomData`.
 *
 * Note that `cbindgen` will generate wrong structures for this type. It is important
 * to go inside the generated headers and fix it - all RetTmp structures without a
 * body should be completely deleted, both as types, and as fields in the
 * groups/objects. If C++11 templates are generated, it is important to define a
 * custom type for CGlueTraitObj that does not have `ret_tmp` defined, and change all
 * type aliases of this trait to use that particular structure.
 */
typedef struct PhysicalMemoryRetTmp_CArc_c_void PhysicalMemoryRetTmp_CArc_c_void;

/**
 * Type definition for temporary return value wrapping storage.
 *
 * The trait does not use return wrapping, thus is a typedef to `PhantomData`.
 *
 * Note that `cbindgen` will generate wrong structures for this type. It is important
 * to go inside the generated headers and fix it - all RetTmp structures without a
 * body should be completely deleted, both as types, and as fields in the
 * groups/objects. If C++11 templates are generated, it is important to define a
 * custom type for CGlueTraitObj that does not have `ret_tmp` defined, and change all
 * type aliases of this trait to use that particular structure.
 */
typedef struct ProcessRetTmp_CArc_c_void ProcessRetTmp_CArc_c_void;

/**
 * Type definition for temporary return value wrapping storage.
 *
 * The trait does not use return wrapping, thus is a typedef to `PhantomData`.
 *
 * Note that `cbindgen` will generate wrong structures for this type. It is important
 * to go inside the generated headers and fix it - all RetTmp structures without a
 * body should be completely deleted, both as types, and as fields in the
 * groups/objects. If C++11 templates are generated, it is important to define a
 * custom type for CGlueTraitObj that does not have `ret_tmp` defined, and change all
 * type aliases of this trait to use that particular structure.
 */
typedef struct ProcessRetTmp_Context ProcessRetTmp_Context;

/**
 * Type definition for temporary return value wrapping storage.
 *
 * The trait does not use return wrapping, thus is a typedef to `PhantomData`.
 *
 * Note that `cbindgen` will generate wrong structures for this type. It is important
 * to go inside the generated headers and fix it - all RetTmp structures without a
 * body should be completely deleted, both as types, and as fields in the
 * groups/objects. If C++11 templates are generated, it is important to define a
 * custom type for CGlueTraitObj that does not have `ret_tmp` defined, and change all
 * type aliases of this trait to use that particular structure.
 */
typedef struct VirtualTranslateRetTmp_CArc_c_void VirtualTranslateRetTmp_CArc_c_void;

/**
 * Type definition for temporary return value wrapping storage.
 *
 * The trait does not use return wrapping, thus is a typedef to `PhantomData`.
 *
 * Note that `cbindgen` will generate wrong structures for this type. It is important
 * to go inside the generated headers and fix it - all RetTmp structures without a
 * body should be completely deleted, both as types, and as fields in the
 * groups/objects. If C++11 templates are generated, it is important to define a
 * custom type for CGlueTraitObj that does not have `ret_tmp` defined, and change all
 * type aliases of this trait to use that particular structure.
 */
typedef struct VirtualTranslateRetTmp_Context VirtualTranslateRetTmp_Context;

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
    struct CloneRetTmp_CArc_c_void ret_tmp_clone;
    struct PhysicalMemoryRetTmp_CArc_c_void ret_tmp_physicalmemory;
    struct ConnectorCpuStateInnerRetTmp_CArc_c_void ret_tmp_connectorcpustateinner;
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
 * Generic type representing an address and associated data.
 *
 * This base type is always used for initialization, but the commonly used type aliases are:
 * `ReadData`, `WriteData`, `PhysicalReadData`, and `PhysicalWriteData`.
 */
typedef struct MemData_PhysicalAddress__CSliceMut_u8 {
    struct PhysicalAddress _0;
    struct CSliceMut_u8 _1;
} MemData_PhysicalAddress__CSliceMut_u8;

/**
 * MemData type for physical memory reads.
 */
typedef struct MemData_PhysicalAddress__CSliceMut_u8 PhysicalReadData;

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

typedef struct Callback_c_void__PhysicalReadData {
    void *context;
    bool (*func)(void*, PhysicalReadData);
} Callback_c_void__PhysicalReadData;

typedef struct Callback_c_void__PhysicalReadData OpaqueCallback_PhysicalReadData;

typedef OpaqueCallback_PhysicalReadData PhysicalReadFailCallback;

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
 * Generic type representing an address and associated data.
 *
 * This base type is always used for initialization, but the commonly used type aliases are:
 * `ReadData`, `WriteData`, `PhysicalReadData`, and `PhysicalWriteData`.
 */
typedef struct MemData_PhysicalAddress__CSliceRef_u8 {
    struct PhysicalAddress _0;
    struct CSliceRef_u8 _1;
} MemData_PhysicalAddress__CSliceRef_u8;

/**
 * MemData type for physical memory writes.
 */
typedef struct MemData_PhysicalAddress__CSliceRef_u8 PhysicalWriteData;

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

typedef struct Callback_c_void__PhysicalWriteData {
    void *context;
    bool (*func)(void*, PhysicalWriteData);
} Callback_c_void__PhysicalWriteData;

typedef struct Callback_c_void__PhysicalWriteData OpaqueCallback_PhysicalWriteData;

typedef OpaqueCallback_PhysicalWriteData PhysicalWriteFailCallback;

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
typedef struct CGlueObjContainer_CBox_c_void_____Context__MemoryViewRetTmp_Context {
    struct CBox_c_void instance;
    Context context;
    struct MemoryViewRetTmp_Context ret_tmp;
} CGlueObjContainer_CBox_c_void_____Context__MemoryViewRetTmp_Context;

/**
 * Generic type representing an address and associated data.
 *
 * This base type is always used for initialization, but the commonly used type aliases are:
 * `ReadData`, `WriteData`, `PhysicalReadData`, and `PhysicalWriteData`.
 */
typedef struct MemData_Address__CSliceMut_u8 {
    Address _0;
    struct CSliceMut_u8 _1;
} MemData_Address__CSliceMut_u8;

/**
 * MemData type for regular memory reads.
 */
typedef struct MemData_Address__CSliceMut_u8 ReadData;

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

typedef struct Callback_c_void__ReadData {
    void *context;
    bool (*func)(void*, ReadData);
} Callback_c_void__ReadData;

typedef struct Callback_c_void__ReadData OpaqueCallback_ReadData;

typedef OpaqueCallback_ReadData ReadFailCallback;

/**
 * Generic type representing an address and associated data.
 *
 * This base type is always used for initialization, but the commonly used type aliases are:
 * `ReadData`, `WriteData`, `PhysicalReadData`, and `PhysicalWriteData`.
 */
typedef struct MemData_Address__CSliceRef_u8 {
    Address _0;
    struct CSliceRef_u8 _1;
} MemData_Address__CSliceRef_u8;

/**
 * MemData type for regular memory writes.
 */
typedef struct MemData_Address__CSliceRef_u8 WriteData;

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

typedef struct Callback_c_void__WriteData {
    void *context;
    bool (*func)(void*, WriteData);
} Callback_c_void__WriteData;

typedef struct Callback_c_void__WriteData OpaqueCallback_WriteData;

typedef OpaqueCallback_WriteData WriteFailCallback;

typedef struct MemoryViewMetadata {
    Address max_address;
    umem real_size;
    bool readonly;
    bool little_endian;
    uint8_t arch_bits;
} MemoryViewMetadata;

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
 * CGlue vtable for trait MemoryView.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____Context__MemoryViewRetTmp_Context {
    int32_t (*read_raw_iter)(struct CGlueObjContainer_CBox_c_void_____Context__MemoryViewRetTmp_Context *cont, struct CIterator_ReadData data, ReadFailCallback *out_fail);
    int32_t (*write_raw_iter)(struct CGlueObjContainer_CBox_c_void_____Context__MemoryViewRetTmp_Context *cont, struct CIterator_WriteData data, WriteFailCallback *out_fail);
    struct MemoryViewMetadata (*metadata)(const struct CGlueObjContainer_CBox_c_void_____Context__MemoryViewRetTmp_Context *cont);
    int32_t (*read_raw_list)(struct CGlueObjContainer_CBox_c_void_____Context__MemoryViewRetTmp_Context *cont, struct CSliceMut_ReadData data);
    int32_t (*read_raw_into)(struct CGlueObjContainer_CBox_c_void_____Context__MemoryViewRetTmp_Context *cont, Address addr, struct CSliceMut_u8 out);
    int32_t (*write_raw_list)(struct CGlueObjContainer_CBox_c_void_____Context__MemoryViewRetTmp_Context *cont, struct CSliceRef_WriteData data);
    int32_t (*write_raw)(struct CGlueObjContainer_CBox_c_void_____Context__MemoryViewRetTmp_Context *cont, Address addr, struct CSliceRef_u8 data);
} MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____Context__MemoryViewRetTmp_Context;

/**
 * Simple CGlue trait object.
 *
 * This is the simplest form of CGlue object, represented by a container and vtable for a single
 * trait.
 *
 * Container merely is a this pointer with some optional temporary return reference context.
 */
typedef struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____Context__MemoryViewRetTmp_Context___________Context__MemoryViewRetTmp_Context {
    const struct MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____Context__MemoryViewRetTmp_Context *vtbl;
    struct CGlueObjContainer_CBox_c_void_____Context__MemoryViewRetTmp_Context container;
} CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____Context__MemoryViewRetTmp_Context___________Context__MemoryViewRetTmp_Context;

/**
 * Base CGlue trait object for trait MemoryView.
 */
typedef struct CGlueTraitObj_CBox_c_void_____MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____Context__MemoryViewRetTmp_Context___________Context__MemoryViewRetTmp_Context MemoryViewBase_CBox_c_void_____Context;

/**
 * CGlue vtable for trait PhysicalMemory.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct PhysicalMemoryVtbl_ConnectorInstanceContainer_CBox_c_void_____CArc_c_void {
    int32_t (*phys_read_raw_iter)(struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CIterator_PhysicalReadData data, PhysicalReadFailCallback *out_fail);
    int32_t (*phys_write_raw_iter)(struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CIterator_PhysicalWriteData data, PhysicalWriteFailCallback *out_fail);
    struct PhysicalMemoryMetadata (*metadata)(const struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void *cont);
    void (*set_mem_map)(struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CSliceRef_PhysicalMemoryMapping _mem_map);
    MemoryViewBase_CBox_c_void_____Context (*into_phys_view)(struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void cont);
    MemoryViewBase_CBox_c_void_____Context (*phys_view)(struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void *cont);
} PhysicalMemoryVtbl_ConnectorInstanceContainer_CBox_c_void_____CArc_c_void;

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
typedef struct CGlueObjContainer_CBox_c_void_____Context__CpuStateRetTmp_Context {
    struct CBox_c_void instance;
    Context context;
    struct CpuStateRetTmp_Context ret_tmp;
} CGlueObjContainer_CBox_c_void_____Context__CpuStateRetTmp_Context;

/**
 * CGlue vtable for trait CpuState.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct CpuStateVtbl_CGlueObjContainer_CBox_c_void_____Context__CpuStateRetTmp_Context {
    void (*pause)(struct CGlueObjContainer_CBox_c_void_____Context__CpuStateRetTmp_Context *cont);
    void (*resume)(struct CGlueObjContainer_CBox_c_void_____Context__CpuStateRetTmp_Context *cont);
} CpuStateVtbl_CGlueObjContainer_CBox_c_void_____Context__CpuStateRetTmp_Context;

/**
 * Simple CGlue trait object.
 *
 * This is the simplest form of CGlue object, represented by a container and vtable for a single
 * trait.
 *
 * Container merely is a this pointer with some optional temporary return reference context.
 */
typedef struct CGlueTraitObj_CBox_c_void_____CpuStateVtbl_CGlueObjContainer_CBox_c_void_____Context__CpuStateRetTmp_Context___________Context__CpuStateRetTmp_Context {
    const struct CpuStateVtbl_CGlueObjContainer_CBox_c_void_____Context__CpuStateRetTmp_Context *vtbl;
    struct CGlueObjContainer_CBox_c_void_____Context__CpuStateRetTmp_Context container;
} CGlueTraitObj_CBox_c_void_____CpuStateVtbl_CGlueObjContainer_CBox_c_void_____Context__CpuStateRetTmp_Context___________Context__CpuStateRetTmp_Context;

/**
 * Base CGlue trait object for trait CpuState.
 */
typedef struct CGlueTraitObj_CBox_c_void_____CpuStateVtbl_CGlueObjContainer_CBox_c_void_____Context__CpuStateRetTmp_Context___________Context__CpuStateRetTmp_Context CpuStateBase_CBox_c_void_____Context;

typedef struct IntoCpuStateContainer_CBox_c_void_____Context {
    struct CBox_c_void instance;
    Context context;
    struct CloneRetTmp_Context ret_tmp_clone;
    struct CpuStateRetTmp_Context ret_tmp_cpustate;
} IntoCpuStateContainer_CBox_c_void_____Context;

/**
 * CGlue vtable for trait Clone.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct CloneVtbl_IntoCpuStateContainer_CBox_c_void_____Context {
    struct IntoCpuStateContainer_CBox_c_void_____Context (*clone)(const struct IntoCpuStateContainer_CBox_c_void_____Context *cont);
} CloneVtbl_IntoCpuStateContainer_CBox_c_void_____Context;

/**
 * CGlue vtable for trait CpuState.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct CpuStateVtbl_IntoCpuStateContainer_CBox_c_void_____Context {
    void (*pause)(struct IntoCpuStateContainer_CBox_c_void_____Context *cont);
    void (*resume)(struct IntoCpuStateContainer_CBox_c_void_____Context *cont);
} CpuStateVtbl_IntoCpuStateContainer_CBox_c_void_____Context;

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
typedef struct IntoCpuState_CBox_c_void_____Context {
    const struct CloneVtbl_IntoCpuStateContainer_CBox_c_void_____Context *vtbl_clone;
    const struct CpuStateVtbl_IntoCpuStateContainer_CBox_c_void_____Context *vtbl_cpustate;
    struct IntoCpuStateContainer_CBox_c_void_____Context container;
} IntoCpuState_CBox_c_void_____Context;

/**
 * CGlue vtable for trait ConnectorCpuStateInner.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct ConnectorCpuStateInnerVtbl_ConnectorInstanceContainer_CBox_c_void_____CArc_c_void {
    int32_t (*cpu_state)(struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void *cont, CpuStateBase_CBox_c_void_____Context *ok_out);
    int32_t (*into_cpu_state)(struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void cont, struct IntoCpuState_CBox_c_void_____Context *ok_out);
} ConnectorCpuStateInnerVtbl_ConnectorInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * Trait group potentially implementing `:: cglue :: ext :: core :: clone :: Clone < > + PhysicalMemory < > + for < 'cglue_c > ConnectorCpuStateInner < 'cglue_c, >` traits.
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
    const struct ConnectorCpuStateInnerVtbl_ConnectorInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_connectorcpustateinner;
    struct ConnectorInstanceContainer_CBox_c_void_____CArc_c_void container;
} ConnectorInstance_CBox_c_void_____CArc_c_void;

typedef struct ConnectorInstance_CBox_c_void_____CArc_c_void ConnectorInstanceBaseCtxBox_c_void__CArc_c_void;

typedef ConnectorInstanceBaseCtxBox_c_void__CArc_c_void ConnectorInstanceBaseArcBox_c_void__c_void;

typedef ConnectorInstanceBaseArcBox_c_void__c_void ConnectorInstanceArcBox;

typedef ConnectorInstanceArcBox MuConnectorInstanceArcBox;

typedef struct OsInstanceContainer_CBox_c_void_____CArc_c_void {
    struct CBox_c_void instance;
    struct CArc_c_void context;
    struct CloneRetTmp_CArc_c_void ret_tmp_clone;
    struct OsInnerRetTmp_CArc_c_void ret_tmp_osinner;
    struct MemoryViewRetTmp_CArc_c_void ret_tmp_memoryview;
    struct OsKeyboardInnerRetTmp_CArc_c_void ret_tmp_oskeyboardinner;
    struct PhysicalMemoryRetTmp_CArc_c_void ret_tmp_physicalmemory;
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
} ProcessInfo;

typedef struct Callback_c_void__ProcessInfo {
    void *context;
    bool (*func)(void*, struct ProcessInfo);
} Callback_c_void__ProcessInfo;

typedef struct Callback_c_void__ProcessInfo OpaqueCallback_ProcessInfo;

typedef OpaqueCallback_ProcessInfo ProcessInfoCallback;

typedef struct ProcessInstanceContainer_CBox_c_void_____Context {
    struct CBox_c_void instance;
    Context context;
    struct MemoryViewRetTmp_Context ret_tmp_memoryview;
    struct ProcessRetTmp_Context ret_tmp_process;
    struct VirtualTranslateRetTmp_Context ret_tmp_virtualtranslate;
} ProcessInstanceContainer_CBox_c_void_____Context;

/**
 * CGlue vtable for trait MemoryView.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct MemoryViewVtbl_ProcessInstanceContainer_CBox_c_void_____Context {
    int32_t (*read_raw_iter)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, struct CIterator_ReadData data, ReadFailCallback *out_fail);
    int32_t (*write_raw_iter)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, struct CIterator_WriteData data, WriteFailCallback *out_fail);
    struct MemoryViewMetadata (*metadata)(const struct ProcessInstanceContainer_CBox_c_void_____Context *cont);
    int32_t (*read_raw_list)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, struct CSliceMut_ReadData data);
    int32_t (*read_raw_into)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, Address addr, struct CSliceMut_u8 out);
    int32_t (*write_raw_list)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, struct CSliceRef_WriteData data);
    int32_t (*write_raw)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, Address addr, struct CSliceRef_u8 data);
} MemoryViewVtbl_ProcessInstanceContainer_CBox_c_void_____Context;

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

/**
 * CGlue vtable for trait Process.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct ProcessVtbl_ProcessInstanceContainer_CBox_c_void_____Context {
    struct ProcessState (*state)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont);
    int32_t (*module_address_list_callback)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, const struct ArchitectureIdent *target_arch, ModuleAddressCallback callback);
    int32_t (*module_list_callback)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, const struct ArchitectureIdent *target_arch, ModuleInfoCallback callback);
    int32_t (*module_by_address)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, Address address, struct ArchitectureIdent architecture, struct ModuleInfo *ok_out);
    int32_t (*module_by_name_arch)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, struct CSliceRef_u8 name, const struct ArchitectureIdent *architecture, struct ModuleInfo *ok_out);
    int32_t (*module_by_name)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, struct CSliceRef_u8 name, struct ModuleInfo *ok_out);
    int32_t (*primary_module_address)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, Address *ok_out);
    int32_t (*primary_module)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, struct ModuleInfo *ok_out);
    int32_t (*module_import_list_callback)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, const struct ModuleInfo *info, ImportCallback callback);
    int32_t (*module_export_list_callback)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, const struct ModuleInfo *info, ExportCallback callback);
    int32_t (*module_section_list_callback)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, const struct ModuleInfo *info, SectionCallback callback);
    int32_t (*module_import_by_name)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, const struct ModuleInfo *info, struct CSliceRef_u8 name, struct ImportInfo *ok_out);
    int32_t (*module_export_by_name)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, const struct ModuleInfo *info, struct CSliceRef_u8 name, struct ExportInfo *ok_out);
    int32_t (*module_section_by_name)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, const struct ModuleInfo *info, struct CSliceRef_u8 name, struct SectionInfo *ok_out);
    const struct ProcessInfo *(*info)(const struct ProcessInstanceContainer_CBox_c_void_____Context *cont);
} ProcessVtbl_ProcessInstanceContainer_CBox_c_void_____Context;

/**
 * Virtual page range information used for callbacks
 */
typedef struct MemoryRange {
    Address address;
    umem size;
} MemoryRange;

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
typedef struct CSliceRef_MemoryRange {
    const struct MemoryRange *data;
    uintptr_t len;
} CSliceRef_MemoryRange;

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

typedef struct Callback_c_void__MemoryRange {
    void *context;
    bool (*func)(void*, struct MemoryRange);
} Callback_c_void__MemoryRange;

typedef struct Callback_c_void__MemoryRange OpaqueCallback_MemoryRange;

typedef OpaqueCallback_MemoryRange MemoryRangeCallback;

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
 * CGlue vtable for trait VirtualTranslate.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct VirtualTranslateVtbl_ProcessInstanceContainer_CBox_c_void_____Context {
    void (*virt_to_phys_list)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, struct CSliceRef_MemoryRange addrs, VirtualTranslationCallback out, VirtualTranslationFailCallback out_fail);
    void (*virt_to_phys_range)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, Address start, Address end, VirtualTranslationCallback out);
    void (*virt_translation_map_range)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, Address start, Address end, VirtualTranslationCallback out);
    void (*virt_page_map_range)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, umem gap_size, Address start, Address end, MemoryRangeCallback out);
    int32_t (*virt_to_phys)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, Address address, struct PhysicalAddress *ok_out);
    int32_t (*virt_page_info)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, Address addr, struct Page *ok_out);
    void (*virt_translation_map)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, VirtualTranslationCallback out);
    struct COption_Address (*phys_to_virt)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, Address phys);
    void (*virt_page_map)(struct ProcessInstanceContainer_CBox_c_void_____Context *cont, umem gap_size, MemoryRangeCallback out);
} VirtualTranslateVtbl_ProcessInstanceContainer_CBox_c_void_____Context;

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
typedef struct ProcessInstance_CBox_c_void_____Context {
    const struct MemoryViewVtbl_ProcessInstanceContainer_CBox_c_void_____Context *vtbl_memoryview;
    const struct ProcessVtbl_ProcessInstanceContainer_CBox_c_void_____Context *vtbl_process;
    const struct VirtualTranslateVtbl_ProcessInstanceContainer_CBox_c_void_____Context *vtbl_virtualtranslate;
    struct ProcessInstanceContainer_CBox_c_void_____Context container;
} ProcessInstance_CBox_c_void_____Context;

typedef struct IntoProcessInstanceContainer_CBox_c_void_____Context {
    struct CBox_c_void instance;
    Context context;
    struct CloneRetTmp_Context ret_tmp_clone;
    struct MemoryViewRetTmp_Context ret_tmp_memoryview;
    struct ProcessRetTmp_Context ret_tmp_process;
    struct VirtualTranslateRetTmp_Context ret_tmp_virtualtranslate;
} IntoProcessInstanceContainer_CBox_c_void_____Context;

/**
 * CGlue vtable for trait Clone.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct CloneVtbl_IntoProcessInstanceContainer_CBox_c_void_____Context {
    struct IntoProcessInstanceContainer_CBox_c_void_____Context (*clone)(const struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont);
} CloneVtbl_IntoProcessInstanceContainer_CBox_c_void_____Context;

/**
 * CGlue vtable for trait MemoryView.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct MemoryViewVtbl_IntoProcessInstanceContainer_CBox_c_void_____Context {
    int32_t (*read_raw_iter)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, struct CIterator_ReadData data, ReadFailCallback *out_fail);
    int32_t (*write_raw_iter)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, struct CIterator_WriteData data, WriteFailCallback *out_fail);
    struct MemoryViewMetadata (*metadata)(const struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont);
    int32_t (*read_raw_list)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, struct CSliceMut_ReadData data);
    int32_t (*read_raw_into)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, Address addr, struct CSliceMut_u8 out);
    int32_t (*write_raw_list)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, struct CSliceRef_WriteData data);
    int32_t (*write_raw)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, Address addr, struct CSliceRef_u8 data);
} MemoryViewVtbl_IntoProcessInstanceContainer_CBox_c_void_____Context;

/**
 * CGlue vtable for trait Process.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct ProcessVtbl_IntoProcessInstanceContainer_CBox_c_void_____Context {
    struct ProcessState (*state)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont);
    int32_t (*module_address_list_callback)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, const struct ArchitectureIdent *target_arch, ModuleAddressCallback callback);
    int32_t (*module_list_callback)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, const struct ArchitectureIdent *target_arch, ModuleInfoCallback callback);
    int32_t (*module_by_address)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, Address address, struct ArchitectureIdent architecture, struct ModuleInfo *ok_out);
    int32_t (*module_by_name_arch)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, struct CSliceRef_u8 name, const struct ArchitectureIdent *architecture, struct ModuleInfo *ok_out);
    int32_t (*module_by_name)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, struct CSliceRef_u8 name, struct ModuleInfo *ok_out);
    int32_t (*primary_module_address)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, Address *ok_out);
    int32_t (*primary_module)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, struct ModuleInfo *ok_out);
    int32_t (*module_import_list_callback)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, const struct ModuleInfo *info, ImportCallback callback);
    int32_t (*module_export_list_callback)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, const struct ModuleInfo *info, ExportCallback callback);
    int32_t (*module_section_list_callback)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, const struct ModuleInfo *info, SectionCallback callback);
    int32_t (*module_import_by_name)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, const struct ModuleInfo *info, struct CSliceRef_u8 name, struct ImportInfo *ok_out);
    int32_t (*module_export_by_name)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, const struct ModuleInfo *info, struct CSliceRef_u8 name, struct ExportInfo *ok_out);
    int32_t (*module_section_by_name)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, const struct ModuleInfo *info, struct CSliceRef_u8 name, struct SectionInfo *ok_out);
    const struct ProcessInfo *(*info)(const struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont);
} ProcessVtbl_IntoProcessInstanceContainer_CBox_c_void_____Context;

/**
 * CGlue vtable for trait VirtualTranslate.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct VirtualTranslateVtbl_IntoProcessInstanceContainer_CBox_c_void_____Context {
    void (*virt_to_phys_list)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, struct CSliceRef_MemoryRange addrs, VirtualTranslationCallback out, VirtualTranslationFailCallback out_fail);
    void (*virt_to_phys_range)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, Address start, Address end, VirtualTranslationCallback out);
    void (*virt_translation_map_range)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, Address start, Address end, VirtualTranslationCallback out);
    void (*virt_page_map_range)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, umem gap_size, Address start, Address end, MemoryRangeCallback out);
    int32_t (*virt_to_phys)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, Address address, struct PhysicalAddress *ok_out);
    int32_t (*virt_page_info)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, Address addr, struct Page *ok_out);
    void (*virt_translation_map)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, VirtualTranslationCallback out);
    struct COption_Address (*phys_to_virt)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, Address phys);
    void (*virt_page_map)(struct IntoProcessInstanceContainer_CBox_c_void_____Context *cont, umem gap_size, MemoryRangeCallback out);
} VirtualTranslateVtbl_IntoProcessInstanceContainer_CBox_c_void_____Context;

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
typedef struct IntoProcessInstance_CBox_c_void_____Context {
    const struct CloneVtbl_IntoProcessInstanceContainer_CBox_c_void_____Context *vtbl_clone;
    const struct MemoryViewVtbl_IntoProcessInstanceContainer_CBox_c_void_____Context *vtbl_memoryview;
    const struct ProcessVtbl_IntoProcessInstanceContainer_CBox_c_void_____Context *vtbl_process;
    const struct VirtualTranslateVtbl_IntoProcessInstanceContainer_CBox_c_void_____Context *vtbl_virtualtranslate;
    struct IntoProcessInstanceContainer_CBox_c_void_____Context container;
} IntoProcessInstance_CBox_c_void_____Context;

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
 * CGlue vtable for trait OsInner.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct OsInnerVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void {
    int32_t (*process_address_list_callback)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, AddressCallback callback);
    int32_t (*process_info_list_callback)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, ProcessInfoCallback callback);
    int32_t (*process_info_by_address)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address address, struct ProcessInfo *ok_out);
    int32_t (*process_info_by_name)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CSliceRef_u8 name, struct ProcessInfo *ok_out);
    int32_t (*process_info_by_pid)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, Pid pid, struct ProcessInfo *ok_out);
    int32_t (*process_by_info)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct ProcessInfo info, struct ProcessInstance_CBox_c_void_____Context *ok_out);
    int32_t (*into_process_by_info)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void cont, struct ProcessInfo info, struct IntoProcessInstance_CBox_c_void_____Context *ok_out);
    int32_t (*process_by_address)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address addr, struct ProcessInstance_CBox_c_void_____Context *ok_out);
    int32_t (*process_by_name)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CSliceRef_u8 name, struct ProcessInstance_CBox_c_void_____Context *ok_out);
    int32_t (*process_by_pid)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, Pid pid, struct ProcessInstance_CBox_c_void_____Context *ok_out);
    int32_t (*into_process_by_address)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void cont, Address addr, struct IntoProcessInstance_CBox_c_void_____Context *ok_out);
    int32_t (*into_process_by_name)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void cont, struct CSliceRef_u8 name, struct IntoProcessInstance_CBox_c_void_____Context *ok_out);
    int32_t (*into_process_by_pid)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void cont, Pid pid, struct IntoProcessInstance_CBox_c_void_____Context *ok_out);
    int32_t (*module_address_list_callback)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, AddressCallback callback);
    int32_t (*module_list_callback)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, ModuleInfoCallback callback);
    int32_t (*module_by_address)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address address, struct ModuleInfo *ok_out);
    int32_t (*module_by_name)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CSliceRef_u8 name, struct ModuleInfo *ok_out);
    const struct OsInfo *(*info)(const struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont);
} OsInnerVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait MemoryView.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct MemoryViewVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void {
    int32_t (*read_raw_iter)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CIterator_ReadData data, ReadFailCallback *out_fail);
    int32_t (*write_raw_iter)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CIterator_WriteData data, WriteFailCallback *out_fail);
    struct MemoryViewMetadata (*metadata)(const struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont);
    int32_t (*read_raw_list)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CSliceMut_ReadData data);
    int32_t (*read_raw_into)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address addr, struct CSliceMut_u8 out);
    int32_t (*write_raw_list)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CSliceRef_WriteData data);
    int32_t (*write_raw)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address addr, struct CSliceRef_u8 data);
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
typedef struct CGlueObjContainer_CBox_c_void_____Context__KeyboardRetTmp_Context {
    struct CBox_c_void instance;
    Context context;
    struct KeyboardRetTmp_Context ret_tmp;
} CGlueObjContainer_CBox_c_void_____Context__KeyboardRetTmp_Context;

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
typedef struct CGlueObjContainer_CBox_c_void_____Context__KeyboardStateRetTmp_Context {
    struct CBox_c_void instance;
    Context context;
    struct KeyboardStateRetTmp_Context ret_tmp;
} CGlueObjContainer_CBox_c_void_____Context__KeyboardStateRetTmp_Context;

/**
 * CGlue vtable for trait KeyboardState.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct KeyboardStateVtbl_CGlueObjContainer_CBox_c_void_____Context__KeyboardStateRetTmp_Context {
    bool (*is_down)(const struct CGlueObjContainer_CBox_c_void_____Context__KeyboardStateRetTmp_Context *cont, int32_t vk);
} KeyboardStateVtbl_CGlueObjContainer_CBox_c_void_____Context__KeyboardStateRetTmp_Context;

/**
 * Simple CGlue trait object.
 *
 * This is the simplest form of CGlue object, represented by a container and vtable for a single
 * trait.
 *
 * Container merely is a this pointer with some optional temporary return reference context.
 */
typedef struct CGlueTraitObj_CBox_c_void_____KeyboardStateVtbl_CGlueObjContainer_CBox_c_void_____Context__KeyboardStateRetTmp_Context___________Context__KeyboardStateRetTmp_Context {
    const struct KeyboardStateVtbl_CGlueObjContainer_CBox_c_void_____Context__KeyboardStateRetTmp_Context *vtbl;
    struct CGlueObjContainer_CBox_c_void_____Context__KeyboardStateRetTmp_Context container;
} CGlueTraitObj_CBox_c_void_____KeyboardStateVtbl_CGlueObjContainer_CBox_c_void_____Context__KeyboardStateRetTmp_Context___________Context__KeyboardStateRetTmp_Context;

/**
 * Base CGlue trait object for trait KeyboardState.
 */
typedef struct CGlueTraitObj_CBox_c_void_____KeyboardStateVtbl_CGlueObjContainer_CBox_c_void_____Context__KeyboardStateRetTmp_Context___________Context__KeyboardStateRetTmp_Context KeyboardStateBase_CBox_c_void_____Context;

/**
 * CGlue vtable for trait Keyboard.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct KeyboardVtbl_CGlueObjContainer_CBox_c_void_____Context__KeyboardRetTmp_Context {
    bool (*is_down)(struct CGlueObjContainer_CBox_c_void_____Context__KeyboardRetTmp_Context *cont, int32_t vk);
    void (*set_down)(struct CGlueObjContainer_CBox_c_void_____Context__KeyboardRetTmp_Context *cont, int32_t vk, bool down);
    int32_t (*state)(struct CGlueObjContainer_CBox_c_void_____Context__KeyboardRetTmp_Context *cont, KeyboardStateBase_CBox_c_void_____Context *ok_out);
} KeyboardVtbl_CGlueObjContainer_CBox_c_void_____Context__KeyboardRetTmp_Context;

/**
 * Simple CGlue trait object.
 *
 * This is the simplest form of CGlue object, represented by a container and vtable for a single
 * trait.
 *
 * Container merely is a this pointer with some optional temporary return reference context.
 */
typedef struct CGlueTraitObj_CBox_c_void_____KeyboardVtbl_CGlueObjContainer_CBox_c_void_____Context__KeyboardRetTmp_Context___________Context__KeyboardRetTmp_Context {
    const struct KeyboardVtbl_CGlueObjContainer_CBox_c_void_____Context__KeyboardRetTmp_Context *vtbl;
    struct CGlueObjContainer_CBox_c_void_____Context__KeyboardRetTmp_Context container;
} CGlueTraitObj_CBox_c_void_____KeyboardVtbl_CGlueObjContainer_CBox_c_void_____Context__KeyboardRetTmp_Context___________Context__KeyboardRetTmp_Context;

/**
 * Base CGlue trait object for trait Keyboard.
 */
typedef struct CGlueTraitObj_CBox_c_void_____KeyboardVtbl_CGlueObjContainer_CBox_c_void_____Context__KeyboardRetTmp_Context___________Context__KeyboardRetTmp_Context KeyboardBase_CBox_c_void_____Context;

typedef struct IntoKeyboardContainer_CBox_c_void_____Context {
    struct CBox_c_void instance;
    Context context;
    struct CloneRetTmp_Context ret_tmp_clone;
    struct KeyboardRetTmp_Context ret_tmp_keyboard;
} IntoKeyboardContainer_CBox_c_void_____Context;

/**
 * CGlue vtable for trait Clone.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct CloneVtbl_IntoKeyboardContainer_CBox_c_void_____Context {
    struct IntoKeyboardContainer_CBox_c_void_____Context (*clone)(const struct IntoKeyboardContainer_CBox_c_void_____Context *cont);
} CloneVtbl_IntoKeyboardContainer_CBox_c_void_____Context;

/**
 * CGlue vtable for trait Keyboard.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct KeyboardVtbl_IntoKeyboardContainer_CBox_c_void_____Context {
    bool (*is_down)(struct IntoKeyboardContainer_CBox_c_void_____Context *cont, int32_t vk);
    void (*set_down)(struct IntoKeyboardContainer_CBox_c_void_____Context *cont, int32_t vk, bool down);
    int32_t (*state)(struct IntoKeyboardContainer_CBox_c_void_____Context *cont, KeyboardStateBase_CBox_c_void_____Context *ok_out);
} KeyboardVtbl_IntoKeyboardContainer_CBox_c_void_____Context;

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
typedef struct IntoKeyboard_CBox_c_void_____Context {
    const struct CloneVtbl_IntoKeyboardContainer_CBox_c_void_____Context *vtbl_clone;
    const struct KeyboardVtbl_IntoKeyboardContainer_CBox_c_void_____Context *vtbl_keyboard;
    struct IntoKeyboardContainer_CBox_c_void_____Context container;
} IntoKeyboard_CBox_c_void_____Context;

/**
 * CGlue vtable for trait OsKeyboardInner.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct OsKeyboardInnerVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void {
    int32_t (*keyboard)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, KeyboardBase_CBox_c_void_____Context *ok_out);
    int32_t (*into_keyboard)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void cont, struct IntoKeyboard_CBox_c_void_____Context *ok_out);
} OsKeyboardInnerVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait PhysicalMemory.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct PhysicalMemoryVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void {
    int32_t (*phys_read_raw_iter)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CIterator_PhysicalReadData data, PhysicalReadFailCallback *out_fail);
    int32_t (*phys_write_raw_iter)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CIterator_PhysicalWriteData data, PhysicalWriteFailCallback *out_fail);
    struct PhysicalMemoryMetadata (*metadata)(const struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont);
    void (*set_mem_map)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CSliceRef_PhysicalMemoryMapping _mem_map);
    MemoryViewBase_CBox_c_void_____Context (*into_phys_view)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void cont);
    MemoryViewBase_CBox_c_void_____Context (*phys_view)(struct OsInstanceContainer_CBox_c_void_____CArc_c_void *cont);
} PhysicalMemoryVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * Trait group potentially implementing `:: cglue :: ext :: core :: clone :: Clone < > + for < 'cglue_c > OsInner < 'cglue_c, > + MemoryView < > + for < 'cglue_c > OsKeyboardInner < 'cglue_c, > + PhysicalMemory < >` traits.
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
    const struct OsInnerVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_osinner;
    const struct MemoryViewVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_memoryview;
    const struct OsKeyboardInnerVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_oskeyboardinner;
    const struct PhysicalMemoryVtbl_OsInstanceContainer_CBox_c_void_____CArc_c_void *vtbl_physicalmemory;
    struct OsInstanceContainer_CBox_c_void_____CArc_c_void container;
} OsInstance_CBox_c_void_____CArc_c_void;

typedef struct OsInstance_CBox_c_void_____CArc_c_void OsInstanceBaseCtxBox_c_void__CArc_c_void;

typedef OsInstanceBaseCtxBox_c_void__CArc_c_void OsInstanceBaseArcBox_c_void__c_void;

typedef OsInstanceBaseArcBox_c_void__c_void OsInstanceArcBox;

typedef OsInstanceArcBox MuOsInstanceArcBox;

typedef struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void {
    struct CBox_c_void instance;
    struct CArc_c_void context;
    struct MemoryViewRetTmp_CArc_c_void ret_tmp_memoryview;
    struct ProcessRetTmp_CArc_c_void ret_tmp_process;
    struct VirtualTranslateRetTmp_CArc_c_void ret_tmp_virtualtranslate;
} ProcessInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait MemoryView.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct MemoryViewVtbl_ProcessInstanceContainer_CBox_c_void_____CArc_c_void {
    int32_t (*read_raw_iter)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CIterator_ReadData data, ReadFailCallback *out_fail);
    int32_t (*write_raw_iter)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CIterator_WriteData data, WriteFailCallback *out_fail);
    struct MemoryViewMetadata (*metadata)(const struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont);
    int32_t (*read_raw_list)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CSliceMut_ReadData data);
    int32_t (*read_raw_into)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address addr, struct CSliceMut_u8 out);
    int32_t (*write_raw_list)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CSliceRef_WriteData data);
    int32_t (*write_raw)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address addr, struct CSliceRef_u8 data);
} MemoryViewVtbl_ProcessInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait Process.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct ProcessVtbl_ProcessInstanceContainer_CBox_c_void_____CArc_c_void {
    struct ProcessState (*state)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont);
    int32_t (*module_address_list_callback)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, const struct ArchitectureIdent *target_arch, ModuleAddressCallback callback);
    int32_t (*module_list_callback)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, const struct ArchitectureIdent *target_arch, ModuleInfoCallback callback);
    int32_t (*module_by_address)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address address, struct ArchitectureIdent architecture, struct ModuleInfo *ok_out);
    int32_t (*module_by_name_arch)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CSliceRef_u8 name, const struct ArchitectureIdent *architecture, struct ModuleInfo *ok_out);
    int32_t (*module_by_name)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CSliceRef_u8 name, struct ModuleInfo *ok_out);
    int32_t (*primary_module_address)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address *ok_out);
    int32_t (*primary_module)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct ModuleInfo *ok_out);
    int32_t (*module_import_list_callback)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, const struct ModuleInfo *info, ImportCallback callback);
    int32_t (*module_export_list_callback)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, const struct ModuleInfo *info, ExportCallback callback);
    int32_t (*module_section_list_callback)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, const struct ModuleInfo *info, SectionCallback callback);
    int32_t (*module_import_by_name)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, const struct ModuleInfo *info, struct CSliceRef_u8 name, struct ImportInfo *ok_out);
    int32_t (*module_export_by_name)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, const struct ModuleInfo *info, struct CSliceRef_u8 name, struct ExportInfo *ok_out);
    int32_t (*module_section_by_name)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, const struct ModuleInfo *info, struct CSliceRef_u8 name, struct SectionInfo *ok_out);
    const struct ProcessInfo *(*info)(const struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont);
} ProcessVtbl_ProcessInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait VirtualTranslate.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct VirtualTranslateVtbl_ProcessInstanceContainer_CBox_c_void_____CArc_c_void {
    void (*virt_to_phys_list)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CSliceRef_MemoryRange addrs, VirtualTranslationCallback out, VirtualTranslationFailCallback out_fail);
    void (*virt_to_phys_range)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address start, Address end, VirtualTranslationCallback out);
    void (*virt_translation_map_range)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address start, Address end, VirtualTranslationCallback out);
    void (*virt_page_map_range)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, umem gap_size, Address start, Address end, MemoryRangeCallback out);
    int32_t (*virt_to_phys)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address address, struct PhysicalAddress *ok_out);
    int32_t (*virt_page_info)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address addr, struct Page *ok_out);
    void (*virt_translation_map)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, VirtualTranslationCallback out);
    struct COption_Address (*phys_to_virt)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address phys);
    void (*virt_page_map)(struct ProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, umem gap_size, MemoryRangeCallback out);
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

typedef struct ProcessInstance_CBox_c_void_____CArc_c_void ProcessInstanceBaseCtxBox_c_void__CArc_c_void;

typedef ProcessInstanceBaseCtxBox_c_void__CArc_c_void ProcessInstanceBaseArcBox_c_void__c_void;

typedef ProcessInstanceBaseArcBox_c_void__c_void ProcessInstanceArcBox;

typedef struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void {
    struct CBox_c_void instance;
    struct CArc_c_void context;
    struct CloneRetTmp_CArc_c_void ret_tmp_clone;
    struct MemoryViewRetTmp_CArc_c_void ret_tmp_memoryview;
    struct ProcessRetTmp_CArc_c_void ret_tmp_process;
    struct VirtualTranslateRetTmp_CArc_c_void ret_tmp_virtualtranslate;
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
    int32_t (*read_raw_iter)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CIterator_ReadData data, ReadFailCallback *out_fail);
    int32_t (*write_raw_iter)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CIterator_WriteData data, WriteFailCallback *out_fail);
    struct MemoryViewMetadata (*metadata)(const struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont);
    int32_t (*read_raw_list)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CSliceMut_ReadData data);
    int32_t (*read_raw_into)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address addr, struct CSliceMut_u8 out);
    int32_t (*write_raw_list)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CSliceRef_WriteData data);
    int32_t (*write_raw)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address addr, struct CSliceRef_u8 data);
} MemoryViewVtbl_IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait Process.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct ProcessVtbl_IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void {
    struct ProcessState (*state)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont);
    int32_t (*module_address_list_callback)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, const struct ArchitectureIdent *target_arch, ModuleAddressCallback callback);
    int32_t (*module_list_callback)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, const struct ArchitectureIdent *target_arch, ModuleInfoCallback callback);
    int32_t (*module_by_address)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address address, struct ArchitectureIdent architecture, struct ModuleInfo *ok_out);
    int32_t (*module_by_name_arch)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CSliceRef_u8 name, const struct ArchitectureIdent *architecture, struct ModuleInfo *ok_out);
    int32_t (*module_by_name)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CSliceRef_u8 name, struct ModuleInfo *ok_out);
    int32_t (*primary_module_address)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address *ok_out);
    int32_t (*primary_module)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct ModuleInfo *ok_out);
    int32_t (*module_import_list_callback)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, const struct ModuleInfo *info, ImportCallback callback);
    int32_t (*module_export_list_callback)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, const struct ModuleInfo *info, ExportCallback callback);
    int32_t (*module_section_list_callback)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, const struct ModuleInfo *info, SectionCallback callback);
    int32_t (*module_import_by_name)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, const struct ModuleInfo *info, struct CSliceRef_u8 name, struct ImportInfo *ok_out);
    int32_t (*module_export_by_name)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, const struct ModuleInfo *info, struct CSliceRef_u8 name, struct ExportInfo *ok_out);
    int32_t (*module_section_by_name)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, const struct ModuleInfo *info, struct CSliceRef_u8 name, struct SectionInfo *ok_out);
    const struct ProcessInfo *(*info)(const struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont);
} ProcessVtbl_IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void;

/**
 * CGlue vtable for trait VirtualTranslate.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct VirtualTranslateVtbl_IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void {
    void (*virt_to_phys_list)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, struct CSliceRef_MemoryRange addrs, VirtualTranslationCallback out, VirtualTranslationFailCallback out_fail);
    void (*virt_to_phys_range)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address start, Address end, VirtualTranslationCallback out);
    void (*virt_translation_map_range)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address start, Address end, VirtualTranslationCallback out);
    void (*virt_page_map_range)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, umem gap_size, Address start, Address end, MemoryRangeCallback out);
    int32_t (*virt_to_phys)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address address, struct PhysicalAddress *ok_out);
    int32_t (*virt_page_info)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address addr, struct Page *ok_out);
    void (*virt_translation_map)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, VirtualTranslationCallback out);
    struct COption_Address (*phys_to_virt)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, Address phys);
    void (*virt_page_map)(struct IntoProcessInstanceContainer_CBox_c_void_____CArc_c_void *cont, umem gap_size, MemoryRangeCallback out);
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
    struct MemoryViewRetTmp_CArc_c_void ret_tmp;
} CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void;

/**
 * CGlue vtable for trait MemoryView.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
typedef struct MemoryViewVtbl_CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void {
    int32_t (*read_raw_iter)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void *cont, struct CIterator_ReadData data, ReadFailCallback *out_fail);
    int32_t (*write_raw_iter)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void *cont, struct CIterator_WriteData data, WriteFailCallback *out_fail);
    struct MemoryViewMetadata (*metadata)(const struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void *cont);
    int32_t (*read_raw_list)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void *cont, struct CSliceMut_ReadData data);
    int32_t (*read_raw_into)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void *cont, Address addr, struct CSliceMut_u8 out);
    int32_t (*write_raw_list)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void *cont, struct CSliceRef_WriteData data);
    int32_t (*write_raw)(struct CGlueObjContainer_CBox_c_void_____CArc_c_void_____MemoryViewRetTmp_CArc_c_void *cont, Address addr, struct CSliceRef_u8 data);
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

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

extern const struct ArchitectureObj *X86_32;

extern const struct ArchitectureObj *X86_32_PAE;

extern const struct ArchitectureObj *X86_64;

void log_init(int32_t level_num);

void debug_error(int32_t error);

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
 * `conn` has to point to a valid [`ConnectorInstance`] created by one of the provided
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

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* MEMFLOW_H */