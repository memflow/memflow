use flow_core::*;

#[cfg(feature = "connector-qemu-procfs")]
pub fn init_qemu_procfs() -> Result<flow_qemu_procfs::Memory> {
    flow_qemu_procfs::Memory::new()
}

#[cfg(not(feature = "connector-qemu-procfs"))]
pub fn init_qemu_procfs() -> Result<super::EmptyVirtualMemory> {
    Err(Error::new("connector qemu-procfs is not enabled"))
}
