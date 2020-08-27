#![no_std]
#![no_main]
#![feature(abi_efiapi)]
use core::*;
use uefi::prelude::*;

#[macro_use]
extern crate alloc;

extern crate rlibc;

use crate::alloc::vec::Vec;

use log::*;

use uefi::{
    data_types::{CStr16, Char16},
    proto::Protocol,
    unsafe_guid, Handle, Status,
};

#[entry]
fn efi_main(handle: Handle, st: SystemTable<Boot>) -> Status {
    uefi_services::init(&st).expect_success("Failed to initialize utilities");

    info!("memflow EFI test");

    let bt = st.boot_services();

    Status::SUCCESS
}
