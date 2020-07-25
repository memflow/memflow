#[macro_export]
#[cfg(feature = "trace_mmu")]
macro_rules! vtop_trace {
    ( $( $x:expr ),* ) => {
        log::trace!( $($x, )* );
    }
}

#[macro_export]
#[cfg(not(feature = "trace_mmu"))]
macro_rules! vtop_trace {
    ( $( $x:expr ),* ) => {};
}
