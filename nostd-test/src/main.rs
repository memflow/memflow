#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

pub fn _start() -> ! {
    loop {}
}
