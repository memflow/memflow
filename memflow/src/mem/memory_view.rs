use crate::prelude::v1::{Result, *};
use std::mem::MaybeUninit;
use std::prelude::v1::*;

use super::mem_data::*;

pub mod arch_overlay;
pub mod batcher;
pub mod remap_view;

pub use arch_overlay::ArchOverlayView;
pub use batcher::MemoryViewBatcher;
pub use remap_view::RemapView;

/// The `MemoryView` trait implements generic access to memory, no matter if it is a process
/// virtual memory, or machine's physical memory.
///
/// The CPU accesses virtual memory by setting the CR3 register to the appropiate Directory Table Base (DTB)
/// for that process. The ntoskrnl.exe Kernel Process has it's own DTB.
/// Using the DTB it is possible to resolve the physical memory location of a virtual address page.
/// After the address has been resolved the physical memory page can then be read or written to.
///
/// There are 3 methods which are required to be implemented by the provider of this trait.
///
/// # Examples
///
/// Reading from a `MemoryView`:
/// ```
/// use memflow::types::Address;
/// use memflow::mem::MemoryView;
///
/// fn read(mem: &mut impl MemoryView, read_addr: Address) {
///     let mut addr = 0u64;
///     mem.read_into(read_addr, &mut addr).unwrap();
///     println!("addr: {:x}", addr);
///     # assert_eq!(addr, 0x00ff_00ff_00ff_00ff);
/// }
/// # use memflow::dummy::{DummyMemory, DummyOs};
/// # use memflow::os::Process;
/// # use memflow::types::size;
/// # let mut proc = DummyOs::quick_process(size::mb(2), &[255, 0, 255, 0, 255, 0, 255, 0]);
/// # let virt_base = proc.info().address;
/// # read(&mut proc, virt_base);
/// ```
#[cglue_trait]
#[cglue_forward]
#[int_result(PartialResult)]
pub trait MemoryView: Send {
    #[int_result]
    fn read_raw_iter<'a>(
        &mut self,
        data: CIterator<ReadData<'a>>,
        out_fail: &mut ReadFailCallback<'_, 'a>,
    ) -> Result<()>;

    #[int_result]
    fn write_raw_iter<'a>(
        &mut self,
        data: CIterator<WriteData<'a>>,
        out_fail: &mut WriteFailCallback<'_, 'a>,
    ) -> Result<()>;

    fn metadata(&self) -> MemoryViewMetadata;

    // Read helpers

    fn read_raw_list(&mut self, data: &mut [ReadData]) -> PartialResult<()> {
        let mut out = Ok(());

        let callback = &mut |MemData(_, mut d): ReadData| {
            out = Err(PartialError::PartialVirtualRead(()));

            // Default behaviour is to zero out any failed data
            for v in d.iter_mut() {
                *v = 0;
            }

            true
        };

        let mut iter = data.iter().map(|MemData(d1, d2)| MemData(*d1, d2.into()));

        self.read_raw_iter((&mut iter).into(), &mut callback.into())?;

        out
    }

    fn read_raw_into(&mut self, addr: Address, out: &mut [u8]) -> PartialResult<()> {
        self.read_raw_list(&mut [MemData(addr, out.into())])
    }

    #[skip_func]
    fn read_raw(&mut self, addr: Address, len: usize) -> PartialResult<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.read_raw_into(addr, &mut *buf).map_data(|_| buf)
    }

    #[skip_func]
    fn read_into<T: Pod + ?Sized>(&mut self, addr: Address, out: &mut T) -> PartialResult<()>
    where
        Self: Sized,
    {
        self.read_raw_into(addr, out.as_bytes_mut())
    }

    #[skip_func]
    #[allow(clippy::uninit_assumed_init)]
    fn read<T: Pod + Sized>(&mut self, addr: Address) -> PartialResult<T>
    where
        Self: Sized,
    {
        let mut obj: T = unsafe { MaybeUninit::uninit().assume_init() };
        // TODO: zero out on partial
        self.read_into(addr, &mut obj).map_data(|_| obj)
    }

    // TODO: allow cglue to somehow pass MaybeUninit to the IntError
    #[skip_func]
    fn read_addr32(&mut self, addr: Address) -> PartialResult<Address>
    where
        Self: Sized,
    {
        self.read::<u32>(addr).map_data(|d| d.into())
    }

    #[skip_func]
    fn read_addr64(&mut self, addr: Address) -> PartialResult<Address>
    where
        Self: Sized,
    {
        self.read::<u64>(addr).map_data(|d| d.into())
    }

    #[skip_func]
    fn read_addr_arch(&mut self, arch: ArchitectureObj, addr: Address) -> PartialResult<Address>
    where
        Self: Sized,
    {
        match arch.bits() {
            64 => self.read_addr64(addr),
            32 => self.read_addr32(addr),
            _ => Err(PartialError::Error(Error(
                ErrorOrigin::VirtualMemory,
                ErrorKind::InvalidArchitecture,
            ))),
        }
    }

    #[skip_func]
    fn read_ptr_into<U: Pod + ?Sized>(&mut self, ptr: Pointer<U>, out: &mut U) -> PartialResult<()>
    where
        Self: Sized,
    {
        let MemoryViewMetadata {
            arch_bits,
            little_endian,
            ..
        } = self.metadata();

        self.read_into(ptr.address(arch_bits, little_endian), out)
    }

    #[skip_func]
    fn read_ptr<U: Pod + Sized>(&mut self, ptr: Pointer<U>) -> PartialResult<U>
    where
        Self: Sized,
    {
        let MemoryViewMetadata {
            arch_bits,
            little_endian,
            ..
        } = self.metadata();

        self.read(ptr.address(arch_bits, little_endian))
    }

    // Write helpers

    fn write_raw_list(&mut self, data: &[WriteData]) -> PartialResult<()> {
        let mut out = Ok(());

        let callback = &mut |_| {
            out = Err(PartialError::PartialVirtualWrite(()));
            true
        };

        let mut iter = data.iter().copied();

        self.write_raw_iter((&mut iter).into(), &mut callback.into())?;

        out
    }

    fn write_raw(&mut self, addr: Address, data: &[u8]) -> PartialResult<()> {
        self.write_raw_list(&[MemData(addr, data.into())])
    }

    #[skip_func]
    fn write<T: Pod + ?Sized>(&mut self, addr: Address, data: &T) -> PartialResult<()>
    where
        Self: Sized,
    {
        self.write_raw(addr, data.as_bytes())
    }

    #[skip_func]
    fn write_ptr<U: Pod + Sized>(&mut self, ptr: Pointer<U>, data: &U) -> PartialResult<()>
    where
        Self: Sized,
    {
        let MemoryViewMetadata {
            arch_bits,
            little_endian,
            ..
        } = self.metadata();

        self.write(ptr.address(arch_bits, little_endian), data)
    }

    /// Reads a fixed length string from the target.
    ///
    /// # Remarks:
    ///
    /// The string does not have to be null-terminated.
    /// If a null terminator is found the string is truncated to the terminator.
    /// If no null terminator is found the resulting string is exactly `len` characters long.
    #[skip_func]
    fn read_char_array(&mut self, addr: Address, len: usize) -> PartialResult<String> {
        let mut buf = vec![0; len];
        self.read_raw_into(addr, &mut buf).data_part()?;
        if let Some((n, _)) = buf.iter().enumerate().find(|(_, c)| **c == 0_u8) {
            buf.truncate(n);
        }
        Ok(String::from_utf8_lossy(&buf).to_string())
    }

    /// Reads a variable length string with a length of up to specified amount from the target.
    ///
    /// # Arguments
    ///
    /// * `addr` - target address to read from
    /// * `n` - maximum number of bytes to read
    ///
    /// # Remarks:
    ///
    /// The string must be null-terminated.
    /// If no null terminator is found the this function will return an error.
    ///
    /// For reading fixed-size char arrays the [`virt_read_char_array`] should be used.
    #[skip_func]
    fn read_char_string_n(&mut self, addr: Address, n: usize) -> PartialResult<String> {
        let mut buf = vec![0; std::cmp::min(32, n)];

        let mut last_n = 0;

        loop {
            let (_, right) = buf.split_at_mut(last_n);

            self.read_raw_into(addr + last_n, right).data_part()?;
            if let Some((n, _)) = right.iter().enumerate().find(|(_, c)| **c == 0_u8) {
                buf.truncate(last_n + n);
                return Ok(String::from_utf8_lossy(&buf).to_string());
            }
            if buf.len() >= n {
                break;
            }
            last_n = buf.len();

            buf.extend((0..buf.len()).map(|_| 0));
        }

        Err(PartialError::Error(Error(
            ErrorOrigin::VirtualMemory,
            ErrorKind::OutOfBounds,
        )))
    }

    /// Reads a variable length string with up to 4kb length from the target.
    ///
    /// # Arguments
    ///
    /// * `addr` - target address to read from
    #[skip_func]
    fn read_char_string(&mut self, addr: Address) -> PartialResult<String> {
        self.read_char_string_n(addr, 4096)
    }

    // TODO: batcher

    #[cfg(feature = "std")]
    #[skip_func]
    fn cursor(&mut self) -> MemoryCursor<Fwd<&mut Self>>
    where
        Self: Sized,
    {
        MemoryCursor::new(self.forward())
    }

    #[cfg(feature = "std")]
    #[skip_func]
    fn into_cursor(self) -> MemoryCursor<Self>
    where
        Self: Sized,
    {
        MemoryCursor::new(self)
    }

    #[cfg(feature = "std")]
    #[skip_func]
    fn cursor_at(&mut self, address: Address) -> MemoryCursor<Fwd<&mut Self>>
    where
        Self: Sized,
    {
        MemoryCursor::at(self.forward(), address)
    }

    #[cfg(feature = "std")]
    #[skip_func]
    fn into_cursor_at(self, address: Address) -> MemoryCursor<Self>
    where
        Self: Sized,
    {
        MemoryCursor::at(self, address)
    }

    #[skip_func]
    fn batcher(&mut self) -> MemoryViewBatcher<Self>
    where
        Self: Sized,
    {
        MemoryViewBatcher::new(self)
    }

    #[skip_func]
    fn into_overlay_arch(self, arch: ArchitectureObj) -> ArchOverlayView<Self>
    where
        Self: Sized,
    {
        ArchOverlayView::new(self, arch)
    }

    #[skip_func]
    fn overlay_arch(&mut self, arch: ArchitectureObj) -> ArchOverlayView<Fwd<&mut Self>>
    where
        Self: Sized,
    {
        ArchOverlayView::new(self.forward_mut(), arch)
    }

    #[skip_func]
    fn into_overlay_arch_parts(self, arch_bits: u8, little_endian: bool) -> ArchOverlayView<Self>
    where
        Self: Sized,
    {
        ArchOverlayView::new_parts(self, arch_bits, little_endian)
    }

    #[skip_func]
    fn overlay_arch_parts(
        &mut self,
        arch_bits: u8,
        little_endian: bool,
    ) -> ArchOverlayView<Fwd<&mut Self>>
    where
        Self: Sized,
    {
        ArchOverlayView::new_parts(self.forward_mut(), arch_bits, little_endian)
    }

    #[skip_func]
    fn into_remap_view(self, mem_map: MemoryMap<(Address, usize)>) -> RemapView<Self> {
        RemapView::new(self, mem_map)
    }

    #[skip_view]
    fn remap_view(&mut self, mem_map: MemoryMap<(Address, usize)>) -> RemapView<Fwd<&mut Self>> {
        self.forward_mut().into_remap_view(mem_map)
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[repr(C)]
pub struct MemoryViewMetadata {
    pub max_address: Address,
    pub real_size: u64,
    pub readonly: bool,
    pub little_endian: bool,
    pub arch_bits: u8,
}

pub type ReadFailCallback<'a, 'b> = OpaqueCallback<'a, ReadData<'b>>;

pub type WriteFailCallback<'a, 'b> = OpaqueCallback<'a, WriteData<'b>>;
