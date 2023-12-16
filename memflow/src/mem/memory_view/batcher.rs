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
///
/// let mut mem = DummyMemory::new(size::mb(1));
/// let mut batcher = MemoryViewBatcher::new(&mut mem);
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
    ///
    /// let mut mem = DummyMemory::new(size::mb(1));
    /// let mut batcher = MemoryViewBatcher::new(&mut mem);
    /// ```
    pub fn new(vmem: &'a mut T) -> Self {
        Self {
            vmem,
            read_list: vec![],
            write_list: vec![],
        }
    }

    /// Reserves capacity for the read list.
    ///
    /// # Arguments
    ///
    /// * `capacity`: The number of elements to reserve space for.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::DummyMemory;
    ///
    /// let mut mem = DummyMemory::new(size::mb(1));
    /// let mut batcher = MemoryViewBatcher::new(&mut mem);
    ///
    /// batcher.read_prealloc(10);
    /// ```
    pub fn read_prealloc(&mut self, capacity: usize) -> &mut Self {
        self.read_list.reserve(capacity);
        self
    }

    /// Commits the reads and writes in the batch to memory.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::DummyMemory;
    ///
    /// let mut mem = DummyMemory::new(size::mb(1));
    /// let mut batcher = MemoryViewBatcher::new(&mut mem);
    ///
    /// // call read / write functions here
    ///
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

    /// Appends a batch of raw read data to the batch.
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
    ///
    /// let mut mem = DummyMemory::new(size::mb(1));
    /// let mut batcher = MemoryViewBatcher::new(&mut mem);
    ///
    /// let addr = Address::from(0x1000);
    /// let mut buf = [0u8; 8];
    ///
    /// batcher.read_raw_iter(std::iter::once(CTup2(addr, buf.as_mut())).into_iter());
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
    ///
    /// let mut mem = DummyMemory::new(size::mb(1));
    /// let mut batch = MemoryViewBatcher::new(&mut mem);
    ///
    /// let write_data = &[0x10, 0x20, 0x30, 0x40];
    /// batch.write_raw_into(Address::from(0x1000), write_data);
    ///
    /// assert!(batch.commit_rw().is_ok());
    /// let read_data = &mut [0u8; 4];
    /// mem.read_raw(Address::from(0x1000), read_data).unwrap();
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
    ///
    /// let mut mem = DummyMemory::new(size::mb(1));
    /// let mut batcher = MemoryViewBatcher::new(&mut mem);
    /// let mut buffer = [0u8; 4];
    ///
    /// // Read 4 bytes from address 0x0 and store the result in `buffer`
    /// batcher.read_raw_into(Address::from(0x0), &mut buffer);
    ///
    /// // Commit the read request to memory
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
    ///
    /// let mut mem = DummyMemory::new(0x1000);
    /// mem.write_bytes(Address::from(0x1000), b"hello world").unwrap();
    ///
    /// let mut buffer = [0u8; 11];
    ///
    /// let mut batcher = MemoryViewBatcher::new(&mut mem);
    /// batcher.read_into(Address::from(0x1000), &mut buffer);
    /// batcher.commit_rw().unwrap();
    ///
    /// assert_eq!(buffer, b"hello world");
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
    ///
    /// let mut mem = DummyMemory::new(size::mb(1));
    /// let mut batch = MemoryViewBatcher::new(&mut mem);
    ///
    /// let write_data = &[0x10, 0x20, 0x30, 0x40];
    /// batch.write_raw_into(Address::from(0x1000), write_data);
    ///
    /// assert!(batch.commit_rw().is_ok());
    /// let read_data = &mut [0u8; 4];
    /// mem.read_raw(Address::from(0x1000), read_data).unwrap();
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
    ///
    /// let mut mem = DummyMemory::new(size::mb(1));
    /// let mut batch = MemoryViewBatcher::new(&mut mem);
    ///
    /// let write_data = 0xdeadbeefu64;
    /// batch.write_into(Address::from(0x1000), &write_data);
    ///
    /// assert!(batch.commit_rw().is_ok());
    /// let read_data = &mut 0u64;
    /// mem.read_into(Address::from(0x1000), read_data).unwrap();
    /// assert_eq!(*read_data, write_data);
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
