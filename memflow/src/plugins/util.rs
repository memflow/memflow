use super::OptionVoid;
use log::error;
use std::ffi::c_void;

pub fn to_heap<T>(a: T) -> &'static mut T {
    Box::leak(Box::new(a))
}

pub unsafe fn extend_lifetime<T>(a: &T) -> &'static T {
    std::mem::transmute(a)
}

pub unsafe fn extend_lifetime_mut<T>(a: &mut T) -> &'static mut T {
    std::mem::transmute(a)
}

pub unsafe fn to_static_heap<T: Sized>(a: T) -> &'static mut c_void {
    std::mem::transmute(Box::leak(Box::new(a)))
}

pub fn to_void<T: Sized>(from: &'static mut T) -> &'static mut c_void {
    unsafe { std::mem::transmute(from) }
}

pub unsafe fn reinterpret_mut<T: Sized>(from: &mut c_void) -> &mut T {
    &mut *(from as *mut c_void as *mut T)
}

pub unsafe fn reinterpret<T: Sized>(from: &c_void) -> &T {
    &*(from as *const c_void as *const T)
}

pub extern "C" fn c_clone<T: Clone>(obj: &c_void) -> OptionVoid {
    let obj = unsafe { &*(obj as *const c_void as *const T) };
    let cloned_conn = Box::new(obj.clone());
    Some(unsafe { &mut *(Box::into_raw(cloned_conn) as *mut c_void) })
}

pub extern "C" fn c_drop<T>(obj: &mut c_void) {
    let _: Box<T> = unsafe { Box::from_raw(std::mem::transmute(obj)) };
    // drop box
}

pub trait ToIntResult<T> {
    fn int_result(self) -> i32;
    fn int_out_result(self, out: &mut T) -> i32;

    fn int_result_logged(self) -> i32
    where
        Self: Sized,
    {
        let res = self.int_result();
        if res != 0 {
            error!("err value: {}", res);
        }
        res
    }
}

impl<T, E: std::fmt::Display> ToIntResult<T> for Result<T, E> {
    fn int_result(self) -> i32 {
        if self.is_ok() {
            0
        } else {
            -1
        }
    }

    fn int_out_result(self, out: &mut T) -> i32 {
        if let Ok(ret) = self {
            *out = ret;
            0
        } else {
            -1
        }
    }

    fn int_result_logged(self) -> i32 {
        if let Err(e) = self {
            error!("{}", e);
            -1
        } else {
            0
        }
    }
}
