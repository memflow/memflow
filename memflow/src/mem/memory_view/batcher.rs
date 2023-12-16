use std::prelude::v1::*;

use super::*;
use crate::dataview::PodMethods;
use crate::error::PartialResult;
use crate::types::Address;

/// A structure for batching memory reads and writes.
///
/// # Examples
///
/// ```
/// use memflow::prelude::v1::*;
/// use memflow::dummy::DummyMemory;
/// # use memflow::dummy::DummyOs;
/// # use memflow::architecture::x86::x64;
///
/// # let phys_mem = DummyMemory::new(size::mb(16));
/// # let mut os = DummyOs::new(phys_mem);
/// # let (dtb, _) = os.alloc_dtb(size::mb(8), &[]);
/// # let phys_mem = os.into_inner();
/// # let translator = x64::new_translator(dtb);
/// let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
/// let mut batcher = MemoryViewBatcher::new(&mut virt_mem);
/// ```
pub struct MemoryViewBatcher<'a, T: MemoryView> {
    vmem: &'a mut T,
    read_list: Vec<ReadData<'a>>,
    write_list: Vec<WriteData<'a>>,
}

impl<'a, T: MemoryView> MemoryViewBatcher<'a, T> {
    /// Creates a new `MemoryViewBatcher` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::DummyMemory;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, _) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    /// let mut batcher = MemoryViewBatcher::new(&mut virt_mem);
    /// ```
    pub fn new(vmem: &'a mut T) -> Self {
        Self {
            vmem,
            read_list: vec![],
            write_list: vec![],
        }
    }

    /// Reserves capacity for the read list.
    /// Reserves capacity for at least `additional` more elements to be handled
    /// in the given `MemoryViewBatcher<'a, T>`. The internal collection may reserve
    /// more space to speculatively avoid frequent reallocations.
    ///
    /// # Arguments
    ///
    /// * `capacity`: The number of operations to reserve space for.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::DummyMemory;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, _) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    /// let mut batcher = MemoryViewBatcher::new(&mut virt_mem);
    ///
    /// // Reserve space 10 operations
    /// batcher.reserve(10);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    pub fn reserve(&mut self, capacity: usize) -> &mut Self {
        self.read_list.reserve(capacity);
        self
    }

    /// Executes all pending operations in this batch.
    ///
    /// This also consumes and discards this batcher so it cannot be used anymore.
    /// The same behavior can be achieved by implicitly calling `drop` on the batcher
    /// (for example, when going out of scope).
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::DummyMemory;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, _) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    /// let mut batcher = MemoryViewBatcher::new(&mut virt_mem);
    ///
    /// // commit the batch to memory, this is optional and just used to check if the operations succeed
    /// batcher.commit_rw().unwrap();
    /// ```
    pub fn commit_rw(&mut self) -> PartialResult<()> {
        if !self.read_list.is_empty() {
            self.vmem.read_raw_list(&mut self.read_list)?;
            self.read_list.clear();
        }

        if !self.write_list.is_empty() {
            self.vmem.write_raw_list(&self.write_list)?;
            self.write_list.clear();
        }

        Ok(())
    }

    /// Appends an iterator over read operations `ReadIter` to this batch.
    ///
    /// # Arguments
    ///
    /// * `iter`: An iterator over `ReadData` instances.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::DummyMemory;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, virt_base) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    ///
    /// let addr = virt_base; // some arbitrary address
    /// let mut buf = [0u8; 8];
    ///
    /// // create the batcher
    /// let mut batcher = MemoryViewBatcher::new(&mut virt_mem);
    ///
    /// // append the read command
    /// batcher.read_raw_iter(std::iter::once(CTup2(addr, buf.as_mut().into())).into_iter());
    ///
    /// // commit the batch to memory, this is optional and just used to check if the operations succeed
    /// assert!(batcher.commit_rw().is_ok());
    /// ```
    pub fn read_raw_iter(&mut self, iter: impl ReadIterator<'a>) -> &mut Self {
        self.read_list.extend(iter);
        self
    }

    /// Reads data from memory and stores it in the provided buffer.
    ///
    /// # Arguments
    ///
    /// * `addr`: The starting address to read from.
    /// * `out`: A mutable reference to the buffer where the data will be stored.
    ///
    /// # Example
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::DummyMemory;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, virt_base) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    ///
    /// let addr = virt_base; // some arbitrary address
    /// let write_data = [0x10, 0x20, 0x30, 0x40];
    /// let mut read_data = [0u8; 4];
    ///
    /// {
    ///     // create batcher in a new scope
    ///     let mut batcher = MemoryViewBatcher::new(&mut virt_mem);
    ///
    ///     // write the `write_data` array to memory
    ///     batcher.write_raw_into(addr, &write_data);
    ///
    ///     // commit the batch to memory, this is optional and just used to check if the operations succeed
    ///     assert!(batcher.commit_rw().is_ok());
    /// }
    ///
    /// // check if the batched write was successful
    /// virt_mem.read_raw_into(addr, &mut read_data).unwrap();
    /// assert_eq!(read_data, write_data);
    /// ```
    pub fn write_raw_iter(&mut self, iter: impl WriteIterator<'a>) -> &mut Self {
        self.write_list.extend(iter);
        self
    }

    /// Reads data from memory and stores it in the provided buffer.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to start reading from.
    /// * `out` - The buffer to store the read data in.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::DummyMemory;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, virt_base) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    ///
    /// let addr = virt_base; // some arbitrary address
    /// let mut buffer = [0u8; 4];
    ///
    /// let mut batcher = MemoryViewBatcher::new(&mut virt_mem);
    ///
    /// // read 4 bytes from some address and store the result in `buffer`
    /// batcher.read_raw_into(addr, &mut buffer);
    ///
    /// // commit the batch to memory, this is optional and just used to check if the operations succeed
    /// batcher.commit_rw().unwrap();
    /// ```
    pub fn read_raw_into<'b: 'a>(&mut self, addr: Address, out: &'b mut [u8]) -> &mut Self {
        self.read_raw_iter(std::iter::once(CTup2(addr, out.into())))
    }

    /// Reads data from memory and stores it in the provided buffer.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to read from.
    /// * `out` - The buffer to store the read data.
    ///
    /// # Example
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::DummyMemory;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, virt_base) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    ///
    /// let addr = virt_base; // some arbitrary address
    ///
    /// // writes the text 'hello world' to the specified address in memory
    /// virt_mem.write(addr, b"hello world").unwrap();
    ///
    /// let mut buffer = [0u8; 11];
    ///
    /// {
    ///     // creates a batcher and reads 11 bytes from memory
    ///     let mut batcher = MemoryViewBatcher::new(&mut virt_mem);
    ///     batcher.read_into(addr, &mut buffer);
    ///
    ///     // commit the batch to memory, this is optional and just used to check if the operations succeed
    ///     batcher.commit_rw().unwrap();
    /// }
    ///
    /// // compare the memory
    /// assert_eq!(&buffer, b"hello world");
    /// ```
    pub fn read_into<'b: 'a, F: Pod + ?Sized>(
        &mut self,
        addr: Address,
        out: &'b mut F,
    ) -> &mut Self {
        self.read_raw_into(addr, out.as_bytes_mut())
    }

    /// Writes data to memory from the provided buffer.
    ///
    /// # Example
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::DummyMemory;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, virt_base) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    ///
    /// let addr = virt_base; // some arbitrary address
    /// let write_data = [0x10, 0x20, 0x30, 0x40];
    /// let mut read_data = [0u8; 4];
    ///
    /// {
    ///     // create batcher in a new scope
    ///     let mut batcher = MemoryViewBatcher::new(&mut virt_mem);
    ///
    ///     // writes the block to memory at the specified address
    ///     batcher.write_raw_into(addr, &write_data);
    ///
    ///     // commit the batch to memory, this is optional and just used to check if the operations succeed
    ///     assert!(batcher.commit_rw().is_ok());
    /// }
    ///
    /// // check if the write succeeded
    /// virt_mem.read_raw_into(addr, &mut read_data).unwrap();
    /// assert_eq!(read_data, write_data);
    /// ```
    pub fn write_raw_into<'b: 'a>(&mut self, addr: Address, out: &'b [u8]) -> &mut Self {
        self.write_raw_iter(std::iter::once(CTup2(addr, out.into())))
    }

    /// Serializes data and writes it to memory.
    ///
    /// # Example
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::DummyMemory;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, virt_base) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    ///
    /// let addr = virt_base; // some arbitrary address
    /// let write_data = 0xdeadbeefu64;
    /// let mut read_data = 0u64;
    ///
    /// {
    ///     // create batcher in a new scope
    ///     let mut batcher = MemoryViewBatcher::new(&mut virt_mem);
    ///
    ///     // writes the block to memory at the specified address
    ///     batcher.write_into(addr, &write_data);
    ///
    ///     // commit the batch to memory, this is optional and just used to check if the operations succeed
    ///     assert!(batcher.commit_rw().is_ok());
    /// }
    ///
    /// // check if the write succeeded
    /// virt_mem.read_into(addr, &mut read_data).unwrap();
    /// assert_eq!(read_data, write_data);
    /// ```
    pub fn write_into<'b: 'a, F: Pod + ?Sized>(&mut self, addr: Address, out: &'b F) -> &mut Self {
        self.write_raw_into(addr, out.as_bytes())
    }
}

impl<'a, T: MemoryView> Drop for MemoryViewBatcher<'a, T> {
    fn drop(&mut self) {
        let _ = self.commit_rw();
    }
}
