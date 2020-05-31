use flow_core::mem::{PhysicalMemoryExt, VirtualAddressTranslator, VirtualMemory};

pub trait MemoryBackend: PhysicalMemoryExt + VirtualMemory + VirtualAddressTranslator {}

impl<T> MemoryBackend for T where
    T: PhysicalMemoryExt + VirtualMemory + VirtualAddressTranslator + ?Sized
{
}
