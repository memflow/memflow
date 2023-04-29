//! The cursor module provides cursor implementations around
//! the [`MemoryView`] trait.
//!
//! The cursor provides the [`Read`](https://doc.rust-lang.org/std/io/trait.Read.html),
//! [`Write`](https://doc.rust-lang.org/std/io/trait.Write.html) and [`Seek`](https://doc.rust-lang.org/std/io/trait.Seek.html) traits
//! for the underlying Memory object.
//!
//! # Examples:
//!
//! ```
//! use std::io::{self, Read, Write, Seek};
//!
//! use memflow::dummy::DummyMemory;
//! use memflow::types::size;
//! use memflow::mem::{MemoryCursor, PhysicalMemory};
//!
//! fn main() -> io::Result<()> {
//!     let phys_mem = DummyMemory::new(size::mb(16));
//!     let mut cursor = MemoryCursor::new(phys_mem.into_phys_view());
//!
//!     // read up to 10 bytes
//!     let mut buffer = [0; 10];
//!     cursor.read(&mut buffer)?;
//!
//!     // write the previously read 10 bytes again
//!     cursor.seek(io::SeekFrom::Start(0));
//!     cursor.write(&buffer)?;
//!
//!     Ok(())
//! }
//! ```

use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};

use super::MemoryView;
use crate::types::{umem, Address};

/// MemoryCursor implments a Cursor around the [`MemoryView`] trait.
///
/// The cursor provides the [`Read`](https://doc.rust-lang.org/std/io/trait.Read.html),
/// [`Write`](https://doc.rust-lang.org/std/io/trait.Write.html) and [`Seek`](https://doc.rust-lang.org/std/io/trait.Seek.html) traits
/// for the underlying [`MemoryView`] object.
///
/// # Examples:
///
/// ```
/// use std::io::{self, Read, Write, Seek};
///
/// use memflow::dummy::{DummyOs, DummyMemory};
/// use memflow::types::size;
/// use memflow::mem::{DirectTranslate, VirtualDma, MemoryCursor};
/// use memflow::architecture::x86::x64;
///
/// fn main() -> io::Result<()> {
///     // setup a pseudo virtual memory reader
///     let phys_mem = DummyMemory::new(size::mb(16));
///     let mut os = DummyOs::new(phys_mem);
///     let (dtb, virt_base) = os.alloc_dtb(size::mb(8), &[]);
///     let phys_mem = os.into_inner();
///     let translator = x64::new_translator(dtb);
///
///     let virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
///
///     // create the actual cursor and seek it to the dummy virt_base
///     let mut cursor = MemoryCursor::new(virt_mem);
///     cursor.seek(io::SeekFrom::Start(virt_base.to_umem() as u64))?;
///
///     // read up to 10 bytes
///     let mut buffer = [0; 10];
///     cursor.read(&mut buffer)?;
///
///     // write the previously read 10 bytes again
///     cursor.seek(io::SeekFrom::Start(virt_base.to_umem() as u64))?;
///     cursor.write(&buffer)?;
///
///     Ok(())
/// }
/// ```
pub struct MemoryCursor<T> {
    mem: T,
    address: Address,
}

impl<T: MemoryView> MemoryCursor<T> {
    /// Creates a new MemoryCursor by wrapping around a [`MemoryView`] object.
    ///
    /// Cursor initial position is `0`.
    ///
    /// # Examples:
    ///
    /// Borrowing a [`MemoryView`] object:
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::MemoryCursor;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::mem::{DirectTranslate, VirtualDma};
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, _) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    /// let mut cursor = MemoryCursor::new(virt_mem);
    /// ```
    ///
    /// Taking (temporary) ownership of a [`MemoryView`] object:
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::MemoryCursor;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::mem::{DirectTranslate, VirtualDma};
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, _) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    /// let mut cursor = MemoryCursor::new(virt_mem);
    /// ```
    pub fn new(mem: T) -> Self {
        Self {
            mem,
            address: Address::NULL,
        }
    }

    /// Creates a new MemoryCursor by wrapping around a [`MemoryView`] object
    /// at the desired starting position.
    ///
    /// Cursor initial position is * `address`.
    ///
    /// # Examples:
    ///
    /// Borrowing a [`MemoryView`] object:
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::MemoryCursor;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::mem::{DirectTranslate, VirtualDma};
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, _) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    /// let mut cursor = MemoryCursor::at(virt_mem, 0x1000.into());
    /// ```
    ///
    /// Taking (temporary) ownership of a [`MemoryView`] object:
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::MemoryCursor;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::mem::{DirectTranslate, VirtualDma};
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, _) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    /// let mut cursor = MemoryCursor::at(virt_mem, 0x1000.into());
    /// ```
    pub fn at(mem: T, address: Address) -> Self {
        Self { mem, address }
    }

    /// Consumes this cursor, returning the underlying [`MemoryView`] object.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::MemoryCursor;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::mem::{DirectTranslate, VirtualDma};
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, _) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut cursor = MemoryCursor::new(VirtualDma::new(phys_mem, x64::ARCH, translator));
    ///
    /// let phys_mem = cursor.into_inner();
    /// ```
    pub fn into_inner(self) -> T {
        self.mem
    }

    /// Gets a reference to the underlying [`MemoryView`] object in this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::MemoryCursor;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::mem::{DirectTranslate, VirtualDma};
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, _) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut cursor = MemoryCursor::new(VirtualDma::new(phys_mem, x64::ARCH, translator));
    ///
    /// let reference = cursor.get_ref();
    /// ```
    pub fn get_ref(&self) -> &T {
        &self.mem
    }

    /// Gets a mutable reference to the underlying [`MemoryView`] object in this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::MemoryCursor;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::mem::{DirectTranslate, VirtualDma};
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, _) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut cursor = MemoryCursor::new(VirtualDma::new(phys_mem, x64::ARCH, translator));
    ///
    /// let reference = cursor.get_mut();
    /// ```
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.mem
    }

    /// Returns the current address of this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Seek, SeekFrom};
    ///
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::{Address, size};
    /// use memflow::mem::MemoryCursor;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::mem::{DirectTranslate, VirtualDma};
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, _) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut cursor = MemoryCursor::new(VirtualDma::new(phys_mem, x64::ARCH, translator));
    ///
    /// assert_eq!(cursor.address(), Address::NULL);
    ///
    /// cursor.seek(SeekFrom::Current(2)).unwrap();
    /// assert_eq!(cursor.address(), Address::from(2));
    ///
    /// cursor.seek(SeekFrom::Current(-1)).unwrap();
    /// assert_eq!(cursor.address(), Address::from(1));
    /// ```
    pub fn address(&self) -> Address {
        self.address
    }

    /// Sets the address of this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::{Address, size};
    /// use memflow::mem::MemoryCursor;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::mem::{DirectTranslate, VirtualDma};
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, _) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut cursor = MemoryCursor::new(VirtualDma::new(phys_mem, x64::ARCH, translator));
    ///
    /// assert_eq!(cursor.address(), Address::NULL);
    ///
    /// cursor.set_address(Address::from(2));
    /// assert_eq!(cursor.address(), Address::from(2));
    ///
    /// cursor.set_address(Address::from(4));
    /// assert_eq!(cursor.address(), Address::from(4));
    /// ```
    pub fn set_address(&mut self, address: Address) {
        self.address = address;
    }
}

impl<T: MemoryView> Read for MemoryCursor<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.mem
            .read_raw_into(self.address, buf)
            .map_err(|err| Error::new(ErrorKind::UnexpectedEof, err))?;
        self.address = (self.address.to_umem() + buf.len() as umem).into();
        Ok(buf.len())
    }
}

impl<T: MemoryView> Write for MemoryCursor<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.mem
            .write_raw(self.address, buf)
            .map_err(|err| Error::new(ErrorKind::UnexpectedEof, err))?;
        self.address = (self.address.to_umem() + buf.len() as umem).into();
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<T: MemoryView> Seek for MemoryCursor<T> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let target_pos = match pos {
            SeekFrom::Start(offs) => offs,
            // TODO: do we need +1?
            SeekFrom::End(offs) => self
                .mem
                .metadata()
                .max_address
                .to_umem()
                .wrapping_add(1)
                .wrapping_add(offs as umem) as u64,
            SeekFrom::Current(offs) => self.address.to_umem().wrapping_add(offs as umem) as u64,
        };

        self.address = target_pos.into();
        Ok(target_pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::architecture::x86::{x64, X86VirtualTranslate};
    use crate::dummy::{DummyMemory, DummyOs};
    use crate::mem::{DirectTranslate, PhysicalMemory, VirtualDma};
    use crate::types::{mem, size};

    fn dummy_phys_mem() -> DummyMemory {
        DummyMemory::new(size::mb(1))
    }

    #[test]
    fn physical_seek() {
        let mut phys_mem = dummy_phys_mem();
        let mut cursor = MemoryCursor::new(phys_mem.phys_view());

        assert_eq!(cursor.stream_position().unwrap(), 0);
        assert_eq!(cursor.seek(SeekFrom::Current(1024)).unwrap(), 1024);
        assert_eq!(cursor.seek(SeekFrom::Current(1024)).unwrap(), 2048);
        assert_eq!(cursor.seek(SeekFrom::Current(-1024)).unwrap(), 1024);

        assert_eq!(cursor.seek(SeekFrom::Start(512)).unwrap(), 512);

        assert_eq!(
            cursor.seek(SeekFrom::End(-512)).unwrap(),
            mem::mb(1) as u64 - 512
        );
    }

    #[test]
    fn physical_read_write() {
        let mut phys_mem = dummy_phys_mem();
        let mut cursor = MemoryCursor::new(phys_mem.phys_view());

        let write_buf = [0xAu8, 0xB, 0xC, 0xD];
        assert_eq!(cursor.write(&write_buf).unwrap(), 4); // write 4 bytes from the start
        assert_eq!(cursor.stream_position().unwrap(), 4); // check if cursor moved 4 bytes

        let mut read_buf = [0u8; 4];
        assert!(cursor.rewind().is_ok()); // roll back cursor to start
        assert_eq!(cursor.read(&mut read_buf).unwrap(), 4); // read 4 bytes from the start
        assert_eq!(read_buf, write_buf); // compare buffers
    }

    #[test]
    fn physical_read_write_seek() {
        let mut phys_mem = dummy_phys_mem();
        let mut cursor = MemoryCursor::new(phys_mem.phys_view());

        assert_eq!(cursor.seek(SeekFrom::Start(512)).unwrap(), 512); // seek to 512th byte

        let write_buf = [0xAu8, 0xB, 0xC, 0xD];
        assert_eq!(cursor.write(&write_buf).unwrap(), 4); // write 4 bytes from 512th byte
        assert_eq!(cursor.stream_position().unwrap(), 512 + 4); // check if cursor moved 4 bytes

        let mut read_buf = [0u8; 4];
        assert_eq!(cursor.seek(SeekFrom::Start(512)).unwrap(), 512); // roll back cursor to 512th byte
        assert_eq!(cursor.read(&mut read_buf).unwrap(), 4); // read 4 bytes from the 512th byte
        assert_eq!(read_buf, write_buf); // compare buffers
    }

    fn dummy_virt_mem() -> (
        VirtualDma<DummyMemory, DirectTranslate, X86VirtualTranslate>,
        Address,
    ) {
        let phys_mem = DummyMemory::new(size::mb(1));
        let mut os = DummyOs::new(phys_mem);
        let (dtb, virt_base) = os.alloc_dtb(size::mb(1), &[]);
        let phys_mem = os.into_inner();
        let translator = x64::new_translator(dtb);
        (VirtualDma::new(phys_mem, x64::ARCH, translator), virt_base)
    }

    #[test]
    fn virtual_seek() {
        let (virt_mem, _) = dummy_virt_mem();
        let mut cursor = MemoryCursor::new(virt_mem);

        assert_eq!(cursor.stream_position().unwrap(), 0);
        assert_eq!(cursor.seek(SeekFrom::Current(1024)).unwrap(), 1024);
        assert_eq!(cursor.seek(SeekFrom::Current(1024)).unwrap(), 2048);
        assert_eq!(cursor.seek(SeekFrom::Current(-1024)).unwrap(), 1024);

        assert_eq!(cursor.seek(SeekFrom::Start(512)).unwrap(), 512);
    }

    #[test]
    fn virtual_read_write() {
        let (virt_mem, virt_base) = dummy_virt_mem();
        let mut cursor = MemoryCursor::new(virt_mem);

        let write_buf = [0xAu8, 0xB, 0xC, 0xD];
        assert_eq!(
            cursor
                .seek(SeekFrom::Start(virt_base.to_umem() as u64))
                .unwrap(),
            virt_base.to_umem() as u64
        );
        assert_eq!(cursor.write(&write_buf).unwrap(), 4); // write 4 bytes from the start
        assert_eq!(
            cursor.stream_position().unwrap(),
            virt_base.to_umem() as u64 + 4
        ); // check if cursor moved 4 bytes

        let mut read_buf = [0u8; 4];
        assert_eq!(
            cursor
                .seek(SeekFrom::Start(virt_base.to_umem() as u64))
                .unwrap(),
            virt_base.to_umem() as u64
        ); // roll back cursor to start
        assert_eq!(cursor.read(&mut read_buf).unwrap(), 4); // read 4 bytes from the start
        assert_eq!(read_buf, write_buf); // compare buffers
    }

    #[test]
    fn virtual_read_write_seek() {
        let (virt_mem, virt_base) = dummy_virt_mem();
        let mut cursor = MemoryCursor::new(virt_mem);

        assert_eq!(
            cursor
                .seek(SeekFrom::Start(virt_base.to_umem() as u64 + 512))
                .unwrap(),
            virt_base.to_umem() as u64 + 512
        ); // seek to 512th byte

        let write_buf = [0xAu8, 0xB, 0xC, 0xD];
        assert_eq!(cursor.write(&write_buf).unwrap(), 4); // write 4 bytes from 512th byte
        assert_eq!(
            cursor.stream_position().unwrap(),
            virt_base.to_umem() as u64 + 512 + 4
        ); // check if cursor moved 4 bytes

        let mut read_buf = [0u8; 4];
        assert_eq!(
            cursor
                .seek(SeekFrom::Start(virt_base.to_umem() as u64 + 512))
                .unwrap(),
            virt_base.to_umem() as u64 + 512
        ); // roll back cursor to 512th byte
        assert_eq!(cursor.read(&mut read_buf).unwrap(), 4); // read 4 bytes from the 512th byte
        assert_eq!(read_buf, write_buf); // compare buffers
    }
}
