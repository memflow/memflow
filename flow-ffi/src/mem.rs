use flow_core::mem::{AccessPhysicalMemory, AccessVirtualMemory, VirtualAddressTranslator};

pub trait MemoryBackend:
    AccessPhysicalMemory + AccessVirtualMemory + VirtualAddressTranslator
{
}

impl<T> MemoryBackend for T where
    T: AccessPhysicalMemory + AccessVirtualMemory + VirtualAddressTranslator + ?Sized
{
}
