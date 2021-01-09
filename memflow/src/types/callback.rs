//! FFI compatible callbacks
//!
//! The essence of them is to be simple, reliable, and flexible. Thus, every callback accepts a C
//! function that takes 3 arguments: `self`, `context`, and `argument`.
//!
//! `self` argument is needed to pass back control of mutable self to the callback handler. This
//! allows safe nesting of functions that use callbacks.
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
pub struct OpaqueCallback<'a, S: ?Sized, T>(Callback<'a, S, c_void, T>);

impl<'a, S: ?Sized, T> OpaqueCallback<'a, S, T> {
    #[must_use = "this value is the stopping condition"]
    pub fn call(&mut self, s: &mut S, arg: T) -> bool {
        (self.0.func)(s, self.0.context, arg)
    }

    pub fn extendable(&'a mut self, s: &'a mut S) -> ExtendCallback<'a, S, T> {
        ExtendCallback(s, self)
    }
}

#[repr(C)]
pub struct Callback<'a, S: ?Sized, T, F> {
    context: &'a mut T,
    func: extern "C" fn(&mut S, &mut T, F) -> bool,
}

impl<'a, S: ?Sized, T, F> From<Callback<'a, S, T, F>> for OpaqueCallback<'a, S, F> {
    fn from(callback: Callback<'a, S, T, F>) -> Self {
        Self(callback.into_opaque())
    }
}

impl<'a, S: ?Sized, T, F> Callback<'a, S, T, F> {
    pub fn into_opaque(self) -> Callback<'a, S, c_void, F> {
        unsafe {
            Callback {
                context: &mut *(self.context as *mut T as *mut std::ffi::c_void),
                func: std::mem::transmute(self.func),
            }
        }
    }

    pub fn new(context: &'a mut T, func: extern "C" fn(&mut S, &mut T, F) -> bool) -> Self {
        Self { context, func }
    }
}

impl<'a, T: FnMut(&mut S, F) -> bool, S: ?Sized, F> From<&'a mut T> for OpaqueCallback<'a, S, F> {
    fn from(func: &'a mut T) -> Self {
        extern "C" fn callback<S: ?Sized, T: FnMut(&mut S, F) -> bool, F>(
            s: &mut S,
            func: &mut T,
            context: F,
        ) -> bool {
            func(s, context)
        }

        Callback {
            context: func,
            func: callback::<S, T, F>,
        }
        .into()
    }
}

impl<'a, S: ?Sized, T> From<&'a mut Vec<T>> for OpaqueCallback<'a, S, T> {
    fn from(vec: &'a mut Vec<T>) -> Self {
        extern "C" fn callback<S: ?Sized, T>(_: &mut S, v: &mut Vec<T>, context: T) -> bool {
            v.push(context);
            true
        }

        Callback {
            context: vec,
            func: callback::<S, T>,
        }
        .into()
    }
}

pub struct ExtendCallback<'a, S: ?Sized, T>(&'a mut S, &'a mut OpaqueCallback<'a, S, T>);

impl<'a, S, T> std::iter::Extend<T> for ExtendCallback<'a, S, T> {
    fn extend<F: IntoIterator<Item = T>>(&mut self, iter: F) {
        for item in iter {
            if self.1.call(self.0, item) {
                break;
            }
        }
    }
}
