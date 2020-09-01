use memflow::architecture::{x86, ArchitectureObj};

#[no_mangle]
pub static X86_32: &ArchitectureObj = &x86::x32::ARCH;

#[no_mangle]
pub static X86_32_PAE: &ArchitectureObj = &x86::x32_pae::ARCH;

#[no_mangle]
pub static X86_64: &ArchitectureObj = &x86::x64::ARCH;

#[no_mangle]
pub extern "C" fn is_x86_arch(arch: &ArchitectureObj) -> bool {
    x86::is_x86_arch(*arch)
}

// TODO: new_translator, if it is feasible
