use log::error;

pub fn inspect_err<E: std::fmt::Display>(e: E) -> E {
    error!("{}", e);
    e
}

pub fn to_heap<T>(a: T) -> &'static mut T {
    Box::leak(Box::new(a))
}
