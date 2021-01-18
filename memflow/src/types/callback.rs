//! FFI compatible callbacks
//!
//! The essence of them is to be simple, reliable, and flexible. Thus, every callback accepts a C
//! function that takes 2 arguments: `context`, and `argument`.
//!
//! `context` is any type of context. We take a sized pointer to it. It can hold anything like a
//! closure that we then wrap for callback functionality.
//!
//! `argument` is the actual argument that gets produced and passed over. This will be variable,
//! and it is passed as value.
//!
//! `OpaqueCallback`, as the name suggests, marks the `context` as opaque, casts it to `c_void`
//! pointer. It allows the code not to care about what's behind the context, it just knows that it
//! needs to pass it over to the callback.

use core::ffi::c_void;
use std::prelude::v1::*;

// C style callbacks that are needed so that C code can easily use callback like functions
#[repr(transparent)]
pub struct OpaqueCallback<'a, T>(Callback<'a, c_void, T>);

impl<'a, T> OpaqueCallback<'a, T> {
    #[must_use = "this value is the stopping condition"]
    pub fn call(&mut self, arg: T) -> bool {
        (self.0.func)(self.0.context, arg)
    }
}

#[repr(C)]
pub struct Callback<'a, T, F> {
    context: &'a mut T,
    func: extern "C" fn(&mut T, F) -> bool,
}

impl<'a, T, F> From<Callback<'a, T, F>> for OpaqueCallback<'a, F> {
    fn from(callback: Callback<'a, T, F>) -> Self {
        Self(callback.into_opaque())
    }
}

impl<'a, T, F> Callback<'a, T, F> {
    pub fn into_opaque(self) -> Callback<'a, c_void, F> {
        unsafe {
            Callback {
                context: &mut *(self.context as *mut T as *mut std::ffi::c_void),
                func: std::mem::transmute(self.func),
            }
        }
    }

    pub fn new(context: &'a mut T, func: extern "C" fn(&mut T, F) -> bool) -> Self {
        Self { context, func }
    }
}

impl<'a, T: FnMut(F) -> bool, F> From<&'a mut T> for OpaqueCallback<'a, F> {
    fn from(func: &'a mut T) -> Self {
        extern "C" fn callback<T: FnMut(F) -> bool, F>(func: &mut T, context: F) -> bool {
            func(context)
        }

        Callback {
            context: func,
            func: callback::<T, F>,
        }
        .into()
    }
}

impl<'a, T> From<&'a mut Vec<T>> for OpaqueCallback<'a, T> {
    fn from(vec: &'a mut Vec<T>) -> Self {
        extern "C" fn callback<T>(v: &mut Vec<T>, context: T) -> bool {
            v.push(context);
            true
        }

        Callback {
            context: vec,
            func: callback::<T>,
        }
        .into()
    }
}

impl<'a, T> std::iter::Extend<T> for OpaqueCallback<'a, T> {
    fn extend<F: IntoIterator<Item = T>>(&mut self, iter: F) {
        for item in iter {
            if self.call(item) {
                break;
            }
        }
    }
}
