#ifndef MEMFLOW_H
#define MEMFLOW_H

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>
typedef void *Library;

/**
 * Identifies the byte order of a architecture
 *
 * This enum is used when reading/writing to/from the memory of a target system.
 * The memory will be automatically converted to the endianess memflow is currently running on.
 *
 * See the [wikipedia article](https://en.wikipedia.org/wiki/Endianness) for more information on the subject.
 */
enum class Endianess : uint8_t {
    /**
     * Little Endianess
     */
    Endianess_LittleEndian,
    /**
     * Big Endianess
     */
    Endianess_BigEndian,
};

struct ArchitectureObj;

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
template<typename CGlueCtx = void>
typedef void CloneRetTmp;

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
template<typename CGlueCtx = void>
typedef void ConnectorCpuStateInnerRetTmp;

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
template<typename CGlueCtx = void>
typedef void CpuStateRetTmp;

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
struct Inventory;

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
template<typename CGlueCtx = void>
typedef void KeyboardRetTmp;

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
template<typename CGlueCtx = void>
typedef void KeyboardStateRetTmp;

template<typename T = void>
using MaybeUninit = T;

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
template<typename CGlueCtx = void>
typedef void MemoryViewRetTmp;

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
template<typename CGlueCtx = void>
typedef void OsInnerRetTmp;

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
template<typename CGlueCtx = void>
typedef void OsKeyboardInnerRetTmp;

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
template<typename CGlueCtx = void>
typedef void PhysicalMemoryRetTmp;

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
template<typename CGlueCtx = void>
typedef void ProcessRetTmp;

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
template<typename CGlueCtx = void>
typedef void VirtualTranslateRetTmp;

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
using umem = uint64_t;

/**
 * This type represents a address on the target system.
 * It internally holds a `umem` value but can also be used
 * when working in 32-bit environments.
 *
 * This type will not handle overflow for 32-bit or 64-bit addresses / lengths.
 */
using Address = umem;
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
static const Address Address_NULL = 0;

/**
 * Describes the type of a page using a bitflag.
 */
using PageType = uint8_t;
/**
 * The page explicitly has no flags.
 */
static const PageType PageType_NONE = 0;
/**
 * The page type is not known.
 */
static const PageType PageType_UNKNOWN = 1;
/**
 * The page contains page table entries.
 */
static const PageType PageType_PAGE_TABLE = 2;
/**
 * The page is a writeable page.
 */
static const PageType PageType_WRITEABLE = 4;
/**
 * The page is read only.
 */
static const PageType PageType_READ_ONLY = 8;
/**
 * The page is not executable.
 */
static const PageType PageType_NOEXEC = 16;

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
struct PhysicalAddress {
    Address address;
    PageType page_type;
    uint8_t page_size_log2;
};

/**
 * FFI-Safe Arc
 *
 * This is an FFI-Safe equivalent of Arc<T> and Option<Arc<T>>.
 */
template<typename T>
struct CArc {
    const T *instance;
    const T *(*clone_fn)(const T*);
    void (*drop_fn)(const T*);
};

/**
 * FFI-safe box
 *
 * This box has a static self reference, alongside a custom drop function.
 *
 * The drop function can be called from anywhere, it will free on correct allocator internally.
 */
template<typename T>
struct CBox {
    T *instance;
    void (*drop_fn)(T*);
};

template<typename CGlueInst, typename CGlueCtx>
struct ConnectorInstanceContainer {
    CGlueInst instance;
    CGlueCtx context;
    CloneRetTmp<CGlueCtx> ret_tmp_clone;
    PhysicalMemoryRetTmp<CGlueCtx> ret_tmp_physicalmemory;
    ConnectorCpuStateInnerRetTmp<CGlueCtx> ret_tmp_connectorcpustateinner;
};

/**
 * CGlue vtable for trait Clone.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct CloneVtbl {
    CGlueC (*clone)(const CGlueC *cont);
};

/**
 * Wrapper around mutable slices.
 *
 * This is meant as a safe type to pass across the FFI boundary with similar semantics as regular
 * slice. However, not all functionality is present, use the slice conversion functions.
 */
template<typename T>
struct CSliceMut {
    T *data;
    uintptr_t len;
};

/**
 * Generic type representing an address and associated data.
 *
 * This base type is always used for initialization, but the commonly used type aliases are:
 * `ReadData`, `WriteData`, `PhysicalReadData`, and `PhysicalWriteData`.
 */
template<typename A, typename T>
struct MemData {
    A _0;
    T _1;
};

/**
 * MemData type for physical memory reads.
 */
using PhysicalReadData = MemData<PhysicalAddress, CSliceMut<uint8_t>>;

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
template<typename T>
struct CIterator {
    void *iter;
    int32_t (*func)(void*, MaybeUninit<T> *out);
};

template<typename T, typename F>
struct Callback {
    T *context;
    bool (*func)(T*, F);
};

template<typename T>
using OpaqueCallback = Callback<void, T>;

using PhysicalReadFailCallback = OpaqueCallback<PhysicalReadData>;

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
template<typename T>
struct CSliceRef {
    const T *data;
    uintptr_t len;
};

/**
 * MemData type for physical memory writes.
 */
using PhysicalWriteData = MemData<PhysicalAddress, CSliceRef<uint8_t>>;

using PhysicalWriteFailCallback = OpaqueCallback<PhysicalWriteData>;

struct PhysicalMemoryMetadata {
    Address max_address;
    umem real_size;
    bool readonly;
    uint32_t ideal_batch_size;
};

struct PhysicalMemoryMapping {
    Address base;
    umem size;
    Address real_base;
};

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
template<typename T, typename C, typename R>
struct CGlueObjContainer {
    T instance;
    C context;
    R ret_tmp;
};

/**
 * MemData type for regular memory reads.
 */
using ReadData = MemData<Address, CSliceMut<uint8_t>>;

using ReadFailCallback = OpaqueCallback<ReadData>;

/**
 * MemData type for regular memory writes.
 */
using WriteData = MemData<Address, CSliceRef<uint8_t>>;

using WriteFailCallback = OpaqueCallback<WriteData>;

struct MemoryViewMetadata {
    Address max_address;
    umem real_size;
    bool readonly;
    bool little_endian;
    uint8_t arch_bits;
};

/**
 * CGlue vtable for trait MemoryView.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct MemoryViewVtbl {
    int32_t (*read_raw_iter)(CGlueC *cont, CIterator<ReadData> data, ReadFailCallback *out_fail);
    int32_t (*write_raw_iter)(CGlueC *cont, CIterator<WriteData> data, WriteFailCallback *out_fail);
    MemoryViewMetadata (*metadata)(const CGlueC *cont);
    int32_t (*read_raw_list)(CGlueC *cont, CSliceMut<ReadData> data);
    int32_t (*read_raw_into)(CGlueC *cont, Address addr, CSliceMut<uint8_t> out);
    int32_t (*write_raw_list)(CGlueC *cont, CSliceRef<WriteData> data);
    int32_t (*write_raw)(CGlueC *cont, Address addr, CSliceRef<uint8_t> data);
};

/**
 * Simple CGlue trait object.
 *
 * This is the simplest form of CGlue object, represented by a container and vtable for a single
 * trait.
 *
 * Container merely is a this pointer with some optional temporary return reference context.
 */
template<typename T, typename V, typename C, typename R>
struct CGlueTraitObj {
    const V *vtbl;
    CGlueObjContainer<T, C, R> container;
};

/**
 * Base CGlue trait object for trait MemoryView.
 */
template<typename CGlueInst, typename CGlueCtx>
using MemoryViewBase = CGlueTraitObj<CGlueInst, MemoryViewVtbl<CGlueObjContainer<CGlueInst, CGlueCtx, MemoryViewRetTmp<CGlueCtx>>>, CGlueCtx, MemoryViewRetTmp<CGlueCtx>>;

/**
 * CGlue vtable for trait PhysicalMemory.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct PhysicalMemoryVtbl {
    int32_t (*phys_read_raw_iter)(CGlueC *cont, CIterator<PhysicalReadData> data, PhysicalReadFailCallback *out_fail);
    int32_t (*phys_write_raw_iter)(CGlueC *cont, CIterator<PhysicalWriteData> data, PhysicalWriteFailCallback *out_fail);
    PhysicalMemoryMetadata (*metadata)(const CGlueC *cont);
    void (*set_mem_map)(CGlueC *cont, CSliceRef<PhysicalMemoryMapping> _mem_map);
    MemoryViewBase<CBox<void>, Context> (*into_phys_view)(CGlueC cont);
    MemoryViewBase<CBox<void>, Context> (*phys_view)(CGlueC *cont);
};

/**
 * CGlue vtable for trait CpuState.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct CpuStateVtbl {
    void (*pause)(CGlueC *cont);
    void (*resume)(CGlueC *cont);
};

/**
 * Base CGlue trait object for trait CpuState.
 */
template<typename CGlueInst, typename CGlueCtx>
using CpuStateBase = CGlueTraitObj<CGlueInst, CpuStateVtbl<CGlueObjContainer<CGlueInst, CGlueCtx, CpuStateRetTmp<CGlueCtx>>>, CGlueCtx, CpuStateRetTmp<CGlueCtx>>;

template<typename CGlueInst, typename CGlueCtx>
struct IntoCpuStateContainer {
    CGlueInst instance;
    CGlueCtx context;
    CloneRetTmp<CGlueCtx> ret_tmp_clone;
    CpuStateRetTmp<CGlueCtx> ret_tmp_cpustate;
};

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
template<typename CGlueInst, typename CGlueCtx>
struct IntoCpuState {
    const CloneVtbl<IntoCpuStateContainer<CGlueInst, CGlueCtx>> *vtbl_clone;
    const CpuStateVtbl<IntoCpuStateContainer<CGlueInst, CGlueCtx>> *vtbl_cpustate;
    IntoCpuStateContainer<CGlueInst, CGlueCtx> container;
};

/**
 * CGlue vtable for trait ConnectorCpuStateInner.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct ConnectorCpuStateInnerVtbl {
    int32_t (*cpu_state)(CGlueC *cont, MaybeUninit<CpuStateBase<CBox<void>, Context>> *ok_out);
    int32_t (*into_cpu_state)(CGlueC cont, MaybeUninit<IntoCpuState<CBox<void>, Context>> *ok_out);
};

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
template<typename CGlueInst, typename CGlueCtx>
struct ConnectorInstance {
    const CloneVtbl<ConnectorInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_clone;
    const PhysicalMemoryVtbl<ConnectorInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_physicalmemory;
    const ConnectorCpuStateInnerVtbl<ConnectorInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_connectorcpustateinner;
    ConnectorInstanceContainer<CGlueInst, CGlueCtx> container;
};

template<typename CGlueT, typename CGlueCtx>
using ConnectorInstanceBaseCtxBox = ConnectorInstance<CBox<CGlueT>, CGlueCtx>;

template<typename CGlueT, typename CGlueArcTy>
using ConnectorInstanceBaseArcBox = ConnectorInstanceBaseCtxBox<CGlueT, CArc<CGlueArcTy>>;

using ConnectorInstanceArcBox = ConnectorInstanceBaseArcBox<void, void>;

using MuConnectorInstanceArcBox = MaybeUninit<ConnectorInstanceArcBox>;

template<typename CGlueInst, typename CGlueCtx>
struct OsInstanceContainer {
    CGlueInst instance;
    CGlueCtx context;
    CloneRetTmp<CGlueCtx> ret_tmp_clone;
    OsInnerRetTmp<CGlueCtx> ret_tmp_osinner;
    MemoryViewRetTmp<CGlueCtx> ret_tmp_memoryview;
    OsKeyboardInnerRetTmp<CGlueCtx> ret_tmp_oskeyboardinner;
    PhysicalMemoryRetTmp<CGlueCtx> ret_tmp_physicalmemory;
};

using AddressCallback = OpaqueCallback<Address>;

/**
 * Type meant for process IDs
 *
 * If there is a case where Pid can be over 32-bit limit, or negative, please open an issue, we
 * would love to see that.
 */
using Pid = uint32_t;

/**
 * Exit code of a process
 */
using ExitCode = int32_t;

/**
 * The state of a process
 *
 * # Remarks
 *
 * In case the exit code isn't known ProcessState::Unknown is set.
 */
struct ProcessState {
    enum class Tag {
        ProcessState_Unknown,
        ProcessState_Alive,
        ProcessState_Dead,
    };

    struct ProcessState_Dead_Body {
        ExitCode _0;
    };

    Tag tag;
    union {
        ProcessState_Dead_Body dead;
    };
};

/**
 * Wrapper around null-terminated C-style strings.
 *
 * Analog to Rust's `String`, [`ReprCString`] owns the underlying data.
 */
using ReprCString = char*;

struct ArchitectureIdent {
    enum class Tag {
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
    };

    struct ArchitectureIdent_Unknown_Body {
        uintptr_t _0;
    };

    struct ArchitectureIdent_X86_Body {
        uint8_t _0;
        bool _1;
    };

    struct ArchitectureIdent_AArch64_Body {
        uintptr_t _0;
    };

    Tag tag;
    union {
        ArchitectureIdent_Unknown_Body unknown;
        ArchitectureIdent_X86_Body x86;
        ArchitectureIdent_AArch64_Body a_arch64;
    };
};

/**
 * Process information structure
 *
 * This structure implements basic process information. Architectures are provided both of the
 * system, and of the process.
 */
struct ProcessInfo {
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
    ProcessState state;
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
    ArchitectureIdent sys_arch;
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
    ArchitectureIdent proc_arch;
};

using ProcessInfoCallback = OpaqueCallback<ProcessInfo>;

template<typename CGlueInst, typename CGlueCtx>
struct ProcessInstanceContainer {
    CGlueInst instance;
    CGlueCtx context;
    MemoryViewRetTmp<CGlueCtx> ret_tmp_memoryview;
    ProcessRetTmp<CGlueCtx> ret_tmp_process;
    VirtualTranslateRetTmp<CGlueCtx> ret_tmp_virtualtranslate;
};

/**
 * Pair of address and architecture used for callbacks
 */
struct ModuleAddressInfo {
    Address address;
    ArchitectureIdent arch;
};

using ModuleAddressCallback = OpaqueCallback<ModuleAddressInfo>;

/**
 * Module information structure
 */
struct ModuleInfo {
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
    ArchitectureIdent arch;
};

using ModuleInfoCallback = OpaqueCallback<ModuleInfo>;

/**
 * Import information structure
 */
struct ImportInfo {
    /**
     * Name of the import
     */
    ReprCString name;
    /**
     * Offset of this import from the containing modules base address
     */
    umem offset;
};

using ImportCallback = OpaqueCallback<ImportInfo>;

/**
 * Export information structure
 */
struct ExportInfo {
    /**
     * Name of the export
     */
    ReprCString name;
    /**
     * Offset of this export from the containing modules base address
     */
    umem offset;
};

using ExportCallback = OpaqueCallback<ExportInfo>;

/**
 * Section information structure
 */
struct SectionInfo {
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
};

using SectionCallback = OpaqueCallback<SectionInfo>;

/**
 * CGlue vtable for trait Process.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct ProcessVtbl {
    ProcessState (*state)(CGlueC *cont);
    int32_t (*module_address_list_callback)(CGlueC *cont, const ArchitectureIdent *target_arch, ModuleAddressCallback callback);
    int32_t (*module_list_callback)(CGlueC *cont, const ArchitectureIdent *target_arch, ModuleInfoCallback callback);
    int32_t (*module_by_address)(CGlueC *cont, Address address, ArchitectureIdent architecture, MaybeUninit<ModuleInfo> *ok_out);
    int32_t (*module_by_name_arch)(CGlueC *cont, CSliceRef<uint8_t> name, const ArchitectureIdent *architecture, MaybeUninit<ModuleInfo> *ok_out);
    int32_t (*module_by_name)(CGlueC *cont, CSliceRef<uint8_t> name, MaybeUninit<ModuleInfo> *ok_out);
    int32_t (*primary_module_address)(CGlueC *cont, MaybeUninit<Address> *ok_out);
    int32_t (*primary_module)(CGlueC *cont, MaybeUninit<ModuleInfo> *ok_out);
    int32_t (*module_import_list_callback)(CGlueC *cont, const ModuleInfo *info, ImportCallback callback);
    int32_t (*module_export_list_callback)(CGlueC *cont, const ModuleInfo *info, ExportCallback callback);
    int32_t (*module_section_list_callback)(CGlueC *cont, const ModuleInfo *info, SectionCallback callback);
    int32_t (*module_import_by_name)(CGlueC *cont, const ModuleInfo *info, CSliceRef<uint8_t> name, MaybeUninit<ImportInfo> *ok_out);
    int32_t (*module_export_by_name)(CGlueC *cont, const ModuleInfo *info, CSliceRef<uint8_t> name, MaybeUninit<ExportInfo> *ok_out);
    int32_t (*module_section_by_name)(CGlueC *cont, const ModuleInfo *info, CSliceRef<uint8_t> name, MaybeUninit<SectionInfo> *ok_out);
    const ProcessInfo *(*info)(const CGlueC *cont);
};

/**
 * Virtual page range information used for callbacks
 */
struct MemoryRange {
    Address address;
    umem size;
};

/**
 * Virtual page range information with physical mappings used for callbacks
 */
struct VirtualTranslation {
    Address in_virtual;
    umem size;
    PhysicalAddress out_physical;
};

using VirtualTranslationCallback = OpaqueCallback<VirtualTranslation>;

struct VirtualTranslationFail {
    Address from;
    umem size;
};

using VirtualTranslationFailCallback = OpaqueCallback<VirtualTranslationFail>;

using MemoryRangeCallback = OpaqueCallback<MemoryRange>;

/**
 * A `Page` holds information about a memory page.
 *
 * More information about paging can be found [here](https://en.wikipedia.org/wiki/Paging).
 */
struct Page {
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
};

/**
 * FFI-safe Option.
 *
 * This type is not really meant for general use, but rather as a last-resort conversion for type
 * wrapping.
 *
 * Typical workflow would include temporarily converting into/from COption.
 */
template<typename T>
struct COption {
    enum class Tag {
        COption_None,
        COption_Some,
    };

    struct COption_Some_Body {
        T _0;
    };

    Tag tag;
    union {
        COption_Some_Body some;
    };
};

/**
 * CGlue vtable for trait VirtualTranslate.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct VirtualTranslateVtbl {
    void (*virt_to_phys_list)(CGlueC *cont, CSliceRef<MemoryRange> addrs, VirtualTranslationCallback out, VirtualTranslationFailCallback out_fail);
    void (*virt_to_phys_range)(CGlueC *cont, Address start, Address end, VirtualTranslationCallback out);
    void (*virt_translation_map_range)(CGlueC *cont, Address start, Address end, VirtualTranslationCallback out);
    void (*virt_page_map_range)(CGlueC *cont, umem gap_size, Address start, Address end, MemoryRangeCallback out);
    int32_t (*virt_to_phys)(CGlueC *cont, Address address, MaybeUninit<PhysicalAddress> *ok_out);
    int32_t (*virt_page_info)(CGlueC *cont, Address addr, MaybeUninit<Page> *ok_out);
    void (*virt_translation_map)(CGlueC *cont, VirtualTranslationCallback out);
    COption<Address> (*phys_to_virt)(CGlueC *cont, Address phys);
    void (*virt_page_map)(CGlueC *cont, umem gap_size, MemoryRangeCallback out);
};

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
template<typename CGlueInst, typename CGlueCtx>
struct ProcessInstance {
    const MemoryViewVtbl<ProcessInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_memoryview;
    const ProcessVtbl<ProcessInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_process;
    const VirtualTranslateVtbl<ProcessInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_virtualtranslate;
    ProcessInstanceContainer<CGlueInst, CGlueCtx> container;
};

template<typename CGlueInst, typename CGlueCtx>
struct IntoProcessInstanceContainer {
    CGlueInst instance;
    CGlueCtx context;
    CloneRetTmp<CGlueCtx> ret_tmp_clone;
    MemoryViewRetTmp<CGlueCtx> ret_tmp_memoryview;
    ProcessRetTmp<CGlueCtx> ret_tmp_process;
    VirtualTranslateRetTmp<CGlueCtx> ret_tmp_virtualtranslate;
};

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
template<typename CGlueInst, typename CGlueCtx>
struct IntoProcessInstance {
    const CloneVtbl<IntoProcessInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_clone;
    const MemoryViewVtbl<IntoProcessInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_memoryview;
    const ProcessVtbl<IntoProcessInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_process;
    const VirtualTranslateVtbl<IntoProcessInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_virtualtranslate;
    IntoProcessInstanceContainer<CGlueInst, CGlueCtx> container;
};

/**
 * Information block about OS
 *
 * This provides some basic information about the OS in question. `base`, and `size` may be
 * omitted in some circumstances (lack of kernel, or privileges). But architecture should always
 * be correct.
 */
struct OsInfo {
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
    ArchitectureIdent arch;
};

/**
 * CGlue vtable for trait OsInner.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct OsInnerVtbl {
    int32_t (*process_address_list_callback)(CGlueC *cont, AddressCallback callback);
    int32_t (*process_info_list_callback)(CGlueC *cont, ProcessInfoCallback callback);
    int32_t (*process_info_by_address)(CGlueC *cont, Address address, MaybeUninit<ProcessInfo> *ok_out);
    int32_t (*process_info_by_name)(CGlueC *cont, CSliceRef<uint8_t> name, MaybeUninit<ProcessInfo> *ok_out);
    int32_t (*process_info_by_pid)(CGlueC *cont, Pid pid, MaybeUninit<ProcessInfo> *ok_out);
    int32_t (*process_by_info)(CGlueC *cont, ProcessInfo info, MaybeUninit<ProcessInstance<CBox<void>, Context>> *ok_out);
    int32_t (*into_process_by_info)(CGlueC cont, ProcessInfo info, MaybeUninit<IntoProcessInstance<CBox<void>, Context>> *ok_out);
    int32_t (*process_by_address)(CGlueC *cont, Address addr, MaybeUninit<ProcessInstance<CBox<void>, Context>> *ok_out);
    int32_t (*process_by_name)(CGlueC *cont, CSliceRef<uint8_t> name, MaybeUninit<ProcessInstance<CBox<void>, Context>> *ok_out);
    int32_t (*process_by_pid)(CGlueC *cont, Pid pid, MaybeUninit<ProcessInstance<CBox<void>, Context>> *ok_out);
    int32_t (*into_process_by_address)(CGlueC cont, Address addr, MaybeUninit<IntoProcessInstance<CBox<void>, Context>> *ok_out);
    int32_t (*into_process_by_name)(CGlueC cont, CSliceRef<uint8_t> name, MaybeUninit<IntoProcessInstance<CBox<void>, Context>> *ok_out);
    int32_t (*into_process_by_pid)(CGlueC cont, Pid pid, MaybeUninit<IntoProcessInstance<CBox<void>, Context>> *ok_out);
    int32_t (*module_address_list_callback)(CGlueC *cont, AddressCallback callback);
    int32_t (*module_list_callback)(CGlueC *cont, ModuleInfoCallback callback);
    int32_t (*module_by_address)(CGlueC *cont, Address address, MaybeUninit<ModuleInfo> *ok_out);
    int32_t (*module_by_name)(CGlueC *cont, CSliceRef<uint8_t> name, MaybeUninit<ModuleInfo> *ok_out);
    const OsInfo *(*info)(const CGlueC *cont);
};

/**
 * CGlue vtable for trait KeyboardState.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct KeyboardStateVtbl {
    bool (*is_down)(const CGlueC *cont, int32_t vk);
};

/**
 * Base CGlue trait object for trait KeyboardState.
 */
template<typename CGlueInst, typename CGlueCtx>
using KeyboardStateBase = CGlueTraitObj<CGlueInst, KeyboardStateVtbl<CGlueObjContainer<CGlueInst, CGlueCtx, KeyboardStateRetTmp<CGlueCtx>>>, CGlueCtx, KeyboardStateRetTmp<CGlueCtx>>;

/**
 * CGlue vtable for trait Keyboard.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct KeyboardVtbl {
    bool (*is_down)(CGlueC *cont, int32_t vk);
    void (*set_down)(CGlueC *cont, int32_t vk, bool down);
    int32_t (*state)(CGlueC *cont, MaybeUninit<KeyboardStateBase<CBox<void>, Context>> *ok_out);
};

/**
 * Base CGlue trait object for trait Keyboard.
 */
template<typename CGlueInst, typename CGlueCtx>
using KeyboardBase = CGlueTraitObj<CGlueInst, KeyboardVtbl<CGlueObjContainer<CGlueInst, CGlueCtx, KeyboardRetTmp<CGlueCtx>>>, CGlueCtx, KeyboardRetTmp<CGlueCtx>>;

template<typename CGlueInst, typename CGlueCtx>
struct IntoKeyboardContainer {
    CGlueInst instance;
    CGlueCtx context;
    CloneRetTmp<CGlueCtx> ret_tmp_clone;
    KeyboardRetTmp<CGlueCtx> ret_tmp_keyboard;
};

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
template<typename CGlueInst, typename CGlueCtx>
struct IntoKeyboard {
    const CloneVtbl<IntoKeyboardContainer<CGlueInst, CGlueCtx>> *vtbl_clone;
    const KeyboardVtbl<IntoKeyboardContainer<CGlueInst, CGlueCtx>> *vtbl_keyboard;
    IntoKeyboardContainer<CGlueInst, CGlueCtx> container;
};

/**
 * CGlue vtable for trait OsKeyboardInner.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct OsKeyboardInnerVtbl {
    int32_t (*keyboard)(CGlueC *cont, MaybeUninit<KeyboardBase<CBox<void>, Context>> *ok_out);
    int32_t (*into_keyboard)(CGlueC cont, MaybeUninit<IntoKeyboard<CBox<void>, Context>> *ok_out);
};

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
template<typename CGlueInst, typename CGlueCtx>
struct OsInstance {
    const CloneVtbl<OsInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_clone;
    const OsInnerVtbl<OsInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_osinner;
    const MemoryViewVtbl<OsInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_memoryview;
    const OsKeyboardInnerVtbl<OsInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_oskeyboardinner;
    const PhysicalMemoryVtbl<OsInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_physicalmemory;
    OsInstanceContainer<CGlueInst, CGlueCtx> container;
};

template<typename CGlueT, typename CGlueCtx>
using OsInstanceBaseCtxBox = OsInstance<CBox<CGlueT>, CGlueCtx>;

template<typename CGlueT, typename CGlueArcTy>
using OsInstanceBaseArcBox = OsInstanceBaseCtxBox<CGlueT, CArc<CGlueArcTy>>;

using OsInstanceArcBox = OsInstanceBaseArcBox<void, void>;

using MuOsInstanceArcBox = MaybeUninit<OsInstanceArcBox>;

template<typename CGlueT, typename CGlueCtx>
using ProcessInstanceBaseCtxBox = ProcessInstance<CBox<CGlueT>, CGlueCtx>;

template<typename CGlueT, typename CGlueArcTy>
using ProcessInstanceBaseArcBox = ProcessInstanceBaseCtxBox<CGlueT, CArc<CGlueArcTy>>;

using ProcessInstanceArcBox = ProcessInstanceBaseArcBox<void, void>;

template<typename CGlueT, typename CGlueCtx>
using IntoProcessInstanceBaseCtxBox = IntoProcessInstance<CBox<CGlueT>, CGlueCtx>;

template<typename CGlueT, typename CGlueArcTy>
using IntoProcessInstanceBaseArcBox = IntoProcessInstanceBaseCtxBox<CGlueT, CArc<CGlueArcTy>>;

using IntoProcessInstanceArcBox = IntoProcessInstanceBaseArcBox<void, void>;

/**
 * CtxBoxed CGlue trait object for trait MemoryView with context.
 */
template<typename CGlueT, typename CGlueCtx>
using MemoryViewBaseCtxBox = MemoryViewBase<CBox<CGlueT>, CGlueCtx>;

/**
 * Boxed CGlue trait object for trait MemoryView with a [`CArc`](cglue::arc::CArc) reference counted context.
 */
template<typename CGlueT, typename CGlueC>
using MemoryViewBaseArcBox = MemoryViewBaseCtxBox<CGlueT, CArc<CGlueC>>;

/**
 * Opaque Boxed CGlue trait object for trait MemoryView with a [`CArc`](cglue::arc::CArc) reference counted context.
 */
using MemoryViewArcBox = MemoryViewBaseArcBox<void, void>;

extern "C" {

extern const ArchitectureObj *X86_32;

extern const ArchitectureObj *X86_32_PAE;

extern const ArchitectureObj *X86_64;

void log_init(int32_t level_num);

void debug_error(int32_t error);

/**
 * Helper to convert `Address` to a `PhysicalAddress`
 *
 * This will create a `PhysicalAddress` with `UNKNOWN` PageType.
 */
PhysicalAddress addr_to_paddr(Address address);

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
Inventory *inventory_scan();

/**
 * Create a new inventory with custom path string
 *
 * # Safety
 *
 * `path` must be a valid null terminated string
 */
Inventory *inventory_scan_path(const char *path);

/**
 * Add a directory to an existing inventory
 *
 * # Safety
 *
 * `dir` must be a valid null terminated string
 */
int32_t inventory_add_dir(Inventory *inv, const char *dir);

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
int32_t inventory_create_connector(Inventory *inv,
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
int32_t inventory_create_os(Inventory *inv,
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
void inventory_free(Inventory *inv);

uint8_t arch_bits(const ArchitectureObj *arch);

Endianess arch_endianess(const ArchitectureObj *arch);

uintptr_t arch_page_size(const ArchitectureObj *arch);

uintptr_t arch_size_addr(const ArchitectureObj *arch);

uint8_t arch_address_space_bits(const ArchitectureObj *arch);

/**
 * Free an architecture reference
 *
 * # Safety
 *
 * `arch` must be a valid heap allocated reference created by one of the API's functions.
 */
void arch_free(ArchitectureObj *arch);

bool is_x86_arch(const ArchitectureObj *arch);

} // extern "C"

#endif // MEMFLOW_H