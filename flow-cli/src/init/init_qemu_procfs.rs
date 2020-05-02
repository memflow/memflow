use flow_core::*;

#[cfg(all(target_os = "linux", feature = "connector-qemu-procfs"))]
pub fn init_qemu_procfs() -> Result<flow_qemu_procfs::Memory<mem::NoCache>> {
    flow_qemu_procfs::Memory::new(mem::NO_CACHE)
}

#[cfg(all(feature = "connector-qemu-procfs", not(target_os = "linux")))]
pub fn init_qemu_procfs() -> Result<super::EmptyVirtualMemory> {
    Err(Error::new(
        "connector qemu_procfs is not available on this system",
    ))
}

#[cfg(not(feature = "connector-qemu-procfs"))]
pub fn init_qemu_procfs() -> Result<super::EmptyVirtualMemory> {
    Err(Error::new("connector qemu-procfs is not enabled"))
}
