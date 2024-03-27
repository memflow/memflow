use memflow::architecture::{ArchitectureObj, Endianess};

pub mod x86;

#[no_mangle]
pub extern "C" fn mf_arch_bits(arch: &ArchitectureObj) -> u8 {
    arch.bits()
}

#[no_mangle]
pub extern "C" fn mf_arch_endianess(arch: &ArchitectureObj) -> Endianess {
    arch.endianess()
}

#[no_mangle]
pub extern "C" fn mf_arch_page_size(arch: &ArchitectureObj) -> usize {
    arch.page_size()
}

#[no_mangle]
pub extern "C" fn mf_arch_size_addr(arch: &ArchitectureObj) -> usize {
    arch.size_addr()
}

#[no_mangle]
pub extern "C" fn mf_arch_address_space_bits(arch: &ArchitectureObj) -> u8 {
    arch.address_space_bits()
}

/// Free an architecture reference
///
/// # Safety
///
/// `arch` must be a valid heap allocated reference created by one of the API's functions.
#[no_mangle]
pub unsafe extern "C" fn mf_arch_free(arch: &'static mut ArchitectureObj) {
    let _ = Box::from_raw(arch);
}
