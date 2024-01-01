use std::mem::MaybeUninit;
use std::prelude::v1::*;

use super::{mem_data::*, phys_mem::*};
use crate::prelude::v1::{Result, *};

pub mod arch_overlay;
pub mod batcher;
pub mod cached_view;
pub mod remap_view;

#[cfg(feature = "std")]
pub mod cursor;

pub use arch_overlay::ArchOverlayView;
pub use batcher::MemoryViewBatcher;
pub use cached_view::CachedView;
pub use remap_view::RemapView;

#[cfg(feature = "std")]
pub use cursor::MemoryCursor;

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
#[cfg_attr(feature = "plugins", cglue_trait)]
#[cglue_forward]
#[int_result(PartialResult)]
pub trait MemoryView: Send {
    #[int_result]
    fn read_raw_iter(&mut self, data: ReadRawMemOps) -> Result<()>;

    #[int_result]
    fn write_raw_iter(&mut self, data: WriteRawMemOps) -> Result<()>;

    fn metadata(&self) -> MemoryViewMetadata;

    // Read helpers

    /// Read arbitrary amount of data.
    ///
    /// # Arguments
    ///
    /// * `inp` - input iterator of (address, buffer) pairs.
    /// * `out` - optional callback for any successful reads - along the way `inp` pairs may be
    /// split and only parts of the reads may succeed. This callback will return any successful
    /// chunks that have their buffers filled in.
    /// * `out_fail` - optional callback for any unsuccessful reads - this is the opposite of
    /// `out`, meaning any unsuccessful chunks with buffers in an unspecified state.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Address;
    /// use memflow::mem::MemoryView;
    /// use memflow::cglue::CTup2;
    ///
    /// fn read(mut mem: impl MemoryView, read_addrs: &[Address]) {
    ///
    ///     let mut bufs = vec![0u8; 8 * read_addrs.len()];
    ///
    ///     let data = read_addrs
    ///         .iter()
    ///         .zip(bufs.chunks_mut(8))
    ///         .map(|(&a, chunk)| CTup2(a, chunk.into()));
    ///
    ///     mem.read_iter(data, None, None).unwrap();
    ///
    ///     println!("{:?}", bufs);
    ///
    ///     # assert!(!bufs.chunks_exact(2).inspect(|c| println!("{:?}", c)).any(|c| c != &[255, 0]));
    /// }
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::types::size;
    /// # use memflow::os::Process;
    /// # let proc = DummyOs::quick_process(
    /// #     size::mb(2),
    /// #     &[255, 0].iter().cycle().copied().take(32).collect::<Vec<u8>>()
    /// # );
    /// # let virt_base = proc.info().address;
    /// # read(proc, &[virt_base, virt_base + 16usize]);
    /// ```
    #[int_result]
    #[vtbl_only]
    #[custom_impl(
        // Types within the C interface other than self and additional wrappers.
        {
            inp: CIterator<ReadData<'a>>,
            out: Option<&mut ReadCallback<'b, 'a>>,
            out_fail: Option<&mut ReadCallback<'b, 'a>>,
        },
        // Unwrapped return type
        Result<()>,
        // Conversion in trait impl to C arguments (signature names are expected).
        {},
        // This is the body of C impl minus the automatic wrapping.
        {
            MemOps::with_raw(
                inp.map(|CTup2(a, b)| CTup3(a, a, b)),
                out,
                out_fail,
                |data| this.read_raw_iter(data),
            )
        },
        // This part is processed in the trait impl after the call returns (impl_func_ret,
        // nothing extra needs to happen here).
        {},
    )]
    fn read_iter<'a, 'b>(
        &mut self,
        inp: impl Iterator<Item = ReadData<'a>>,
        out: Option<&mut ReadCallback<'b, 'a>>,
        out_fail: Option<&mut ReadCallback<'b, 'a>>,
    ) -> Result<()> {
        MemOps::with_raw(
            inp.map(|CTup2(a, b)| CTup3(a, a, b)),
            out,
            out_fail,
            |data| self.read_raw_iter(data),
        )
    }

    fn read_raw_list(&mut self, data: &mut [ReadData]) -> PartialResult<()> {
        let mut out = Ok(());

        let callback = &mut |CTup2(_, mut d): ReadData| {
            out = Err(PartialError::PartialVirtualRead(()));

            // Default behaviour is to zero out any failed data
            for v in d.iter_mut() {
                *v = 0;
            }

            true
        };

        let iter = data
            .iter_mut()
            .map(|CTup2(d1, d2)| CTup3(*d1, *d1, d2.into()));

        MemOps::with_raw(iter, None, Some(&mut callback.into()), |data| {
            self.read_raw_iter(data)
        })?;

        out
    }

    fn read_raw_into(&mut self, addr: Address, out: &mut [u8]) -> PartialResult<()> {
        self.read_raw_list(&mut [CTup2(addr, out.into())])
    }

    #[skip_func]
    fn read_raw(&mut self, addr: Address, len: usize) -> PartialResult<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.read_raw_into(addr, &mut buf).map_data(|_| buf)
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

    /// Reads the specified address as a rip-relative address.
    #[skip_func]
    fn read_addr64_rip(&mut self, addr: Address) -> PartialResult<Address>
    where
        Self: Sized,
    {
        let displacement = match self.read::<i32>(addr + 0x3) {
            Ok(d) => d,
            Err(e) => return Err(PartialError::Error(e.into())),
        };
        Ok(addr + 0x7 + displacement)
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
    fn read_ptr_into<U: PrimitiveAddress, T: Pod + ?Sized>(
        &mut self,
        ptr: Pointer<U, T>,
        out: &mut T,
    ) -> PartialResult<()>
    where
        Self: Sized,
    {
        self.read_into(ptr.into(), out)
    }

    #[skip_func]
    fn read_ptr<U: PrimitiveAddress, T: Pod + Sized>(
        &mut self,
        ptr: Pointer<U, T>,
    ) -> PartialResult<T>
    where
        Self: Sized,
    {
        self.read(ptr.into())
    }

    // Write helpers

    /// Write arbitrary amount of data.
    ///
    /// # Arguments
    ///
    /// * `inp` - input iterator of (address, buffer) pairs.
    /// * `out` - optional callback for any successful writes - along the way `inp` pairs may be
    /// split and only parts of the writes may succeed. This callback will return any successful
    /// chunks that have their buffers filled in.
    /// * `out_fail` - optional callback for any unsuccessful writes - this is the opposite of
    /// `out`, meaning any unsuccessful chunks with buffers in an unspecified state.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Address;
    /// use memflow::mem::MemoryView;
    /// use memflow::cglue::CTup2;
    /// use dataview::PodMethods;
    ///
    /// fn write(mut mem: impl MemoryView, writes: &[(Address, usize)]) {
    ///
    ///     let data = writes
    ///         .iter()
    ///         .map(|(a, chunk)| CTup2(*a, chunk.as_bytes().into()));
    ///
    ///     mem.write_iter(data, None, None).unwrap();
    ///
    ///     # assert_eq!(mem.read::<usize>(writes[0].0), Ok(3));
    ///     # assert_eq!(mem.read::<usize>(writes[1].0), Ok(4));
    /// }
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::types::size;
    /// # use memflow::os::Process;
    /// # let proc = DummyOs::quick_process(
    /// #     size::mb(2),
    /// #     &[255, 0].iter().cycle().copied().take(32).collect::<Vec<u8>>()
    /// # );
    /// # let virt_base = proc.info().address;
    /// # write(proc, &[(virt_base, 3), (virt_base + 16usize, 4)]);
    /// ```
    #[int_result]
    #[vtbl_only]
    #[custom_impl(
        // Types within the C interface other than self and additional wrappers.
        {
            inp: CIterator<WriteData<'a>>,
            out: Option<&mut WriteCallback<'b, 'a>>,
            out_fail: Option<&mut WriteCallback<'b, 'a>>,
        },
        // Unwrapped return type
        Result<()>,
        // Conversion in trait impl to C arguments (signature names are expected).
        {},
        // This is the body of C impl minus the automatic wrapping.
        {
            MemOps::with_raw(
                inp.map(|CTup2(a, b)| CTup3(a, a, b)),
                out,
                out_fail,
                |data| this.write_raw_iter(data),
            )
        },
        // This part is processed in the trait impl after the call returns (impl_func_ret,
        // nothing extra needs to happen here).
        {},
    )]
    fn write_iter<'a, 'b>(
        &mut self,
        inp: impl Iterator<Item = WriteData<'a>>,
        out: Option<&mut WriteCallback<'b, 'a>>,
        out_fail: Option<&mut WriteCallback<'b, 'a>>,
    ) -> Result<()> {
        MemOps::with_raw(
            inp.map(|CTup2(a, b)| CTup3(a, a, b)),
            out,
            out_fail,
            |data| self.write_raw_iter(data),
        )
    }

    fn write_raw_list(&mut self, data: &[WriteData]) -> PartialResult<()> {
        let mut out = Ok(());

        let callback = &mut |_| {
            out = Err(PartialError::PartialVirtualWrite(()));
            true
        };

        let iter = data.iter().copied();

        MemOps::with_raw(iter, None, Some(&mut callback.into()), |data| {
            self.write_iter(data.inp, data.out, data.out_fail)
        })?;

        out
    }

    fn write_raw(&mut self, addr: Address, data: &[u8]) -> PartialResult<()> {
        self.write_raw_list(&[CTup2(addr, data.into())])
    }

    #[skip_func]
    fn write<T: Pod + ?Sized>(&mut self, addr: Address, data: &T) -> PartialResult<()>
    where
        Self: Sized,
    {
        self.write_raw(addr, data.as_bytes())
    }

    #[skip_func]
    fn write_ptr<U: PrimitiveAddress, T: Pod + ?Sized>(
        &mut self,
        ptr: Pointer<U, T>,
        data: &T,
    ) -> PartialResult<()>
    where
        Self: Sized,
    {
        self.write(ptr.into(), data)
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
    /// For reading fixed-size char arrays the [`read_char_array`](Self::read_char_array) should be used.
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
    fn into_remap_view(self, mem_map: MemoryMap<(Address, umem)>) -> RemapView<Self>
    where
        Self: Sized,
    {
        RemapView::new(self, mem_map)
    }

    #[skip_func]
    fn remap_view(&mut self, mem_map: MemoryMap<(Address, umem)>) -> RemapView<Fwd<&mut Self>>
    where
        Self: Sized,
    {
        self.forward_mut().into_remap_view(mem_map)
    }

    // deprecated = Expose this via cglue
    #[skip_func]
    fn into_phys_mem(self) -> PhysicalMemoryOnView<Self>
    where
        Self: Sized,
    {
        PhysicalMemoryOnView { mem: self }
    }

    // deprecated = Expose this via cglue
    #[skip_func]
    fn phys_mem(&mut self) -> PhysicalMemoryOnView<Fwd<&mut Self>>
    where
        Self: Sized,
    {
        self.forward_mut().into_phys_mem()
    }
}

/// Creates a PhysicalMemory object from a MemoryView without doing any translations.
/// This function simply redirects all calls to PhysicalMemory to the underlying MemoryView
#[repr(C)]
#[derive(Clone)]
#[cfg_attr(feature = "abi_stable", derive(::abi_stable::StableAbi))]
pub struct PhysicalMemoryOnView<T> {
    mem: T,
}

impl<T: MemoryView> PhysicalMemory for PhysicalMemoryOnView<T>
where
    T: MemoryView,
{
    #[inline]
    fn phys_read_raw_iter(
        &mut self,
        MemOps { inp, out, out_fail }: PhysicalReadMemOps,
    ) -> Result<()> {
        let inp = inp.map(|CTup3(addr, meta_addr, data)| CTup3(addr.into(), meta_addr, data));
        MemOps::with_raw(inp, out, out_fail, |data| self.mem.read_raw_iter(data))
    }

    #[inline]
    fn phys_write_raw_iter(
        &mut self,
        MemOps { inp, out, out_fail }: PhysicalWriteMemOps,
    ) -> Result<()> {
        let inp = inp.map(|CTup3(addr, meta_addr, data)| CTup3(addr.into(), meta_addr, data));
        MemOps::with_raw(inp, out, out_fail, |data| self.mem.write_raw_iter(data))
    }

    #[inline]
    fn metadata(&self) -> PhysicalMemoryMetadata {
        let md = self.mem.metadata();

        PhysicalMemoryMetadata {
            max_address: md.max_address,
            real_size: md.real_size,
            readonly: md.readonly,
            ideal_batch_size: 4096,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "abi_stable", derive(::abi_stable::StableAbi))]
pub struct MemoryViewMetadata {
    pub max_address: Address,
    pub real_size: umem,
    pub readonly: bool,
    pub little_endian: bool,
    pub arch_bits: u8,
}
