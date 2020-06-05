use flow_core::mem::{PhysicalMemory, VirtualMemory, VAT};

pub trait MemoryBackend: PhysicalMemoryExt + VirtualMemory + VirtualAddressTranslator {}

impl<T> MemoryBackend for T where
    T: PhysicalMemoryExt + VirtualMemory + VirtualAddressTranslator + ?Sized
{
}
