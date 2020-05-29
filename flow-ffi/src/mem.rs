use flow_core::mem::{AccessPhysicalMemoryExt, AccessVirtualMemory, VirtualAddressTranslator};

pub trait MemoryBackend:
    AccessPhysicalMemoryExt + AccessVirtualMemory + VirtualAddressTranslator
{
}

impl<T> MemoryBackend for T where
    T: AccessPhysicalMemoryExt + AccessVirtualMemory + VirtualAddressTranslator + ?Sized
{
}
