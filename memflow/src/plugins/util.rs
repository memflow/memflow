use super::OptionVoid;
use log::error;
use std::ffi::c_void;

pub unsafe fn to_static_heap<T: Sized>(a: T) -> &'static mut c_void {
    &mut *(Box::leak(Box::new(a)) as *mut T as *mut std::ffi::c_void)
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
