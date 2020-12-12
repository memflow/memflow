#ifndef BINDDESTR_H
#define BINDDESTR_H

#include <functional>

// Binds a particular destructor function to the type, automatically destroying it
template<typename T, void (*D)(T *)>
struct BindDestr
{
    T *inner;

    BindDestr(BindDestr &other) = delete;

    BindDestr(BindDestr &&other) {
        this->inner = other.inner;
        other.inner = NULL;
    }

    BindDestr(T *inner2)
        : inner(inner2) {}

    ~BindDestr() {
        if (this->inner) {
            D(this->inner);
        }
    }

    inline operator const T *() const {
        return this->inner;
    }

    inline T *invalidate() {
        T *ret = this->inner;
        this->inner = NULL;
        return ret;
    }
};

// Wrap a C function with a particular class prefix (removes it in the class function)
// and specified return type
#define WRAP_FN_TYPE(TYPE, CLASS, FNAME) \
    template<typename... Args> \
    inline TYPE FNAME (Args... args) { \
        return :: CLASS##_##FNAME (this->inner, args...); \
    }

// Wrap a C function with a particular class prefix (removes it in the class function)
#define WRAP_FN(CLASS, FNAME) WRAP_FN_TYPE(std::function<decltype( :: CLASS##_##FNAME )>::result_type, CLASS, FNAME)

// Same, but invalidates the pointer
#define WRAP_FN_TYPE_INVALIDATE(TYPE, CLASS, FNAME) \
    template<typename... Args> \
    inline TYPE FNAME (Args... args) { \
        return :: CLASS##_##FNAME (this->invalidate(), args...); \
    }

#define WRAP_FN_INVALIDATE(CLASS, FNAME) WRAP_FN_TYPE_INVALIDATE(std::function<decltype( :: CLASS##_##FNAME )>::result_type, CLASS, FNAME)

// Wrap a C function in a raw way with specified return type
#define WRAP_FN_RAW_TYPE(TYPE, FNAME) \
    template<typename... Args> \
    inline TYPE FNAME (Args... args) { \
        return :: FNAME (this->inner, args...); \
    }

// Wrap a C function in a raw way
#define WRAP_FN_RAW(FNAME) WRAP_FN_RAW_TYPE(std::function<decltype( :: FNAME )>::result_type, FNAME)

#endif
