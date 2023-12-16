#ifndef MEMFLOW_H
#define MEMFLOW_H

#include <cstdarg>
#include <cstring>
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

/**
 * An enum representing the available verbosity levels of the logger.
 *
 * Typical usage includes: checking if a certain `Level` is enabled with
 * [`log_enabled!`](macro.log_enabled.html), specifying the `Level` of
 * [`log!`](macro.log.html), and comparing a `Level` directly to a
 * [`LevelFilter`](enum.LevelFilter.html).
 */
enum class Level : uintptr_t {
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
enum class LevelFilter : uintptr_t {
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

struct ArchitectureObj;


/** Destruct the object. */
template<typename T>
inline typename std::enable_if<!std::is_pointer<T>::value>::type mem_drop(T &&self) noexcept {
    std::move(self).drop();
}

template<typename T>
inline typename std::enable_if<std::is_pointer<T>::value>::type mem_drop(T &&self) noexcept {}

/** Forget the object's resources (null them out). */
template<typename T>
inline typename std::enable_if<!std::is_pointer<T>::value>::type mem_forget(T &self) noexcept {
    self.forget();
}

template<typename T>
inline typename std::enable_if<std::is_pointer<T>::value>::type mem_forget(T &self) noexcept {}

/** Defer mem_forget call when object goes out of scope. */
template<typename T>
struct DeferedForget {
    T &val;

    DeferedForget(T &val) : val(val) {}

    ~DeferedForget() {
        mem_forget(val);
    }
};

/** Workaround for void types in generic functions. */
struct StoreAll {
    constexpr bool operator[](StoreAll) const {
        return false;
    }

    template <class T>
    constexpr T && operator[](T &&t) const {
        return std::forward<T>(t);
    }

    template <class T>
    friend T && operator,(T &&t, StoreAll) {
        return std::forward<T>(t);
    }
};

template<typename CGlueCtx = void>
using CloneRetTmp = void;

template<typename CGlueCtx = void>
using ConnectorCpuStateRetTmp = void;

template<typename CGlueCtx = void>
using CpuStateRetTmp = void;

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
struct Inventory;

template<typename CGlueCtx = void>
using KeyboardRetTmp = void;

template<typename CGlueCtx = void>
using KeyboardStateRetTmp = void;

template<typename T = void>
struct alignas(alignof(T)) RustMaybeUninit {
    char pad[sizeof(T)];
    inline T &assume_init() {
        return *(T *)this;
    }
    constexpr const T &assume_init() const {
        return *(const T *)this;
    }
};

template<typename CGlueCtx = void>
using MemoryViewRetTmp = void;

template<typename CGlueCtx = void>
using OsKeyboardRetTmp = void;

template<typename CGlueCtx = void>
using OsRetTmp = void;

template<typename CGlueCtx = void>
using PhysicalMemoryRetTmp = void;

template<typename CGlueCtx = void>
using ProcessRetTmp = void;

template<typename CGlueCtx = void>
using VirtualTranslateRetTmp = void;

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
constexpr static const Address Address_NULL = 0;
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
constexpr static const Address Address_INVALID = ~0;

/**
 * Describes the type of a page using a bitflag.
 */
using PageType = uint8_t;
/**
 * The page explicitly has no flags.
 */
constexpr static const PageType PageType_NONE = 0;
/**
 * The page type is not known.
 */
constexpr static const PageType PageType_UNKNOWN = 1;
/**
 * The page contains page table entries.
 */
constexpr static const PageType PageType_PAGE_TABLE = 2;
/**
 * The page is a writeable page.
 */
constexpr static const PageType PageType_WRITEABLE = 4;
/**
 * The page is read only.
 */
constexpr static const PageType PageType_READ_ONLY = 8;
/**
 * The page is not executable.
 */
constexpr static const PageType PageType_NOEXEC = 16;

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
 * A physical address with an invalid value.
 */
constexpr static const PhysicalAddress PhysicalAddress_INVALID = PhysicalAddress{ /* .address = */ Address_INVALID, /* .page_type = */ PageType_UNKNOWN, /* .page_size_log2 = */ 0 };

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

    inline CArc clone() const noexcept {
        CArc ret;
        ret.instance = clone_fn(instance);
        ret.clone_fn = clone_fn;
        ret.drop_fn = drop_fn;
        return ret;
    }

    inline void drop() && noexcept {
        if (drop_fn)
            drop_fn(instance);
        forget();
    }

    inline void forget() noexcept {
        instance = nullptr;
        clone_fn = nullptr;
        drop_fn = nullptr;
    }
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

    CBox() = default;
    CBox(T *instance) : instance(instance), drop_fn(nullptr) {}
    CBox(T *instance, void (*drop_fn)(T *)) : instance(instance), drop_fn(drop_fn) {}
    template<typename U = T, class = typename std::enable_if<std::is_same<U, T>::value>::type, class = typename std::enable_if<!std::is_same<U, void>::value>::type>
    CBox(U &&instance) : instance(new U(instance)), drop_fn(&CBox::delete_fn) {}

    static void delete_fn(T *v) {
        delete v;
    }

    inline operator CBox<void> () const {
        CBox<void> ret;
        ret.instance = (void*)instance;
        ret.drop_fn = (void(*)(void *))drop_fn;
        return ret;
    }

    static inline CBox new_box() {
        CBox ret;
        ret.instance = new T;
        ret.drop_fn = &CBox::delete_fn;
        return ret;
    }

    inline void drop() && noexcept {
        if (drop_fn && instance)
            drop_fn(instance);
        forget();
    }

    inline void forget() noexcept {
        instance = nullptr;
        drop_fn = nullptr;
    }

    inline T *operator->() {
        return instance;
    }

    inline const T *operator->() const {
        return instance;
    }
};

template<typename CGlueInst = CBox<void>, typename CGlueCtx = CArc<void>>
struct ConnectorInstanceContainer {
    typedef CGlueCtx Context;
    CGlueInst instance;
    CGlueCtx context;

    inline Context clone_context() noexcept {
        return context.clone();
    }

    inline void drop() && noexcept {
        mem_drop(std::move(instance));
        mem_drop(std::move(context));
    }

    inline void forget() noexcept {
        mem_forget(instance);
        mem_forget(context);
    }
};

template<typename CGlueInst>
struct ConnectorInstanceContainer<CGlueInst, void> {
    typedef void Context;
    CGlueInst instance;

    inline Context clone_context() noexcept {}

    inline void drop() && noexcept {
        mem_drop(std::move(instance));
    }

    inline void forget() noexcept {
        mem_forget(instance);
    }
};

/**
 * CGlue vtable for trait Clone.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct CloneVtbl {
    typedef typename CGlueC::Context Context;
    CGlueC (*clone)(const CGlueC *cont);
};

template<typename Impl>
struct CloneVtblImpl : CloneVtbl<typename Impl::Parent> {
constexpr CloneVtblImpl() :
    CloneVtbl<typename Impl::Parent> {
        &Impl::clone
    } {}
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

    CSliceMut () = default;

    template<typename Cont, class = typename std::enable_if<
        std::is_same<decltype((*(Cont *)nullptr).data()), T *>::value
        && std::is_same<decltype((*(Cont *)nullptr).size()), size_t>::value
    >::type>
    CSliceMut (Cont &data) : data(data.data()), len(data.size()) {}

    template<typename U = T, class = typename std::enable_if<
        (std::is_same<T, char>::value || std::is_same<T, unsigned char>::value)
        && std::is_same<T, U>::value
    >::type>
    CSliceMut (char *value) : data((T *)value), len(strlen(value)) {}

    template<typename U = T, class = typename std::enable_if<
        (std::is_same<T, char>::value || std::is_same<T, unsigned char>::value)
        && std::is_same<T, U>::value
    >::type>
    CSliceMut (char *value, uintptr_t len) : data((T *)value), len(len) {}

    template<typename U = T, class = typename std::enable_if<
        (std::is_same<T, char>::value || std::is_same<T, unsigned char>::value)
        && std::is_same<T, U>::value
    >::type>
    CSliceMut (std::string &value) : data((T *)value.data()), len(value.length()) {}

    template<typename U = T, class = typename std::enable_if<
        (std::is_same<T, char>::value || std::is_same<T, unsigned char>::value)
        && std::is_same<T, U>::value
    >::type>
    inline operator std::string() const {
        return std::string((char *)data, len);
    }
};

/**
 * FFI-safe 3 element tuple.
 */
template<typename A, typename B, typename C>
struct CTup3 {
    A _0;
    B _1;
    C _2;
};

/**
 * MemData type for physical memory reads.
 */
using PhysicalReadData = CTup3<PhysicalAddress, Address, CSliceMut<uint8_t>>;

/**
 * FFI-safe 2 element tuple.
 */
template<typename A, typename B>
struct CTup2 {
    A _0;
    B _1;
};

using ReadData = CTup2<Address, CSliceMut<uint8_t>>;

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
    int32_t (*func)(void*, T *out);

    class iterator : std::iterator<std::input_iterator_tag, T> {
        CIterator<T> *iter;
        RustMaybeUninit<T> data;
        bool initialized = false;
        bool end = false;

      public:
        explicit iterator() : end(true) {}

        explicit iterator(CIterator<T> *iter) : iter(iter) {
            end = iter->func(iter->iter, &data.assume_init());
        }

        iterator &operator++() {
            if (!iter || end) {
                return *this;
            }

            end = iter->func(iter->iter, &data.assume_init());

            return *this;
        }

        constexpr bool operator==(const iterator &other) const {
            return (end && other.end)
                || (!end && !other.end && data.assume_init() == other.data.assume_init());
        }

        constexpr bool operator!=(const iterator &other) const {
            return !(*this == other);
        }

        inline T &operator*() {
            return data.assume_init();
        }

        constexpr const T &operator*() const {
            return data.assume_init();
        }
    };

    constexpr iterator begin() {
        return iterator(this);
    }

    constexpr iterator end() {
        return iterator();
    }
};

template<typename Container>
struct CPPIterator {

    typedef typename Container::iterator::value_type T;

    CIterator<T> iter;
    typename Container::iterator cur, end;

    static int32_t next(void *data, T *out) {
        CPPIterator *i = (CPPIterator *)data;

        if (i->cur == i->end) {
            return 1;
        } else {
            *out = *i->cur;
            i->cur++;
            return 0;
        }
    }

    CPPIterator(Container &cont)
        : cur(cont.begin()), end(cont.end())
    {
        iter.iter = &iter - offsetof(CPPIterator<Container>, iter);
        iter.func = &CPPIterator::next;
    }

    CPPIterator(CPPIterator &&o) {
        iter = o.iter;
        iter.iter = &this;
        cur = o.cur;
        end = o.end;
    }

    CPPIterator(CPPIterator &o) {
        iter = o.iter;
        iter.iter = &this;
        cur = o.cur;
        end = o.end;
    }

    inline operator CIterator<T> &() {
        return iter;
    }
};

template<typename T, typename F>
struct Callback {
    T *context;
    bool (*func)(T*, F);

    template<typename Container>
    static bool push_back(Container *context, F data) {
        context->push_back(data);
        return true;
    }

    template<typename Function>
    static bool functional(Function *function, F data) {
        return (*function)(data);
    }

    Callback() = default;

    template<typename OT, typename = decltype(std::declval<OT>().push_back(std::declval<F>()))>
    Callback(OT *cont) :
        context((T *)cont),
        func((decltype(func))(&Callback::push_back<OT>)) {}

    template<typename Function, typename = decltype(std::declval<Function>()(std::declval<F>()))>
    Callback(const Function &function) :
        context((T *)&function),
        func((decltype(func))(&Callback::functional<Function>)) {}

    constexpr operator Callback<void, F> &() {
        return *((Callback<void, F> *)this);
    }
};

template<typename T>
using OpaqueCallback = Callback<void, T>;

/**
 * Data needed to perform memory operations.
 *
 * `inp` is an iterator containing
 */
template<typename T, typename P>
struct MemOps {
    CIterator<T> inp;
    OpaqueCallback<P> *out;
    OpaqueCallback<P> *out_fail;
};

using PhysicalReadMemOps = MemOps<PhysicalReadData, ReadData>;

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

    CSliceRef () = default;

    template<typename Cont, class = typename std::enable_if<
        std::is_same<decltype((*(const Cont *)nullptr).data()), const T *>::value
        && std::is_same<decltype((*(const Cont *)nullptr).size()), size_t>::value
    >::type>
    CSliceRef (const Cont &data) : data(data.data()), len(data.size()) {}

    template<typename U = T, class = typename std::enable_if<
        (std::is_same<T, char>::value || std::is_same<T, unsigned char>::value)
        && std::is_same<T, U>::value
    >::type>
    CSliceRef (const char *value) : data((const T *)value), len(strlen(value)) {}

    template<typename U = T, class = typename std::enable_if<
        (std::is_same<T, char>::value || std::is_same<T, unsigned char>::value)
        && std::is_same<T, U>::value
    >::type>
    CSliceRef (const char *value, uintptr_t len) : data((const T *)value), len(len) {}

    template<typename U = T, class = typename std::enable_if<
        (std::is_same<T, char>::value || std::is_same<T, unsigned char>::value)
        && std::is_same<T, U>::value
    >::type>
    CSliceRef (const std::string &value) : data((const T *)value.data()), len(value.length()) {}

    template<typename U = T, class = typename std::enable_if<
        (std::is_same<T, char>::value || std::is_same<T, unsigned char>::value)
        && std::is_same<T, U>::value
    >::type>
    inline operator std::string() const {
        return std::string((char *)data, len);
    }
};

/**
 * MemData type for physical memory writes.
 */
using PhysicalWriteData = CTup3<PhysicalAddress, Address, CSliceRef<uint8_t>>;

using WriteData = CTup2<Address, CSliceRef<uint8_t>>;

using PhysicalWriteMemOps = MemOps<PhysicalWriteData, WriteData>;

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
    typedef C Context;
    T instance;
    C context;
    RustMaybeUninit<R> ret_tmp;

    inline Context clone_context() noexcept {
        return context.clone();
    }

    inline void drop() && noexcept {
        mem_drop(std::move(instance));
        mem_drop(std::move(context));
    }

    inline void forget() noexcept {
        mem_forget(instance);
        mem_forget(context);
    }
};

template<typename T, typename R>
struct CGlueObjContainer<T, void, R> {
    typedef void Context;
    T instance;
    RustMaybeUninit<R> ret_tmp;

    inline Context clone_context() noexcept {}

    inline void drop() && noexcept {
        mem_drop(std::move(instance));
    }

    inline void forget() noexcept {
        mem_forget(instance);
    }
};

template<typename T, typename C>
struct CGlueObjContainer<T, C, void> {
    typedef C Context;
    T instance;
    C context;

    inline Context clone_context() noexcept {
        return context.clone();
    }

    void drop() && noexcept {
        mem_drop(std::move(instance));
        mem_drop(std::move(context));
    }

    void forget() noexcept {
        mem_forget(instance);
        mem_forget(context);
    }
};

template<typename T>
struct CGlueObjContainer<T, void, void> {
    typedef void Context;
    T instance;

    inline Context clone_context() noexcept {}

    inline void drop() && noexcept {
        mem_drop(std::move(instance));
    }

    inline void forget() noexcept {
        mem_forget(instance);
    }
};

/**
 * MemData type for regular memory reads.
 */
using ReadDataRaw = CTup3<Address, Address, CSliceMut<uint8_t>>;

using ReadRawMemOps = MemOps<ReadDataRaw, ReadData>;

/**
 * MemData type for regular memory writes.
 */
using WriteDataRaw = CTup3<Address, Address, CSliceRef<uint8_t>>;

using WriteRawMemOps = MemOps<WriteDataRaw, WriteData>;

struct MemoryViewMetadata {
    Address max_address;
    umem real_size;
    bool readonly;
    bool little_endian;
    uint8_t arch_bits;
};

using ReadCallback = OpaqueCallback<ReadData>;

using WriteCallback = OpaqueCallback<WriteData>;

/**
 * CGlue vtable for trait MemoryView.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct MemoryViewVtbl {
    typedef typename CGlueC::Context Context;
    int32_t (*read_raw_iter)(CGlueC *cont, ReadRawMemOps data);
    int32_t (*write_raw_iter)(CGlueC *cont, WriteRawMemOps data);
    MemoryViewMetadata (*metadata)(const CGlueC *cont);
    int32_t (*read_iter)(CGlueC *cont,
                         CIterator<ReadData> inp,
                         ReadCallback *out,
                         ReadCallback *out_fail);
    int32_t (*read_raw_list)(CGlueC *cont, CSliceMut<ReadData> data);
    int32_t (*read_raw_into)(CGlueC *cont, Address addr, CSliceMut<uint8_t> out);
    int32_t (*write_iter)(CGlueC *cont,
                          CIterator<WriteData> inp,
                          WriteCallback *out,
                          WriteCallback *out_fail);
    int32_t (*write_raw_list)(CGlueC *cont, CSliceRef<WriteData> data);
    int32_t (*write_raw)(CGlueC *cont, Address addr, CSliceRef<uint8_t> data);
};

template<typename Impl>
struct MemoryViewVtblImpl : MemoryViewVtbl<typename Impl::Parent> {
constexpr MemoryViewVtblImpl() :
    MemoryViewVtbl<typename Impl::Parent> {
        &Impl::read_raw_iter,
        &Impl::write_raw_iter,
        &Impl::metadata,
        &Impl::read_iter,
        &Impl::read_raw_list,
        &Impl::read_raw_into,
        &Impl::write_iter,
        &Impl::write_raw_list,
        &Impl::write_raw
    } {}
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
template<typename CGlueInst = CBox<void>, typename CGlueCtx = CArc<void>>
using MemoryViewBase = CGlueTraitObj<CGlueInst, MemoryViewVtbl<CGlueObjContainer<CGlueInst, CGlueCtx, MemoryViewRetTmp<CGlueCtx>>>, CGlueCtx, MemoryViewRetTmp<CGlueCtx>>;

/**
 * CGlue vtable for trait PhysicalMemory.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct PhysicalMemoryVtbl {
    typedef typename CGlueC::Context Context;
    int32_t (*phys_read_raw_iter)(CGlueC *cont, PhysicalReadMemOps data);
    int32_t (*phys_write_raw_iter)(CGlueC *cont, PhysicalWriteMemOps data);
    PhysicalMemoryMetadata (*metadata)(const CGlueC *cont);
    void (*set_mem_map)(CGlueC *cont, CSliceRef<PhysicalMemoryMapping> _mem_map);
    MemoryViewBase<CBox<void>, Context> (*into_phys_view)(CGlueC cont);
    MemoryViewBase<CBox<void>, Context> (*phys_view)(CGlueC *cont);
};

template<typename Impl>
struct PhysicalMemoryVtblImpl : PhysicalMemoryVtbl<typename Impl::Parent> {
constexpr PhysicalMemoryVtblImpl() :
    PhysicalMemoryVtbl<typename Impl::Parent> {
        &Impl::phys_read_raw_iter,
        &Impl::phys_write_raw_iter,
        &Impl::metadata,
        &Impl::set_mem_map,
        &Impl::into_phys_view,
        &Impl::phys_view
    } {}
};

/**
 * CGlue vtable for trait CpuState.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct CpuStateVtbl {
    typedef typename CGlueC::Context Context;
    void (*pause)(CGlueC *cont);
    void (*resume)(CGlueC *cont);
};

template<typename Impl>
struct CpuStateVtblImpl : CpuStateVtbl<typename Impl::Parent> {
constexpr CpuStateVtblImpl() :
    CpuStateVtbl<typename Impl::Parent> {
        &Impl::pause,
        &Impl::resume
    } {}
};

/**
 * Base CGlue trait object for trait CpuState.
 */
template<typename CGlueInst = CBox<void>, typename CGlueCtx = CArc<void>>
using CpuStateBase = CGlueTraitObj<CGlueInst, CpuStateVtbl<CGlueObjContainer<CGlueInst, CGlueCtx, CpuStateRetTmp<CGlueCtx>>>, CGlueCtx, CpuStateRetTmp<CGlueCtx>>;

template<typename CGlueInst = CBox<void>, typename CGlueCtx = CArc<void>>
struct IntoCpuStateContainer {
    typedef CGlueCtx Context;
    CGlueInst instance;
    CGlueCtx context;

    inline Context clone_context() noexcept {
        return context.clone();
    }

    inline void drop() && noexcept {
        mem_drop(std::move(instance));
        mem_drop(std::move(context));
    }

    inline void forget() noexcept {
        mem_forget(instance);
        mem_forget(context);
    }
};

template<typename CGlueInst>
struct IntoCpuStateContainer<CGlueInst, void> {
    typedef void Context;
    CGlueInst instance;

    inline Context clone_context() noexcept {}

    inline void drop() && noexcept {
        mem_drop(std::move(instance));
    }

    inline void forget() noexcept {
        mem_forget(instance);
    }
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
template<typename CGlueInst = CBox<void>, typename CGlueCtx = CArc<void>>
struct IntoCpuState {
    const CloneVtbl<IntoCpuStateContainer<CGlueInst, CGlueCtx>> *vtbl_clone;
    const CpuStateVtbl<IntoCpuStateContainer<CGlueInst, CGlueCtx>> *vtbl_cpustate;
    IntoCpuStateContainer<CGlueInst, CGlueCtx> container;

    IntoCpuState() : container{} , vtbl_clone{}, vtbl_cpustate{} {}

    ~IntoCpuState() noexcept {
        mem_drop(std::move(container));
    }

    typedef CGlueCtx Context;

    inline IntoCpuState clone() const noexcept {
        IntoCpuState __ret;
            __ret.vtbl_clone = this->vtbl_clone;
            __ret.vtbl_cpustate = this->vtbl_cpustate;
        __ret.container = (this->vtbl_clone)->clone(&this->container);
        return __ret;
    }

    inline void pause() noexcept {
    (this->vtbl_cpustate)->pause(&this->container);

    }

    inline void resume() noexcept {
    (this->vtbl_cpustate)->resume(&this->container);

    }

};

/**
 * CGlue vtable for trait ConnectorCpuState.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct ConnectorCpuStateVtbl {
    typedef typename CGlueC::Context Context;
    int32_t (*cpu_state)(CGlueC *cont, CpuStateBase<CBox<void>, Context> *ok_out);
    int32_t (*into_cpu_state)(CGlueC cont, IntoCpuState<CBox<void>, Context> *ok_out);
};

template<typename Impl>
struct ConnectorCpuStateVtblImpl : ConnectorCpuStateVtbl<typename Impl::Parent> {
constexpr ConnectorCpuStateVtblImpl() :
    ConnectorCpuStateVtbl<typename Impl::Parent> {
        &Impl::cpu_state,
        &Impl::into_cpu_state
    } {}
};

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
template<typename CGlueInst = CBox<void>, typename CGlueCtx = CArc<void>>
struct ConnectorInstance {
    const CloneVtbl<ConnectorInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_clone;
    const PhysicalMemoryVtbl<ConnectorInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_physicalmemory;
    const ConnectorCpuStateVtbl<ConnectorInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_connectorcpustate;
    ConnectorInstanceContainer<CGlueInst, CGlueCtx> container;

    ConnectorInstance() : container{} , vtbl_clone{}, vtbl_physicalmemory{}, vtbl_connectorcpustate{} {}

    ~ConnectorInstance() noexcept {
        mem_drop(std::move(container));
    }

    typedef CGlueCtx Context;

    inline ConnectorInstance clone() const noexcept {
        ConnectorInstance __ret;
            __ret.vtbl_clone = this->vtbl_clone;
            __ret.vtbl_physicalmemory = this->vtbl_physicalmemory;
            __ret.vtbl_connectorcpustate = this->vtbl_connectorcpustate;
        __ret.container = (this->vtbl_clone)->clone(&this->container);
        return __ret;
    }

    inline int32_t phys_read_raw_iter(PhysicalReadMemOps data) noexcept {
        int32_t __ret = (this->vtbl_physicalmemory)->phys_read_raw_iter(&this->container, data);
        return __ret;
    }

    inline int32_t phys_write_raw_iter(PhysicalWriteMemOps data) noexcept {
        int32_t __ret = (this->vtbl_physicalmemory)->phys_write_raw_iter(&this->container, data);
        return __ret;
    }

    inline PhysicalMemoryMetadata metadata() const noexcept {
        PhysicalMemoryMetadata __ret = (this->vtbl_physicalmemory)->metadata(&this->container);
        return __ret;
    }

    inline void set_mem_map(CSliceRef<PhysicalMemoryMapping> _mem_map) noexcept {
    (this->vtbl_physicalmemory)->set_mem_map(&this->container, _mem_map);

    }

    inline MemoryViewBase<CBox<void>, Context> into_phys_view() && noexcept {
        auto ___ctx = StoreAll()[this->container.clone_context(), StoreAll()];
        MemoryViewBase<CBox<void>, Context> __ret = (this->vtbl_physicalmemory)->into_phys_view(this->container);
        mem_forget(this->container);
        return __ret;
    }

    inline MemoryViewBase<CBox<void>, Context> phys_view() noexcept {
        MemoryViewBase<CBox<void>, Context> __ret = (this->vtbl_physicalmemory)->phys_view(&this->container);
        return __ret;
    }

    inline int32_t cpu_state(CpuStateBase<CBox<void>, Context> * ok_out) noexcept {
        int32_t __ret = (this->vtbl_connectorcpustate)->cpu_state(&this->container, ok_out);
        return __ret;
    }

    inline int32_t into_cpu_state(IntoCpuState<CBox<void>, Context> * ok_out) && noexcept {
        auto ___ctx = StoreAll()[this->container.clone_context(), StoreAll()];
        int32_t __ret = (this->vtbl_connectorcpustate)->into_cpu_state(this->container, ok_out);
        mem_forget(this->container);
        return __ret;
    }

};

template<typename CGlueT, typename CGlueCtx = CArc<void>>
using ConnectorInstanceBaseCtxBox = ConnectorInstance<CBox<CGlueT>, CGlueCtx>;

template<typename CGlueT, typename CGlueArcTy>
using ConnectorInstanceBaseArcBox = ConnectorInstanceBaseCtxBox<CGlueT, CArc<CGlueArcTy>>;
// Typedef for default contaienr and context type
template<typename CGlueT, typename CGlueArcTy>
using ConnectorInstanceBase = ConnectorInstanceBaseArcBox<CGlueT,CGlueArcTy>;

using ConnectorInstanceArcBox = ConnectorInstanceBaseArcBox<void, void>;

using MuConnectorInstanceArcBox = ConnectorInstanceArcBox;
// Typedef for default contaienr and context type
using MuConnectorInstance = MuConnectorInstanceArcBox;

template<typename CGlueInst = CBox<void>, typename CGlueCtx = CArc<void>>
struct OsInstanceContainer {
    typedef CGlueCtx Context;
    CGlueInst instance;
    CGlueCtx context;

    inline Context clone_context() noexcept {
        return context.clone();
    }

    inline void drop() && noexcept {
        mem_drop(std::move(instance));
        mem_drop(std::move(context));
    }

    inline void forget() noexcept {
        mem_forget(instance);
        mem_forget(context);
    }
};

template<typename CGlueInst>
struct OsInstanceContainer<CGlueInst, void> {
    typedef void Context;
    CGlueInst instance;

    inline Context clone_context() noexcept {}

    inline void drop() && noexcept {
        mem_drop(std::move(instance));
    }

    inline void forget() noexcept {
        mem_forget(instance);
    }
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
};

using ProcessInfoCallback = OpaqueCallback<ProcessInfo>;

template<typename CGlueInst = CBox<void>, typename CGlueCtx = CArc<void>>
struct ProcessInstanceContainer {
    typedef CGlueCtx Context;
    CGlueInst instance;
    CGlueCtx context;

    inline Context clone_context() noexcept {
        return context.clone();
    }

    inline void drop() && noexcept {
        mem_drop(std::move(instance));
        mem_drop(std::move(context));
    }

    inline void forget() noexcept {
        mem_forget(instance);
        mem_forget(context);
    }
};

template<typename CGlueInst>
struct ProcessInstanceContainer<CGlueInst, void> {
    typedef void Context;
    CGlueInst instance;

    inline Context clone_context() noexcept {}

    inline void drop() && noexcept {
        mem_drop(std::move(instance));
    }

    inline void forget() noexcept {
        mem_forget(instance);
    }
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

using imem = int64_t;

using MemoryRange = CTup3<Address, umem, PageType>;

using MemoryRangeCallback = OpaqueCallback<MemoryRange>;

/**
 * CGlue vtable for trait Process.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct ProcessVtbl {
    typedef typename CGlueC::Context Context;
    ProcessState (*state)(CGlueC *cont);
    int32_t (*set_dtb)(CGlueC *cont, Address dtb1, Address dtb2);
    int32_t (*module_address_list_callback)(CGlueC *cont,
                                            const ArchitectureIdent *target_arch,
                                            ModuleAddressCallback callback);
    int32_t (*module_list_callback)(CGlueC *cont,
                                    const ArchitectureIdent *target_arch,
                                    ModuleInfoCallback callback);
    int32_t (*module_by_address)(CGlueC *cont,
                                 Address address,
                                 ArchitectureIdent architecture,
                                 ModuleInfo *ok_out);
    int32_t (*module_by_name_arch)(CGlueC *cont,
                                   CSliceRef<uint8_t> name,
                                   const ArchitectureIdent *architecture,
                                   ModuleInfo *ok_out);
    int32_t (*module_by_name)(CGlueC *cont, CSliceRef<uint8_t> name, ModuleInfo *ok_out);
    int32_t (*primary_module_address)(CGlueC *cont, Address *ok_out);
    int32_t (*primary_module)(CGlueC *cont, ModuleInfo *ok_out);
    int32_t (*module_import_list_callback)(CGlueC *cont,
                                           const ModuleInfo *info,
                                           ImportCallback callback);
    int32_t (*module_export_list_callback)(CGlueC *cont,
                                           const ModuleInfo *info,
                                           ExportCallback callback);
    int32_t (*module_section_list_callback)(CGlueC *cont,
                                            const ModuleInfo *info,
                                            SectionCallback callback);
    int32_t (*module_import_by_name)(CGlueC *cont,
                                     const ModuleInfo *info,
                                     CSliceRef<uint8_t> name,
                                     ImportInfo *ok_out);
    int32_t (*module_export_by_name)(CGlueC *cont,
                                     const ModuleInfo *info,
                                     CSliceRef<uint8_t> name,
                                     ExportInfo *ok_out);
    int32_t (*module_section_by_name)(CGlueC *cont,
                                      const ModuleInfo *info,
                                      CSliceRef<uint8_t> name,
                                      SectionInfo *ok_out);
    const ProcessInfo *(*info)(const CGlueC *cont);
    void (*mapped_mem_range)(CGlueC *cont,
                             imem gap_size,
                             Address start,
                             Address end,
                             MemoryRangeCallback out);
    void (*mapped_mem)(CGlueC *cont, imem gap_size, MemoryRangeCallback out);
};

template<typename Impl>
struct ProcessVtblImpl : ProcessVtbl<typename Impl::Parent> {
constexpr ProcessVtblImpl() :
    ProcessVtbl<typename Impl::Parent> {
        &Impl::state,
        &Impl::set_dtb,
        &Impl::module_address_list_callback,
        &Impl::module_list_callback,
        &Impl::module_by_address,
        &Impl::module_by_name_arch,
        &Impl::module_by_name,
        &Impl::primary_module_address,
        &Impl::primary_module,
        &Impl::module_import_list_callback,
        &Impl::module_export_list_callback,
        &Impl::module_section_list_callback,
        &Impl::module_import_by_name,
        &Impl::module_export_by_name,
        &Impl::module_section_by_name,
        &Impl::info,
        &Impl::mapped_mem_range,
        &Impl::mapped_mem
    } {}
};

using VtopRange = CTup2<Address, umem>;

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
 * A page object that is invalid.
 */
constexpr static const Page Page_INVALID = Page{ /* .page_type = */ PageType_UNKNOWN, /* .page_base = */ Address_INVALID, /* .page_size = */ 0 };

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
    typedef typename CGlueC::Context Context;
    void (*virt_to_phys_list)(CGlueC *cont,
                              CSliceRef<VtopRange> addrs,
                              VirtualTranslationCallback out,
                              VirtualTranslationFailCallback out_fail);
    void (*virt_to_phys_range)(CGlueC *cont,
                               Address start,
                               Address end,
                               VirtualTranslationCallback out);
    void (*virt_translation_map_range)(CGlueC *cont,
                                       Address start,
                                       Address end,
                                       VirtualTranslationCallback out);
    void (*virt_page_map_range)(CGlueC *cont,
                                imem gap_size,
                                Address start,
                                Address end,
                                MemoryRangeCallback out);
    int32_t (*virt_to_phys)(CGlueC *cont, Address address, PhysicalAddress *ok_out);
    int32_t (*virt_page_info)(CGlueC *cont, Address addr, Page *ok_out);
    void (*virt_translation_map)(CGlueC *cont, VirtualTranslationCallback out);
    COption<Address> (*phys_to_virt)(CGlueC *cont, Address phys);
    void (*virt_page_map)(CGlueC *cont, imem gap_size, MemoryRangeCallback out);
};

template<typename Impl>
struct VirtualTranslateVtblImpl : VirtualTranslateVtbl<typename Impl::Parent> {
constexpr VirtualTranslateVtblImpl() :
    VirtualTranslateVtbl<typename Impl::Parent> {
        &Impl::virt_to_phys_list,
        &Impl::virt_to_phys_range,
        &Impl::virt_translation_map_range,
        &Impl::virt_page_map_range,
        &Impl::virt_to_phys,
        &Impl::virt_page_info,
        &Impl::virt_translation_map,
        &Impl::phys_to_virt,
        &Impl::virt_page_map
    } {}
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
template<typename CGlueInst = CBox<void>, typename CGlueCtx = CArc<void>>
struct ProcessInstance {
    const MemoryViewVtbl<ProcessInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_memoryview;
    const ProcessVtbl<ProcessInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_process;
    const VirtualTranslateVtbl<ProcessInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_virtualtranslate;
    ProcessInstanceContainer<CGlueInst, CGlueCtx> container;

    ProcessInstance() : container{} , vtbl_memoryview{}, vtbl_process{}, vtbl_virtualtranslate{} {}

    ~ProcessInstance() noexcept {
        mem_drop(std::move(container));
    }

    typedef CGlueCtx Context;

    inline int32_t read_raw_iter(ReadRawMemOps data) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->read_raw_iter(&this->container, data);
        return __ret;
    }

    inline int32_t write_raw_iter(WriteRawMemOps data) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->write_raw_iter(&this->container, data);
        return __ret;
    }

    inline MemoryViewMetadata metadata() const noexcept {
        MemoryViewMetadata __ret = (this->vtbl_memoryview)->metadata(&this->container);
        return __ret;
    }

    inline int32_t read_iter(CIterator<ReadData> inp, ReadCallback * out, ReadCallback * out_fail) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->read_iter(&this->container, inp, out, out_fail);
        return __ret;
    }

    inline int32_t read_raw_list(CSliceMut<ReadData> data) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->read_raw_list(&this->container, data);
        return __ret;
    }

    inline int32_t read_raw_into(Address addr, CSliceMut<uint8_t> out) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->read_raw_into(&this->container, addr, out);
        return __ret;
    }

    inline int32_t write_iter(CIterator<WriteData> inp, WriteCallback * out, WriteCallback * out_fail) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->write_iter(&this->container, inp, out, out_fail);
        return __ret;
    }

    inline int32_t write_raw_list(CSliceRef<WriteData> data) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->write_raw_list(&this->container, data);
        return __ret;
    }

    inline int32_t write_raw(Address addr, CSliceRef<uint8_t> data) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->write_raw(&this->container, addr, data);
        return __ret;
    }

    inline ProcessState state() noexcept {
        ProcessState __ret = (this->vtbl_process)->state(&this->container);
        return __ret;
    }

    inline int32_t set_dtb(Address dtb1, Address dtb2) noexcept {
        int32_t __ret = (this->vtbl_process)->set_dtb(&this->container, dtb1, dtb2);
        return __ret;
    }

    inline int32_t module_address_list_callback(const ArchitectureIdent * target_arch, ModuleAddressCallback callback) noexcept {
        int32_t __ret = (this->vtbl_process)->module_address_list_callback(&this->container, target_arch, callback);
        return __ret;
    }

    inline int32_t module_list_callback(const ArchitectureIdent * target_arch, ModuleInfoCallback callback) noexcept {
        int32_t __ret = (this->vtbl_process)->module_list_callback(&this->container, target_arch, callback);
        return __ret;
    }

    inline int32_t module_by_address(Address address, ArchitectureIdent architecture, ModuleInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_process)->module_by_address(&this->container, address, architecture, ok_out);
        return __ret;
    }

    inline int32_t module_by_name_arch(CSliceRef<uint8_t> name, const ArchitectureIdent * architecture, ModuleInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_process)->module_by_name_arch(&this->container, name, architecture, ok_out);
        return __ret;
    }

    inline int32_t module_by_name(CSliceRef<uint8_t> name, ModuleInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_process)->module_by_name(&this->container, name, ok_out);
        return __ret;
    }

    inline int32_t primary_module_address(Address * ok_out) noexcept {
        int32_t __ret = (this->vtbl_process)->primary_module_address(&this->container, ok_out);
        return __ret;
    }

    inline int32_t primary_module(ModuleInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_process)->primary_module(&this->container, ok_out);
        return __ret;
    }

    inline int32_t module_import_list_callback(const ModuleInfo * info, ImportCallback callback) noexcept {
        int32_t __ret = (this->vtbl_process)->module_import_list_callback(&this->container, info, callback);
        return __ret;
    }

    inline int32_t module_export_list_callback(const ModuleInfo * info, ExportCallback callback) noexcept {
        int32_t __ret = (this->vtbl_process)->module_export_list_callback(&this->container, info, callback);
        return __ret;
    }

    inline int32_t module_section_list_callback(const ModuleInfo * info, SectionCallback callback) noexcept {
        int32_t __ret = (this->vtbl_process)->module_section_list_callback(&this->container, info, callback);
        return __ret;
    }

    inline int32_t module_import_by_name(const ModuleInfo * info, CSliceRef<uint8_t> name, ImportInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_process)->module_import_by_name(&this->container, info, name, ok_out);
        return __ret;
    }

    inline int32_t module_export_by_name(const ModuleInfo * info, CSliceRef<uint8_t> name, ExportInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_process)->module_export_by_name(&this->container, info, name, ok_out);
        return __ret;
    }

    inline int32_t module_section_by_name(const ModuleInfo * info, CSliceRef<uint8_t> name, SectionInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_process)->module_section_by_name(&this->container, info, name, ok_out);
        return __ret;
    }

    inline const ProcessInfo * info() const noexcept {
        const ProcessInfo * __ret = (this->vtbl_process)->info(&this->container);
        return __ret;
    }

    inline void mapped_mem_range(imem gap_size, Address start, Address end, MemoryRangeCallback out) noexcept {
    (this->vtbl_process)->mapped_mem_range(&this->container, gap_size, start, end, out);

    }

    inline void mapped_mem(imem gap_size, MemoryRangeCallback out) noexcept {
    (this->vtbl_process)->mapped_mem(&this->container, gap_size, out);

    }

    inline void virt_to_phys_list(CSliceRef<VtopRange> addrs, VirtualTranslationCallback out, VirtualTranslationFailCallback out_fail) noexcept {
    (this->vtbl_virtualtranslate)->virt_to_phys_list(&this->container, addrs, out, out_fail);

    }

    inline void virt_to_phys_range(Address start, Address end, VirtualTranslationCallback out) noexcept {
    (this->vtbl_virtualtranslate)->virt_to_phys_range(&this->container, start, end, out);

    }

    inline void virt_translation_map_range(Address start, Address end, VirtualTranslationCallback out) noexcept {
    (this->vtbl_virtualtranslate)->virt_translation_map_range(&this->container, start, end, out);

    }

    inline void virt_page_map_range(imem gap_size, Address start, Address end, MemoryRangeCallback out) noexcept {
    (this->vtbl_virtualtranslate)->virt_page_map_range(&this->container, gap_size, start, end, out);

    }

    inline int32_t virt_to_phys(Address address, PhysicalAddress * ok_out) noexcept {
        int32_t __ret = (this->vtbl_virtualtranslate)->virt_to_phys(&this->container, address, ok_out);
        return __ret;
    }

    inline int32_t virt_page_info(Address addr, Page * ok_out) noexcept {
        int32_t __ret = (this->vtbl_virtualtranslate)->virt_page_info(&this->container, addr, ok_out);
        return __ret;
    }

    inline void virt_translation_map(VirtualTranslationCallback out) noexcept {
    (this->vtbl_virtualtranslate)->virt_translation_map(&this->container, out);

    }

    inline COption<Address> phys_to_virt(Address phys) noexcept {
        COption<Address> __ret = (this->vtbl_virtualtranslate)->phys_to_virt(&this->container, phys);
        return __ret;
    }

    inline void virt_page_map(imem gap_size, MemoryRangeCallback out) noexcept {
    (this->vtbl_virtualtranslate)->virt_page_map(&this->container, gap_size, out);

    }

};

template<typename CGlueInst = CBox<void>, typename CGlueCtx = CArc<void>>
struct IntoProcessInstanceContainer {
    typedef CGlueCtx Context;
    CGlueInst instance;
    CGlueCtx context;

    inline Context clone_context() noexcept {
        return context.clone();
    }

    inline void drop() && noexcept {
        mem_drop(std::move(instance));
        mem_drop(std::move(context));
    }

    inline void forget() noexcept {
        mem_forget(instance);
        mem_forget(context);
    }
};

template<typename CGlueInst>
struct IntoProcessInstanceContainer<CGlueInst, void> {
    typedef void Context;
    CGlueInst instance;

    inline Context clone_context() noexcept {}

    inline void drop() && noexcept {
        mem_drop(std::move(instance));
    }

    inline void forget() noexcept {
        mem_forget(instance);
    }
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
template<typename CGlueInst = CBox<void>, typename CGlueCtx = CArc<void>>
struct IntoProcessInstance {
    const CloneVtbl<IntoProcessInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_clone;
    const MemoryViewVtbl<IntoProcessInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_memoryview;
    const ProcessVtbl<IntoProcessInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_process;
    const VirtualTranslateVtbl<IntoProcessInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_virtualtranslate;
    IntoProcessInstanceContainer<CGlueInst, CGlueCtx> container;

    IntoProcessInstance() : container{} , vtbl_clone{}, vtbl_memoryview{}, vtbl_process{}, vtbl_virtualtranslate{} {}

    ~IntoProcessInstance() noexcept {
        mem_drop(std::move(container));
    }

    typedef CGlueCtx Context;

    inline IntoProcessInstance clone() const noexcept {
        IntoProcessInstance __ret;
            __ret.vtbl_clone = this->vtbl_clone;
            __ret.vtbl_memoryview = this->vtbl_memoryview;
            __ret.vtbl_process = this->vtbl_process;
            __ret.vtbl_virtualtranslate = this->vtbl_virtualtranslate;
        __ret.container = (this->vtbl_clone)->clone(&this->container);
        return __ret;
    }

    inline int32_t read_raw_iter(ReadRawMemOps data) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->read_raw_iter(&this->container, data);
        return __ret;
    }

    inline int32_t write_raw_iter(WriteRawMemOps data) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->write_raw_iter(&this->container, data);
        return __ret;
    }

    inline MemoryViewMetadata metadata() const noexcept {
        MemoryViewMetadata __ret = (this->vtbl_memoryview)->metadata(&this->container);
        return __ret;
    }

    inline int32_t read_iter(CIterator<ReadData> inp, ReadCallback * out, ReadCallback * out_fail) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->read_iter(&this->container, inp, out, out_fail);
        return __ret;
    }

    inline int32_t read_raw_list(CSliceMut<ReadData> data) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->read_raw_list(&this->container, data);
        return __ret;
    }

    inline int32_t read_raw_into(Address addr, CSliceMut<uint8_t> out) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->read_raw_into(&this->container, addr, out);
        return __ret;
    }

    inline int32_t write_iter(CIterator<WriteData> inp, WriteCallback * out, WriteCallback * out_fail) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->write_iter(&this->container, inp, out, out_fail);
        return __ret;
    }

    inline int32_t write_raw_list(CSliceRef<WriteData> data) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->write_raw_list(&this->container, data);
        return __ret;
    }

    inline int32_t write_raw(Address addr, CSliceRef<uint8_t> data) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->write_raw(&this->container, addr, data);
        return __ret;
    }

    inline ProcessState state() noexcept {
        ProcessState __ret = (this->vtbl_process)->state(&this->container);
        return __ret;
    }

    inline int32_t set_dtb(Address dtb1, Address dtb2) noexcept {
        int32_t __ret = (this->vtbl_process)->set_dtb(&this->container, dtb1, dtb2);
        return __ret;
    }

    inline int32_t module_address_list_callback(const ArchitectureIdent * target_arch, ModuleAddressCallback callback) noexcept {
        int32_t __ret = (this->vtbl_process)->module_address_list_callback(&this->container, target_arch, callback);
        return __ret;
    }

    inline int32_t module_list_callback(const ArchitectureIdent * target_arch, ModuleInfoCallback callback) noexcept {
        int32_t __ret = (this->vtbl_process)->module_list_callback(&this->container, target_arch, callback);
        return __ret;
    }

    inline int32_t module_by_address(Address address, ArchitectureIdent architecture, ModuleInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_process)->module_by_address(&this->container, address, architecture, ok_out);
        return __ret;
    }

    inline int32_t module_by_name_arch(CSliceRef<uint8_t> name, const ArchitectureIdent * architecture, ModuleInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_process)->module_by_name_arch(&this->container, name, architecture, ok_out);
        return __ret;
    }

    inline int32_t module_by_name(CSliceRef<uint8_t> name, ModuleInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_process)->module_by_name(&this->container, name, ok_out);
        return __ret;
    }

    inline int32_t primary_module_address(Address * ok_out) noexcept {
        int32_t __ret = (this->vtbl_process)->primary_module_address(&this->container, ok_out);
        return __ret;
    }

    inline int32_t primary_module(ModuleInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_process)->primary_module(&this->container, ok_out);
        return __ret;
    }

    inline int32_t module_import_list_callback(const ModuleInfo * info, ImportCallback callback) noexcept {
        int32_t __ret = (this->vtbl_process)->module_import_list_callback(&this->container, info, callback);
        return __ret;
    }

    inline int32_t module_export_list_callback(const ModuleInfo * info, ExportCallback callback) noexcept {
        int32_t __ret = (this->vtbl_process)->module_export_list_callback(&this->container, info, callback);
        return __ret;
    }

    inline int32_t module_section_list_callback(const ModuleInfo * info, SectionCallback callback) noexcept {
        int32_t __ret = (this->vtbl_process)->module_section_list_callback(&this->container, info, callback);
        return __ret;
    }

    inline int32_t module_import_by_name(const ModuleInfo * info, CSliceRef<uint8_t> name, ImportInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_process)->module_import_by_name(&this->container, info, name, ok_out);
        return __ret;
    }

    inline int32_t module_export_by_name(const ModuleInfo * info, CSliceRef<uint8_t> name, ExportInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_process)->module_export_by_name(&this->container, info, name, ok_out);
        return __ret;
    }

    inline int32_t module_section_by_name(const ModuleInfo * info, CSliceRef<uint8_t> name, SectionInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_process)->module_section_by_name(&this->container, info, name, ok_out);
        return __ret;
    }

    inline const ProcessInfo * info() const noexcept {
        const ProcessInfo * __ret = (this->vtbl_process)->info(&this->container);
        return __ret;
    }

    inline void mapped_mem_range(imem gap_size, Address start, Address end, MemoryRangeCallback out) noexcept {
    (this->vtbl_process)->mapped_mem_range(&this->container, gap_size, start, end, out);

    }

    inline void mapped_mem(imem gap_size, MemoryRangeCallback out) noexcept {
    (this->vtbl_process)->mapped_mem(&this->container, gap_size, out);

    }

    inline void virt_to_phys_list(CSliceRef<VtopRange> addrs, VirtualTranslationCallback out, VirtualTranslationFailCallback out_fail) noexcept {
    (this->vtbl_virtualtranslate)->virt_to_phys_list(&this->container, addrs, out, out_fail);

    }

    inline void virt_to_phys_range(Address start, Address end, VirtualTranslationCallback out) noexcept {
    (this->vtbl_virtualtranslate)->virt_to_phys_range(&this->container, start, end, out);

    }

    inline void virt_translation_map_range(Address start, Address end, VirtualTranslationCallback out) noexcept {
    (this->vtbl_virtualtranslate)->virt_translation_map_range(&this->container, start, end, out);

    }

    inline void virt_page_map_range(imem gap_size, Address start, Address end, MemoryRangeCallback out) noexcept {
    (this->vtbl_virtualtranslate)->virt_page_map_range(&this->container, gap_size, start, end, out);

    }

    inline int32_t virt_to_phys(Address address, PhysicalAddress * ok_out) noexcept {
        int32_t __ret = (this->vtbl_virtualtranslate)->virt_to_phys(&this->container, address, ok_out);
        return __ret;
    }

    inline int32_t virt_page_info(Address addr, Page * ok_out) noexcept {
        int32_t __ret = (this->vtbl_virtualtranslate)->virt_page_info(&this->container, addr, ok_out);
        return __ret;
    }

    inline void virt_translation_map(VirtualTranslationCallback out) noexcept {
    (this->vtbl_virtualtranslate)->virt_translation_map(&this->container, out);

    }

    inline COption<Address> phys_to_virt(Address phys) noexcept {
        COption<Address> __ret = (this->vtbl_virtualtranslate)->phys_to_virt(&this->container, phys);
        return __ret;
    }

    inline void virt_page_map(imem gap_size, MemoryRangeCallback out) noexcept {
    (this->vtbl_virtualtranslate)->virt_page_map(&this->container, gap_size, out);

    }

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
 * CGlue vtable for trait Os.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct OsVtbl {
    typedef typename CGlueC::Context Context;
    int32_t (*process_address_list_callback)(CGlueC *cont, AddressCallback callback);
    int32_t (*process_info_list_callback)(CGlueC *cont, ProcessInfoCallback callback);
    int32_t (*process_info_by_address)(CGlueC *cont,
                                       Address address,
                                       ProcessInfo *ok_out);
    int32_t (*process_info_by_name)(CGlueC *cont,
                                    CSliceRef<uint8_t> name,
                                    ProcessInfo *ok_out);
    int32_t (*process_info_by_pid)(CGlueC *cont, Pid pid, ProcessInfo *ok_out);
    int32_t (*process_by_info)(CGlueC *cont,
                               ProcessInfo info,
                               ProcessInstance<CBox<void>, Context> *ok_out);
    int32_t (*into_process_by_info)(CGlueC cont,
                                    ProcessInfo info,
                                    IntoProcessInstance<CBox<void>, Context> *ok_out);
    int32_t (*process_by_address)(CGlueC *cont,
                                  Address addr,
                                  ProcessInstance<CBox<void>, Context> *ok_out);
    int32_t (*process_by_name)(CGlueC *cont,
                               CSliceRef<uint8_t> name,
                               ProcessInstance<CBox<void>, Context> *ok_out);
    int32_t (*process_by_pid)(CGlueC *cont,
                              Pid pid,
                              ProcessInstance<CBox<void>, Context> *ok_out);
    int32_t (*into_process_by_address)(CGlueC cont,
                                       Address addr,
                                       IntoProcessInstance<CBox<void>, Context> *ok_out);
    int32_t (*into_process_by_name)(CGlueC cont,
                                    CSliceRef<uint8_t> name,
                                    IntoProcessInstance<CBox<void>, Context> *ok_out);
    int32_t (*into_process_by_pid)(CGlueC cont,
                                   Pid pid,
                                   IntoProcessInstance<CBox<void>, Context> *ok_out);
    int32_t (*module_address_list_callback)(CGlueC *cont, AddressCallback callback);
    int32_t (*module_list_callback)(CGlueC *cont, ModuleInfoCallback callback);
    int32_t (*module_by_address)(CGlueC *cont, Address address, ModuleInfo *ok_out);
    int32_t (*module_by_name)(CGlueC *cont, CSliceRef<uint8_t> name, ModuleInfo *ok_out);
    int32_t (*primary_module_address)(CGlueC *cont, Address *ok_out);
    int32_t (*primary_module)(CGlueC *cont, ModuleInfo *ok_out);
    int32_t (*module_import_list_callback)(CGlueC *cont,
                                           const ModuleInfo *info,
                                           ImportCallback callback);
    int32_t (*module_export_list_callback)(CGlueC *cont,
                                           const ModuleInfo *info,
                                           ExportCallback callback);
    int32_t (*module_section_list_callback)(CGlueC *cont,
                                            const ModuleInfo *info,
                                            SectionCallback callback);
    int32_t (*module_import_by_name)(CGlueC *cont,
                                     const ModuleInfo *info,
                                     CSliceRef<uint8_t> name,
                                     ImportInfo *ok_out);
    int32_t (*module_export_by_name)(CGlueC *cont,
                                     const ModuleInfo *info,
                                     CSliceRef<uint8_t> name,
                                     ExportInfo *ok_out);
    int32_t (*module_section_by_name)(CGlueC *cont,
                                      const ModuleInfo *info,
                                      CSliceRef<uint8_t> name,
                                      SectionInfo *ok_out);
    const OsInfo *(*info)(const CGlueC *cont);
};

template<typename Impl>
struct OsVtblImpl : OsVtbl<typename Impl::Parent> {
constexpr OsVtblImpl() :
    OsVtbl<typename Impl::Parent> {
        &Impl::process_address_list_callback,
        &Impl::process_info_list_callback,
        &Impl::process_info_by_address,
        &Impl::process_info_by_name,
        &Impl::process_info_by_pid,
        &Impl::process_by_info,
        &Impl::into_process_by_info,
        &Impl::process_by_address,
        &Impl::process_by_name,
        &Impl::process_by_pid,
        &Impl::into_process_by_address,
        &Impl::into_process_by_name,
        &Impl::into_process_by_pid,
        &Impl::module_address_list_callback,
        &Impl::module_list_callback,
        &Impl::module_by_address,
        &Impl::module_by_name,
        &Impl::primary_module_address,
        &Impl::primary_module,
        &Impl::module_import_list_callback,
        &Impl::module_export_list_callback,
        &Impl::module_section_list_callback,
        &Impl::module_import_by_name,
        &Impl::module_export_by_name,
        &Impl::module_section_by_name,
        &Impl::info
    } {}
};

/**
 * CGlue vtable for trait KeyboardState.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct KeyboardStateVtbl {
    typedef typename CGlueC::Context Context;
    bool (*is_down)(const CGlueC *cont, int32_t vk);
};

template<typename Impl>
struct KeyboardStateVtblImpl : KeyboardStateVtbl<typename Impl::Parent> {
constexpr KeyboardStateVtblImpl() :
    KeyboardStateVtbl<typename Impl::Parent> {
        &Impl::is_down
    } {}
};

/**
 * Base CGlue trait object for trait KeyboardState.
 */
template<typename CGlueInst = CBox<void>, typename CGlueCtx = CArc<void>>
using KeyboardStateBase = CGlueTraitObj<CGlueInst, KeyboardStateVtbl<CGlueObjContainer<CGlueInst, CGlueCtx, KeyboardStateRetTmp<CGlueCtx>>>, CGlueCtx, KeyboardStateRetTmp<CGlueCtx>>;

/**
 * CGlue vtable for trait Keyboard.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct KeyboardVtbl {
    typedef typename CGlueC::Context Context;
    bool (*is_down)(CGlueC *cont, int32_t vk);
    void (*set_down)(CGlueC *cont, int32_t vk, bool down);
    int32_t (*state)(CGlueC *cont, KeyboardStateBase<CBox<void>, Context> *ok_out);
};

template<typename Impl>
struct KeyboardVtblImpl : KeyboardVtbl<typename Impl::Parent> {
constexpr KeyboardVtblImpl() :
    KeyboardVtbl<typename Impl::Parent> {
        &Impl::is_down,
        &Impl::set_down,
        &Impl::state
    } {}
};

/**
 * Base CGlue trait object for trait Keyboard.
 */
template<typename CGlueInst = CBox<void>, typename CGlueCtx = CArc<void>>
using KeyboardBase = CGlueTraitObj<CGlueInst, KeyboardVtbl<CGlueObjContainer<CGlueInst, CGlueCtx, KeyboardRetTmp<CGlueCtx>>>, CGlueCtx, KeyboardRetTmp<CGlueCtx>>;

template<typename CGlueInst = CBox<void>, typename CGlueCtx = CArc<void>>
struct IntoKeyboardContainer {
    typedef CGlueCtx Context;
    CGlueInst instance;
    CGlueCtx context;

    inline Context clone_context() noexcept {
        return context.clone();
    }

    inline void drop() && noexcept {
        mem_drop(std::move(instance));
        mem_drop(std::move(context));
    }

    inline void forget() noexcept {
        mem_forget(instance);
        mem_forget(context);
    }
};

template<typename CGlueInst>
struct IntoKeyboardContainer<CGlueInst, void> {
    typedef void Context;
    CGlueInst instance;

    inline Context clone_context() noexcept {}

    inline void drop() && noexcept {
        mem_drop(std::move(instance));
    }

    inline void forget() noexcept {
        mem_forget(instance);
    }
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
template<typename CGlueInst = CBox<void>, typename CGlueCtx = CArc<void>>
struct IntoKeyboard {
    const CloneVtbl<IntoKeyboardContainer<CGlueInst, CGlueCtx>> *vtbl_clone;
    const KeyboardVtbl<IntoKeyboardContainer<CGlueInst, CGlueCtx>> *vtbl_keyboard;
    IntoKeyboardContainer<CGlueInst, CGlueCtx> container;

    IntoKeyboard() : container{} , vtbl_clone{}, vtbl_keyboard{} {}

    ~IntoKeyboard() noexcept {
        mem_drop(std::move(container));
    }

    typedef CGlueCtx Context;

    inline IntoKeyboard clone() const noexcept {
        IntoKeyboard __ret;
            __ret.vtbl_clone = this->vtbl_clone;
            __ret.vtbl_keyboard = this->vtbl_keyboard;
        __ret.container = (this->vtbl_clone)->clone(&this->container);
        return __ret;
    }

    inline bool is_down(int32_t vk) noexcept {
        bool __ret = (this->vtbl_keyboard)->is_down(&this->container, vk);
        return __ret;
    }

    inline void set_down(int32_t vk, bool down) noexcept {
    (this->vtbl_keyboard)->set_down(&this->container, vk, down);

    }

    inline int32_t state(KeyboardStateBase<CBox<void>, Context> * ok_out) noexcept {
        int32_t __ret = (this->vtbl_keyboard)->state(&this->container, ok_out);
        return __ret;
    }

};

/**
 * CGlue vtable for trait OsKeyboard.
 *
 * This virtual function table contains ABI-safe interface for the given trait.
 */
template<typename CGlueC>
struct OsKeyboardVtbl {
    typedef typename CGlueC::Context Context;
    int32_t (*keyboard)(CGlueC *cont, KeyboardBase<CBox<void>, Context> *ok_out);
    int32_t (*into_keyboard)(CGlueC cont, IntoKeyboard<CBox<void>, Context> *ok_out);
};

template<typename Impl>
struct OsKeyboardVtblImpl : OsKeyboardVtbl<typename Impl::Parent> {
constexpr OsKeyboardVtblImpl() :
    OsKeyboardVtbl<typename Impl::Parent> {
        &Impl::keyboard,
        &Impl::into_keyboard
    } {}
};

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
template<typename CGlueInst = CBox<void>, typename CGlueCtx = CArc<void>>
struct OsInstance {
    const CloneVtbl<OsInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_clone;
    const OsVtbl<OsInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_os;
    const MemoryViewVtbl<OsInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_memoryview;
    const OsKeyboardVtbl<OsInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_oskeyboard;
    const PhysicalMemoryVtbl<OsInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_physicalmemory;
    const VirtualTranslateVtbl<OsInstanceContainer<CGlueInst, CGlueCtx>> *vtbl_virtualtranslate;
    OsInstanceContainer<CGlueInst, CGlueCtx> container;

    OsInstance() : container{} , vtbl_clone{}, vtbl_os{}, vtbl_memoryview{}, vtbl_oskeyboard{}, vtbl_physicalmemory{}, vtbl_virtualtranslate{} {}

    ~OsInstance() noexcept {
        mem_drop(std::move(container));
    }

    typedef CGlueCtx Context;

    inline OsInstance clone() const noexcept {
        OsInstance __ret;
            __ret.vtbl_clone = this->vtbl_clone;
            __ret.vtbl_os = this->vtbl_os;
            __ret.vtbl_memoryview = this->vtbl_memoryview;
            __ret.vtbl_oskeyboard = this->vtbl_oskeyboard;
            __ret.vtbl_physicalmemory = this->vtbl_physicalmemory;
            __ret.vtbl_virtualtranslate = this->vtbl_virtualtranslate;
        __ret.container = (this->vtbl_clone)->clone(&this->container);
        return __ret;
    }

    inline int32_t process_address_list_callback(AddressCallback callback) noexcept {
        int32_t __ret = (this->vtbl_os)->process_address_list_callback(&this->container, callback);
        return __ret;
    }

    inline int32_t process_info_list_callback(ProcessInfoCallback callback) noexcept {
        int32_t __ret = (this->vtbl_os)->process_info_list_callback(&this->container, callback);
        return __ret;
    }

    inline int32_t process_info_by_address(Address address, ProcessInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_os)->process_info_by_address(&this->container, address, ok_out);
        return __ret;
    }

    inline int32_t process_info_by_name(CSliceRef<uint8_t> name, ProcessInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_os)->process_info_by_name(&this->container, name, ok_out);
        return __ret;
    }

    inline int32_t process_info_by_pid(Pid pid, ProcessInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_os)->process_info_by_pid(&this->container, pid, ok_out);
        return __ret;
    }

    inline int32_t process_by_info(ProcessInfo info, ProcessInstance<CBox<void>, Context> * ok_out) noexcept {
        int32_t __ret = (this->vtbl_os)->process_by_info(&this->container, info, ok_out);
        return __ret;
    }

    inline int32_t into_process_by_info(ProcessInfo info, IntoProcessInstance<CBox<void>, Context> * ok_out) && noexcept {
        auto ___ctx = StoreAll()[this->container.clone_context(), StoreAll()];
        int32_t __ret = (this->vtbl_os)->into_process_by_info(this->container, info, ok_out);
        mem_forget(this->container);
        return __ret;
    }

    inline int32_t process_by_address(Address addr, ProcessInstance<CBox<void>, Context> * ok_out) noexcept {
        int32_t __ret = (this->vtbl_os)->process_by_address(&this->container, addr, ok_out);
        return __ret;
    }

    inline int32_t process_by_name(CSliceRef<uint8_t> name, ProcessInstance<CBox<void>, Context> * ok_out) noexcept {
        int32_t __ret = (this->vtbl_os)->process_by_name(&this->container, name, ok_out);
        return __ret;
    }

    inline int32_t process_by_pid(Pid pid, ProcessInstance<CBox<void>, Context> * ok_out) noexcept {
        int32_t __ret = (this->vtbl_os)->process_by_pid(&this->container, pid, ok_out);
        return __ret;
    }

    inline int32_t into_process_by_address(Address addr, IntoProcessInstance<CBox<void>, Context> * ok_out) && noexcept {
        auto ___ctx = StoreAll()[this->container.clone_context(), StoreAll()];
        int32_t __ret = (this->vtbl_os)->into_process_by_address(this->container, addr, ok_out);
        mem_forget(this->container);
        return __ret;
    }

    inline int32_t into_process_by_name(CSliceRef<uint8_t> name, IntoProcessInstance<CBox<void>, Context> * ok_out) && noexcept {
        auto ___ctx = StoreAll()[this->container.clone_context(), StoreAll()];
        int32_t __ret = (this->vtbl_os)->into_process_by_name(this->container, name, ok_out);
        mem_forget(this->container);
        return __ret;
    }

    inline int32_t into_process_by_pid(Pid pid, IntoProcessInstance<CBox<void>, Context> * ok_out) && noexcept {
        auto ___ctx = StoreAll()[this->container.clone_context(), StoreAll()];
        int32_t __ret = (this->vtbl_os)->into_process_by_pid(this->container, pid, ok_out);
        mem_forget(this->container);
        return __ret;
    }

    inline int32_t module_address_list_callback(AddressCallback callback) noexcept {
        int32_t __ret = (this->vtbl_os)->module_address_list_callback(&this->container, callback);
        return __ret;
    }

    inline int32_t module_list_callback(ModuleInfoCallback callback) noexcept {
        int32_t __ret = (this->vtbl_os)->module_list_callback(&this->container, callback);
        return __ret;
    }

    inline int32_t module_by_address(Address address, ModuleInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_os)->module_by_address(&this->container, address, ok_out);
        return __ret;
    }

    inline int32_t module_by_name(CSliceRef<uint8_t> name, ModuleInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_os)->module_by_name(&this->container, name, ok_out);
        return __ret;
    }

    inline int32_t primary_module_address(Address * ok_out) noexcept {
        int32_t __ret = (this->vtbl_os)->primary_module_address(&this->container, ok_out);
        return __ret;
    }

    inline int32_t primary_module(ModuleInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_os)->primary_module(&this->container, ok_out);
        return __ret;
    }

    inline int32_t module_import_list_callback(const ModuleInfo * info, ImportCallback callback) noexcept {
        int32_t __ret = (this->vtbl_os)->module_import_list_callback(&this->container, info, callback);
        return __ret;
    }

    inline int32_t module_export_list_callback(const ModuleInfo * info, ExportCallback callback) noexcept {
        int32_t __ret = (this->vtbl_os)->module_export_list_callback(&this->container, info, callback);
        return __ret;
    }

    inline int32_t module_section_list_callback(const ModuleInfo * info, SectionCallback callback) noexcept {
        int32_t __ret = (this->vtbl_os)->module_section_list_callback(&this->container, info, callback);
        return __ret;
    }

    inline int32_t module_import_by_name(const ModuleInfo * info, CSliceRef<uint8_t> name, ImportInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_os)->module_import_by_name(&this->container, info, name, ok_out);
        return __ret;
    }

    inline int32_t module_export_by_name(const ModuleInfo * info, CSliceRef<uint8_t> name, ExportInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_os)->module_export_by_name(&this->container, info, name, ok_out);
        return __ret;
    }

    inline int32_t module_section_by_name(const ModuleInfo * info, CSliceRef<uint8_t> name, SectionInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl_os)->module_section_by_name(&this->container, info, name, ok_out);
        return __ret;
    }

    inline const OsInfo * info() const noexcept {
        const OsInfo * __ret = (this->vtbl_os)->info(&this->container);
        return __ret;
    }

    inline int32_t read_raw_iter(ReadRawMemOps data) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->read_raw_iter(&this->container, data);
        return __ret;
    }

    inline int32_t write_raw_iter(WriteRawMemOps data) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->write_raw_iter(&this->container, data);
        return __ret;
    }

    inline MemoryViewMetadata memoryview_metadata() const noexcept {
        MemoryViewMetadata __ret = (this->vtbl_memoryview)->metadata(&this->container);
        return __ret;
    }

    inline int32_t read_iter(CIterator<ReadData> inp, ReadCallback * out, ReadCallback * out_fail) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->read_iter(&this->container, inp, out, out_fail);
        return __ret;
    }

    inline int32_t read_raw_list(CSliceMut<ReadData> data) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->read_raw_list(&this->container, data);
        return __ret;
    }

    inline int32_t read_raw_into(Address addr, CSliceMut<uint8_t> out) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->read_raw_into(&this->container, addr, out);
        return __ret;
    }

    inline int32_t write_iter(CIterator<WriteData> inp, WriteCallback * out, WriteCallback * out_fail) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->write_iter(&this->container, inp, out, out_fail);
        return __ret;
    }

    inline int32_t write_raw_list(CSliceRef<WriteData> data) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->write_raw_list(&this->container, data);
        return __ret;
    }

    inline int32_t write_raw(Address addr, CSliceRef<uint8_t> data) noexcept {
        int32_t __ret = (this->vtbl_memoryview)->write_raw(&this->container, addr, data);
        return __ret;
    }

    inline int32_t keyboard(KeyboardBase<CBox<void>, Context> * ok_out) noexcept {
        int32_t __ret = (this->vtbl_oskeyboard)->keyboard(&this->container, ok_out);
        return __ret;
    }

    inline int32_t into_keyboard(IntoKeyboard<CBox<void>, Context> * ok_out) && noexcept {
        auto ___ctx = StoreAll()[this->container.clone_context(), StoreAll()];
        int32_t __ret = (this->vtbl_oskeyboard)->into_keyboard(this->container, ok_out);
        mem_forget(this->container);
        return __ret;
    }

    inline int32_t phys_read_raw_iter(PhysicalReadMemOps data) noexcept {
        int32_t __ret = (this->vtbl_physicalmemory)->phys_read_raw_iter(&this->container, data);
        return __ret;
    }

    inline int32_t phys_write_raw_iter(PhysicalWriteMemOps data) noexcept {
        int32_t __ret = (this->vtbl_physicalmemory)->phys_write_raw_iter(&this->container, data);
        return __ret;
    }

    inline PhysicalMemoryMetadata physicalmemory_metadata() const noexcept {
        PhysicalMemoryMetadata __ret = (this->vtbl_physicalmemory)->metadata(&this->container);
        return __ret;
    }

    inline void set_mem_map(CSliceRef<PhysicalMemoryMapping> _mem_map) noexcept {
    (this->vtbl_physicalmemory)->set_mem_map(&this->container, _mem_map);

    }

    inline MemoryViewBase<CBox<void>, Context> into_phys_view() && noexcept {
        auto ___ctx = StoreAll()[this->container.clone_context(), StoreAll()];
        MemoryViewBase<CBox<void>, Context> __ret = (this->vtbl_physicalmemory)->into_phys_view(this->container);
        mem_forget(this->container);
        return __ret;
    }

    inline MemoryViewBase<CBox<void>, Context> phys_view() noexcept {
        MemoryViewBase<CBox<void>, Context> __ret = (this->vtbl_physicalmemory)->phys_view(&this->container);
        return __ret;
    }

    inline void virt_to_phys_list(CSliceRef<VtopRange> addrs, VirtualTranslationCallback out, VirtualTranslationFailCallback out_fail) noexcept {
    (this->vtbl_virtualtranslate)->virt_to_phys_list(&this->container, addrs, out, out_fail);

    }

    inline void virt_to_phys_range(Address start, Address end, VirtualTranslationCallback out) noexcept {
    (this->vtbl_virtualtranslate)->virt_to_phys_range(&this->container, start, end, out);

    }

    inline void virt_translation_map_range(Address start, Address end, VirtualTranslationCallback out) noexcept {
    (this->vtbl_virtualtranslate)->virt_translation_map_range(&this->container, start, end, out);

    }

    inline void virt_page_map_range(imem gap_size, Address start, Address end, MemoryRangeCallback out) noexcept {
    (this->vtbl_virtualtranslate)->virt_page_map_range(&this->container, gap_size, start, end, out);

    }

    inline int32_t virt_to_phys(Address address, PhysicalAddress * ok_out) noexcept {
        int32_t __ret = (this->vtbl_virtualtranslate)->virt_to_phys(&this->container, address, ok_out);
        return __ret;
    }

    inline int32_t virt_page_info(Address addr, Page * ok_out) noexcept {
        int32_t __ret = (this->vtbl_virtualtranslate)->virt_page_info(&this->container, addr, ok_out);
        return __ret;
    }

    inline void virt_translation_map(VirtualTranslationCallback out) noexcept {
    (this->vtbl_virtualtranslate)->virt_translation_map(&this->container, out);

    }

    inline COption<Address> phys_to_virt(Address phys) noexcept {
        COption<Address> __ret = (this->vtbl_virtualtranslate)->phys_to_virt(&this->container, phys);
        return __ret;
    }

    inline void virt_page_map(imem gap_size, MemoryRangeCallback out) noexcept {
    (this->vtbl_virtualtranslate)->virt_page_map(&this->container, gap_size, out);

    }

};

template<typename CGlueT, typename CGlueCtx = CArc<void>>
using OsInstanceBaseCtxBox = OsInstance<CBox<CGlueT>, CGlueCtx>;

template<typename CGlueT, typename CGlueArcTy>
using OsInstanceBaseArcBox = OsInstanceBaseCtxBox<CGlueT, CArc<CGlueArcTy>>;
// Typedef for default contaienr and context type
template<typename CGlueT, typename CGlueArcTy>
using OsInstanceBase = OsInstanceBaseArcBox<CGlueT,CGlueArcTy>;

using OsInstanceArcBox = OsInstanceBaseArcBox<void, void>;

using MuOsInstanceArcBox = OsInstanceArcBox;
// Typedef for default contaienr and context type
using MuOsInstance = MuOsInstanceArcBox;

template<typename CGlueT, typename CGlueCtx = CArc<void>>
using ProcessInstanceBaseCtxBox = ProcessInstance<CBox<CGlueT>, CGlueCtx>;

template<typename CGlueT, typename CGlueArcTy>
using ProcessInstanceBaseArcBox = ProcessInstanceBaseCtxBox<CGlueT, CArc<CGlueArcTy>>;
// Typedef for default contaienr and context type
template<typename CGlueT, typename CGlueArcTy>
using ProcessInstanceBase = ProcessInstanceBaseArcBox<CGlueT,CGlueArcTy>;

using ProcessInstanceArcBox = ProcessInstanceBaseArcBox<void, void>;

template<typename CGlueT, typename CGlueCtx = CArc<void>>
using IntoProcessInstanceBaseCtxBox = IntoProcessInstance<CBox<CGlueT>, CGlueCtx>;

template<typename CGlueT, typename CGlueArcTy>
using IntoProcessInstanceBaseArcBox = IntoProcessInstanceBaseCtxBox<CGlueT, CArc<CGlueArcTy>>;
// Typedef for default contaienr and context type
template<typename CGlueT, typename CGlueArcTy>
using IntoProcessInstanceBase = IntoProcessInstanceBaseArcBox<CGlueT,CGlueArcTy>;

using IntoProcessInstanceArcBox = IntoProcessInstanceBaseArcBox<void, void>;

/**
 * CtxBoxed CGlue trait object for trait MemoryView with context.
 */
template<typename CGlueT, typename CGlueCtx = CArc<void>>
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
// Typedef for default contaienr and context type
using MemoryView = MemoryViewArcBox;

extern "C" {

extern const ArchitectureObj *X86_32;

extern const ArchitectureObj *X86_32_PAE;

extern const ArchitectureObj *X86_64;

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
void log_set_max_level(LevelFilter level_filter, const Inventory *inventory);

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


template<typename T, typename C, typename R>
struct CGlueTraitObj<T, CloneVtbl<CGlueObjContainer<T, C, R>>, C, R> {
    const CloneVtbl<CGlueObjContainer<T, C, R>> *vtbl;
    CGlueObjContainer<T, C, R> container;

    CGlueTraitObj() : container{} {}

    ~CGlueTraitObj() noexcept {
        mem_drop(std::move(container));
    }

    typedef C Context;

    inline CGlueTraitObj clone() const noexcept {
        CGlueTraitObj __ret;
            __ret.vtbl = this->vtbl;
        __ret.container = (this->vtbl)->clone(&this->container);
        return __ret;
    }

};

template<typename T, typename C, typename R>
struct CGlueTraitObj<T, MemoryViewVtbl<CGlueObjContainer<T, C, R>>, C, R> {
    const MemoryViewVtbl<CGlueObjContainer<T, C, R>> *vtbl;
    CGlueObjContainer<T, C, R> container;

    CGlueTraitObj() : container{} {}

    ~CGlueTraitObj() noexcept {
        mem_drop(std::move(container));
    }

    typedef C Context;

    inline int32_t read_raw_iter(ReadRawMemOps data) noexcept {
        int32_t __ret = (this->vtbl)->read_raw_iter(&this->container, data);
        return __ret;
    }

    inline int32_t write_raw_iter(WriteRawMemOps data) noexcept {
        int32_t __ret = (this->vtbl)->write_raw_iter(&this->container, data);
        return __ret;
    }

    inline MemoryViewMetadata metadata() const noexcept {
        MemoryViewMetadata __ret = (this->vtbl)->metadata(&this->container);
        return __ret;
    }

    inline int32_t read_iter(CIterator<ReadData> inp, ReadCallback * out, ReadCallback * out_fail) noexcept {
        int32_t __ret = (this->vtbl)->read_iter(&this->container, inp, out, out_fail);
        return __ret;
    }

    inline int32_t read_raw_list(CSliceMut<ReadData> data) noexcept {
        int32_t __ret = (this->vtbl)->read_raw_list(&this->container, data);
        return __ret;
    }

    inline int32_t read_raw_into(Address addr, CSliceMut<uint8_t> out) noexcept {
        int32_t __ret = (this->vtbl)->read_raw_into(&this->container, addr, out);
        return __ret;
    }

    inline int32_t write_iter(CIterator<WriteData> inp, WriteCallback * out, WriteCallback * out_fail) noexcept {
        int32_t __ret = (this->vtbl)->write_iter(&this->container, inp, out, out_fail);
        return __ret;
    }

    inline int32_t write_raw_list(CSliceRef<WriteData> data) noexcept {
        int32_t __ret = (this->vtbl)->write_raw_list(&this->container, data);
        return __ret;
    }

    inline int32_t write_raw(Address addr, CSliceRef<uint8_t> data) noexcept {
        int32_t __ret = (this->vtbl)->write_raw(&this->container, addr, data);
        return __ret;
    }

};

template<typename T, typename C, typename R>
struct CGlueTraitObj<T, PhysicalMemoryVtbl<CGlueObjContainer<T, C, R>>, C, R> {
    const PhysicalMemoryVtbl<CGlueObjContainer<T, C, R>> *vtbl;
    CGlueObjContainer<T, C, R> container;

    CGlueTraitObj() : container{} {}

    ~CGlueTraitObj() noexcept {
        mem_drop(std::move(container));
    }

    typedef C Context;

    inline int32_t phys_read_raw_iter(PhysicalReadMemOps data) noexcept {
        int32_t __ret = (this->vtbl)->phys_read_raw_iter(&this->container, data);
        return __ret;
    }

    inline int32_t phys_write_raw_iter(PhysicalWriteMemOps data) noexcept {
        int32_t __ret = (this->vtbl)->phys_write_raw_iter(&this->container, data);
        return __ret;
    }

    inline PhysicalMemoryMetadata metadata() const noexcept {
        PhysicalMemoryMetadata __ret = (this->vtbl)->metadata(&this->container);
        return __ret;
    }

    inline void set_mem_map(CSliceRef<PhysicalMemoryMapping> _mem_map) noexcept {
    (this->vtbl)->set_mem_map(&this->container, _mem_map);

    }

    inline MemoryViewBase<CBox<void>, Context> into_phys_view() && noexcept {
        auto ___ctx = StoreAll()[this->container.clone_context(), StoreAll()];
        MemoryViewBase<CBox<void>, Context> __ret = (this->vtbl)->into_phys_view(this->container);
        mem_forget(this->container);
        return __ret;
    }

    inline MemoryViewBase<CBox<void>, Context> phys_view() noexcept {
        MemoryViewBase<CBox<void>, Context> __ret = (this->vtbl)->phys_view(&this->container);
        return __ret;
    }

};

template<typename T, typename C, typename R>
struct CGlueTraitObj<T, CpuStateVtbl<CGlueObjContainer<T, C, R>>, C, R> {
    const CpuStateVtbl<CGlueObjContainer<T, C, R>> *vtbl;
    CGlueObjContainer<T, C, R> container;

    CGlueTraitObj() : container{} {}

    ~CGlueTraitObj() noexcept {
        mem_drop(std::move(container));
    }

    typedef C Context;

    inline void pause() noexcept {
    (this->vtbl)->pause(&this->container);

    }

    inline void resume() noexcept {
    (this->vtbl)->resume(&this->container);

    }

};

template<typename T, typename C, typename R>
struct CGlueTraitObj<T, ConnectorCpuStateVtbl<CGlueObjContainer<T, C, R>>, C, R> {
    const ConnectorCpuStateVtbl<CGlueObjContainer<T, C, R>> *vtbl;
    CGlueObjContainer<T, C, R> container;

    CGlueTraitObj() : container{} {}

    ~CGlueTraitObj() noexcept {
        mem_drop(std::move(container));
    }

    typedef C Context;

    inline int32_t cpu_state(CpuStateBase<CBox<void>, Context> * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->cpu_state(&this->container, ok_out);
        return __ret;
    }

    inline int32_t into_cpu_state(IntoCpuState<CBox<void>, Context> * ok_out) && noexcept {
        auto ___ctx = StoreAll()[this->container.clone_context(), StoreAll()];
        int32_t __ret = (this->vtbl)->into_cpu_state(this->container, ok_out);
        mem_forget(this->container);
        return __ret;
    }

};

template<typename T, typename C, typename R>
struct CGlueTraitObj<T, ProcessVtbl<CGlueObjContainer<T, C, R>>, C, R> {
    const ProcessVtbl<CGlueObjContainer<T, C, R>> *vtbl;
    CGlueObjContainer<T, C, R> container;

    CGlueTraitObj() : container{} {}

    ~CGlueTraitObj() noexcept {
        mem_drop(std::move(container));
    }

    typedef C Context;

    inline ProcessState state() noexcept {
        ProcessState __ret = (this->vtbl)->state(&this->container);
        return __ret;
    }

    inline int32_t set_dtb(Address dtb1, Address dtb2) noexcept {
        int32_t __ret = (this->vtbl)->set_dtb(&this->container, dtb1, dtb2);
        return __ret;
    }

    inline int32_t module_address_list_callback(const ArchitectureIdent * target_arch, ModuleAddressCallback callback) noexcept {
        int32_t __ret = (this->vtbl)->module_address_list_callback(&this->container, target_arch, callback);
        return __ret;
    }

    inline int32_t module_list_callback(const ArchitectureIdent * target_arch, ModuleInfoCallback callback) noexcept {
        int32_t __ret = (this->vtbl)->module_list_callback(&this->container, target_arch, callback);
        return __ret;
    }

    inline int32_t module_by_address(Address address, ArchitectureIdent architecture, ModuleInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->module_by_address(&this->container, address, architecture, ok_out);
        return __ret;
    }

    inline int32_t module_by_name_arch(CSliceRef<uint8_t> name, const ArchitectureIdent * architecture, ModuleInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->module_by_name_arch(&this->container, name, architecture, ok_out);
        return __ret;
    }

    inline int32_t module_by_name(CSliceRef<uint8_t> name, ModuleInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->module_by_name(&this->container, name, ok_out);
        return __ret;
    }

    inline int32_t primary_module_address(Address * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->primary_module_address(&this->container, ok_out);
        return __ret;
    }

    inline int32_t primary_module(ModuleInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->primary_module(&this->container, ok_out);
        return __ret;
    }

    inline int32_t module_import_list_callback(const ModuleInfo * info, ImportCallback callback) noexcept {
        int32_t __ret = (this->vtbl)->module_import_list_callback(&this->container, info, callback);
        return __ret;
    }

    inline int32_t module_export_list_callback(const ModuleInfo * info, ExportCallback callback) noexcept {
        int32_t __ret = (this->vtbl)->module_export_list_callback(&this->container, info, callback);
        return __ret;
    }

    inline int32_t module_section_list_callback(const ModuleInfo * info, SectionCallback callback) noexcept {
        int32_t __ret = (this->vtbl)->module_section_list_callback(&this->container, info, callback);
        return __ret;
    }

    inline int32_t module_import_by_name(const ModuleInfo * info, CSliceRef<uint8_t> name, ImportInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->module_import_by_name(&this->container, info, name, ok_out);
        return __ret;
    }

    inline int32_t module_export_by_name(const ModuleInfo * info, CSliceRef<uint8_t> name, ExportInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->module_export_by_name(&this->container, info, name, ok_out);
        return __ret;
    }

    inline int32_t module_section_by_name(const ModuleInfo * info, CSliceRef<uint8_t> name, SectionInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->module_section_by_name(&this->container, info, name, ok_out);
        return __ret;
    }

    inline const ProcessInfo * info() const noexcept {
        const ProcessInfo * __ret = (this->vtbl)->info(&this->container);
        return __ret;
    }

    inline void mapped_mem_range(imem gap_size, Address start, Address end, MemoryRangeCallback out) noexcept {
    (this->vtbl)->mapped_mem_range(&this->container, gap_size, start, end, out);

    }

    inline void mapped_mem(imem gap_size, MemoryRangeCallback out) noexcept {
    (this->vtbl)->mapped_mem(&this->container, gap_size, out);

    }

};

template<typename T, typename C, typename R>
struct CGlueTraitObj<T, VirtualTranslateVtbl<CGlueObjContainer<T, C, R>>, C, R> {
    const VirtualTranslateVtbl<CGlueObjContainer<T, C, R>> *vtbl;
    CGlueObjContainer<T, C, R> container;

    CGlueTraitObj() : container{} {}

    ~CGlueTraitObj() noexcept {
        mem_drop(std::move(container));
    }

    typedef C Context;

    inline void virt_to_phys_list(CSliceRef<VtopRange> addrs, VirtualTranslationCallback out, VirtualTranslationFailCallback out_fail) noexcept {
    (this->vtbl)->virt_to_phys_list(&this->container, addrs, out, out_fail);

    }

    inline void virt_to_phys_range(Address start, Address end, VirtualTranslationCallback out) noexcept {
    (this->vtbl)->virt_to_phys_range(&this->container, start, end, out);

    }

    inline void virt_translation_map_range(Address start, Address end, VirtualTranslationCallback out) noexcept {
    (this->vtbl)->virt_translation_map_range(&this->container, start, end, out);

    }

    inline void virt_page_map_range(imem gap_size, Address start, Address end, MemoryRangeCallback out) noexcept {
    (this->vtbl)->virt_page_map_range(&this->container, gap_size, start, end, out);

    }

    inline int32_t virt_to_phys(Address address, PhysicalAddress * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->virt_to_phys(&this->container, address, ok_out);
        return __ret;
    }

    inline int32_t virt_page_info(Address addr, Page * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->virt_page_info(&this->container, addr, ok_out);
        return __ret;
    }

    inline void virt_translation_map(VirtualTranslationCallback out) noexcept {
    (this->vtbl)->virt_translation_map(&this->container, out);

    }

    inline COption<Address> phys_to_virt(Address phys) noexcept {
        COption<Address> __ret = (this->vtbl)->phys_to_virt(&this->container, phys);
        return __ret;
    }

    inline void virt_page_map(imem gap_size, MemoryRangeCallback out) noexcept {
    (this->vtbl)->virt_page_map(&this->container, gap_size, out);

    }

};

template<typename T, typename C, typename R>
struct CGlueTraitObj<T, OsVtbl<CGlueObjContainer<T, C, R>>, C, R> {
    const OsVtbl<CGlueObjContainer<T, C, R>> *vtbl;
    CGlueObjContainer<T, C, R> container;

    CGlueTraitObj() : container{} {}

    ~CGlueTraitObj() noexcept {
        mem_drop(std::move(container));
    }

    typedef C Context;

    inline int32_t process_address_list_callback(AddressCallback callback) noexcept {
        int32_t __ret = (this->vtbl)->process_address_list_callback(&this->container, callback);
        return __ret;
    }

    inline int32_t process_info_list_callback(ProcessInfoCallback callback) noexcept {
        int32_t __ret = (this->vtbl)->process_info_list_callback(&this->container, callback);
        return __ret;
    }

    inline int32_t process_info_by_address(Address address, ProcessInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->process_info_by_address(&this->container, address, ok_out);
        return __ret;
    }

    inline int32_t process_info_by_name(CSliceRef<uint8_t> name, ProcessInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->process_info_by_name(&this->container, name, ok_out);
        return __ret;
    }

    inline int32_t process_info_by_pid(Pid pid, ProcessInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->process_info_by_pid(&this->container, pid, ok_out);
        return __ret;
    }

    inline int32_t process_by_info(ProcessInfo info, ProcessInstance<CBox<void>, Context> * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->process_by_info(&this->container, info, ok_out);
        return __ret;
    }

    inline int32_t into_process_by_info(ProcessInfo info, IntoProcessInstance<CBox<void>, Context> * ok_out) && noexcept {
        auto ___ctx = StoreAll()[this->container.clone_context(), StoreAll()];
        int32_t __ret = (this->vtbl)->into_process_by_info(this->container, info, ok_out);
        mem_forget(this->container);
        return __ret;
    }

    inline int32_t process_by_address(Address addr, ProcessInstance<CBox<void>, Context> * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->process_by_address(&this->container, addr, ok_out);
        return __ret;
    }

    inline int32_t process_by_name(CSliceRef<uint8_t> name, ProcessInstance<CBox<void>, Context> * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->process_by_name(&this->container, name, ok_out);
        return __ret;
    }

    inline int32_t process_by_pid(Pid pid, ProcessInstance<CBox<void>, Context> * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->process_by_pid(&this->container, pid, ok_out);
        return __ret;
    }

    inline int32_t into_process_by_address(Address addr, IntoProcessInstance<CBox<void>, Context> * ok_out) && noexcept {
        auto ___ctx = StoreAll()[this->container.clone_context(), StoreAll()];
        int32_t __ret = (this->vtbl)->into_process_by_address(this->container, addr, ok_out);
        mem_forget(this->container);
        return __ret;
    }

    inline int32_t into_process_by_name(CSliceRef<uint8_t> name, IntoProcessInstance<CBox<void>, Context> * ok_out) && noexcept {
        auto ___ctx = StoreAll()[this->container.clone_context(), StoreAll()];
        int32_t __ret = (this->vtbl)->into_process_by_name(this->container, name, ok_out);
        mem_forget(this->container);
        return __ret;
    }

    inline int32_t into_process_by_pid(Pid pid, IntoProcessInstance<CBox<void>, Context> * ok_out) && noexcept {
        auto ___ctx = StoreAll()[this->container.clone_context(), StoreAll()];
        int32_t __ret = (this->vtbl)->into_process_by_pid(this->container, pid, ok_out);
        mem_forget(this->container);
        return __ret;
    }

    inline int32_t module_address_list_callback(AddressCallback callback) noexcept {
        int32_t __ret = (this->vtbl)->module_address_list_callback(&this->container, callback);
        return __ret;
    }

    inline int32_t module_list_callback(ModuleInfoCallback callback) noexcept {
        int32_t __ret = (this->vtbl)->module_list_callback(&this->container, callback);
        return __ret;
    }

    inline int32_t module_by_address(Address address, ModuleInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->module_by_address(&this->container, address, ok_out);
        return __ret;
    }

    inline int32_t module_by_name(CSliceRef<uint8_t> name, ModuleInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->module_by_name(&this->container, name, ok_out);
        return __ret;
    }

    inline int32_t primary_module_address(Address * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->primary_module_address(&this->container, ok_out);
        return __ret;
    }

    inline int32_t primary_module(ModuleInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->primary_module(&this->container, ok_out);
        return __ret;
    }

    inline int32_t module_import_list_callback(const ModuleInfo * info, ImportCallback callback) noexcept {
        int32_t __ret = (this->vtbl)->module_import_list_callback(&this->container, info, callback);
        return __ret;
    }

    inline int32_t module_export_list_callback(const ModuleInfo * info, ExportCallback callback) noexcept {
        int32_t __ret = (this->vtbl)->module_export_list_callback(&this->container, info, callback);
        return __ret;
    }

    inline int32_t module_section_list_callback(const ModuleInfo * info, SectionCallback callback) noexcept {
        int32_t __ret = (this->vtbl)->module_section_list_callback(&this->container, info, callback);
        return __ret;
    }

    inline int32_t module_import_by_name(const ModuleInfo * info, CSliceRef<uint8_t> name, ImportInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->module_import_by_name(&this->container, info, name, ok_out);
        return __ret;
    }

    inline int32_t module_export_by_name(const ModuleInfo * info, CSliceRef<uint8_t> name, ExportInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->module_export_by_name(&this->container, info, name, ok_out);
        return __ret;
    }

    inline int32_t module_section_by_name(const ModuleInfo * info, CSliceRef<uint8_t> name, SectionInfo * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->module_section_by_name(&this->container, info, name, ok_out);
        return __ret;
    }

    inline const OsInfo * info() const noexcept {
        const OsInfo * __ret = (this->vtbl)->info(&this->container);
        return __ret;
    }

};

template<typename T, typename C, typename R>
struct CGlueTraitObj<T, KeyboardStateVtbl<CGlueObjContainer<T, C, R>>, C, R> {
    const KeyboardStateVtbl<CGlueObjContainer<T, C, R>> *vtbl;
    CGlueObjContainer<T, C, R> container;

    CGlueTraitObj() : container{} {}

    ~CGlueTraitObj() noexcept {
        mem_drop(std::move(container));
    }

    typedef C Context;

    inline bool is_down(int32_t vk) const noexcept {
        bool __ret = (this->vtbl)->is_down(&this->container, vk);
        return __ret;
    }

};

template<typename T, typename C, typename R>
struct CGlueTraitObj<T, KeyboardVtbl<CGlueObjContainer<T, C, R>>, C, R> {
    const KeyboardVtbl<CGlueObjContainer<T, C, R>> *vtbl;
    CGlueObjContainer<T, C, R> container;

    CGlueTraitObj() : container{} {}

    ~CGlueTraitObj() noexcept {
        mem_drop(std::move(container));
    }

    typedef C Context;

    inline bool is_down(int32_t vk) noexcept {
        bool __ret = (this->vtbl)->is_down(&this->container, vk);
        return __ret;
    }

    inline void set_down(int32_t vk, bool down) noexcept {
    (this->vtbl)->set_down(&this->container, vk, down);

    }

    inline int32_t state(KeyboardStateBase<CBox<void>, Context> * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->state(&this->container, ok_out);
        return __ret;
    }

};

template<typename T, typename C, typename R>
struct CGlueTraitObj<T, OsKeyboardVtbl<CGlueObjContainer<T, C, R>>, C, R> {
    const OsKeyboardVtbl<CGlueObjContainer<T, C, R>> *vtbl;
    CGlueObjContainer<T, C, R> container;

    CGlueTraitObj() : container{} {}

    ~CGlueTraitObj() noexcept {
        mem_drop(std::move(container));
    }

    typedef C Context;

    inline int32_t keyboard(KeyboardBase<CBox<void>, Context> * ok_out) noexcept {
        int32_t __ret = (this->vtbl)->keyboard(&this->container, ok_out);
        return __ret;
    }

    inline int32_t into_keyboard(IntoKeyboard<CBox<void>, Context> * ok_out) && noexcept {
        auto ___ctx = StoreAll()[this->container.clone_context(), StoreAll()];
        int32_t __ret = (this->vtbl)->into_keyboard(this->container, ok_out);
        mem_forget(this->container);
        return __ret;
    }

};

#endif // MEMFLOW_H
