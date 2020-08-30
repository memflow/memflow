use log::error;

pub fn inspect_err<E: std::fmt::Display>(e: E) -> E {
    error!("{}", e);
    e
}

pub fn to_heap<T>(a: T) -> &'static mut T {
    Box::leak(Box::new(a))
}

pub trait ToIntResult {
    fn int_result(self) -> i32;

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

impl<T, E: std::fmt::Display> ToIntResult for Result<T, E> {
    fn int_result(self) -> i32 {
        if self.is_ok() {
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
