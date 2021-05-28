/*!
The cursor module provides cursor implementations around
the [`PhysicalMemory`] and [`VirtualMemory`] set of traits.

The cursor provides the [`Read`](https://doc.rust-lang.org/std/io/trait.Read.html),
[`Write`](https://doc.rust-lang.org/std/io/trait.Write.html) and [`Seek`](https://doc.rust-lang.org/std/io/trait.Seek.html) traits
for the underlying Memory object.

# Examples:

```
use std::io::{self, Read, Write, Seek};

use memflow::dummy::DummyMemory;
use memflow::types::size;
use memflow::mem::PhysicalMemoryCursor;

fn main() -> io::Result<()> {
    let mut phys_mem = DummyMemory::new(size::mb(16));
    let mut cursor = PhysicalMemoryCursor::new(&mut phys_mem);

    // read up to 10 bytes
    let mut buffer = [0; 10];
    cursor.read(&mut buffer)?;

    // write the previously read 10 bytes again
    cursor.seek(io::SeekFrom::Start(0));
    cursor.write(&buffer)?;

    Ok(())
}
```
*/

use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};

use super::{PhysicalMemory, PhysicalMemoryMetadata, VirtualMemory};
use crate::types::{Address, PhysicalAddress};

/// PhysicalMemoryCursor implments a Cursor around the [`PhysicalMemory`] trait.
///
/// The cursor provides the [`Read`](https://doc.rust-lang.org/std/io/trait.Read.html),
/// [`Write`](https://doc.rust-lang.org/std/io/trait.Write.html) and [`Seek`](https://doc.rust-lang.org/std/io/trait.Seek.html) traits
/// for the underlying [`PhysicalMemory`] object.
///
/// # Examples:
///
/// ```
/// use std::io::{self, Read, Write, Seek};
///
/// use memflow::dummy::DummyMemory;
/// use memflow::types::size;
/// use memflow::mem::PhysicalMemoryCursor;
///
/// fn main() -> io::Result<()> {
///     let mut phys_mem = DummyMemory::new(size::mb(16));
///     let mut cursor = PhysicalMemoryCursor::new(&mut phys_mem);
///
///     // read up to 10 bytes
///     let mut buffer = [0; 10];
///     cursor.read(&mut buffer)?;
///
///     // write the previously read 10 bytes again
///     cursor.seek(io::SeekFrom::Start(0));
///     cursor.write(&buffer)?;
///
///     Ok(())
/// }
/// ```
pub struct PhysicalMemoryCursor<T> {
    phys_mem: T,
    metadata: PhysicalMemoryMetadata,
    address: PhysicalAddress,
}

impl<T: PhysicalMemory> PhysicalMemoryCursor<T> {
    /// Creates a new PhysicalMemoryCursor by wrapping around a [`PhysicalMemory`] object.
    ///
    /// Cursor initial position is `0`.
    ///
    /// # Examples:
    ///
    /// Borrowing a [`PhysicalMemory`] object:
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::PhysicalMemoryCursor;
    ///
    /// let mut phys_mem = DummyMemory::new(size::mb(16));
    /// let mut cursor = PhysicalMemoryCursor::new(&mut phys_mem);
    /// ```
    ///
    /// Taking (temporary) ownership of a [`PhysicalMemory`] object:
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::PhysicalMemoryCursor;
    ///
    /// let phys_mem = DummyMemory::new(size::mb(16));
    /// let mut cursor = PhysicalMemoryCursor::new(phys_mem);
    /// ```
    pub fn new(phys_mem: T) -> Self {
        let metadata = phys_mem.metadata();
        Self {
            phys_mem,
            metadata,
            address: PhysicalAddress::NULL,
        }
    }

    /// Creates a new PhysicalMemoryCursor by wrapping around a [`PhysicalMemory`] object
    /// at the desired starting position.
    ///
    /// Cursor initial position is * `address`.
    ///
    /// # Examples:
    ///
    /// Borrowing a [`PhysicalMemory`] object:
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::PhysicalMemoryCursor;
    ///
    /// let mut phys_mem = DummyMemory::new(size::mb(16));
    /// let mut cursor = PhysicalMemoryCursor::at(&mut phys_mem, 0x1000.into());
    /// ```
    ///
    /// Taking (temporary) ownership of a [`PhysicalMemory`] object:
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::PhysicalMemoryCursor;
    ///
    /// let phys_mem = DummyMemory::new(size::mb(16));
    /// let mut cursor = PhysicalMemoryCursor::at(phys_mem, 0x1000.into());
    /// ```
    pub fn at(phys_mem: T, address: PhysicalAddress) -> Self {
        let metadata = phys_mem.metadata();
        Self {
            phys_mem,
            metadata,
            address,
        }
    }

    /// Consumes this cursor, returning the underlying [`PhysicalMemory`] object.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::PhysicalMemoryCursor;
    ///
    /// let mut cursor = PhysicalMemoryCursor::new(DummyMemory::new(size::mb(16)));
    ///
    /// let phys_mem = cursor.into_inner();
    /// ```
    pub fn into_inner(self) -> T {
        self.phys_mem
    }

    /// Gets a reference to the underlying [`PhysicalMemory`] object in this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::PhysicalMemoryCursor;
    ///
    /// let cursor = PhysicalMemoryCursor::new(DummyMemory::new(size::mb(16)));
    ///
    /// let reference = cursor.get_ref();
    /// ```
    pub fn get_ref(&self) -> &T {
        &self.phys_mem
    }

    /// Gets a mutable reference to the underlying [`PhysicalMemory`] object in this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::PhysicalMemoryCursor;
    ///
    /// let mut cursor = PhysicalMemoryCursor::new(DummyMemory::new(size::mb(16)));
    ///
    /// let reference = cursor.get_mut();
    /// ```
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.phys_mem
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
    /// use memflow::mem::PhysicalMemoryCursor;
    ///
    /// let mut cursor = PhysicalMemoryCursor::new(DummyMemory::new(size::mb(16)));
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
        self.address.address()
    }

    /// Sets the address of this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::{Address, size};
    /// use memflow::mem::PhysicalMemoryCursor;
    ///
    /// let mut cursor = PhysicalMemoryCursor::new(DummyMemory::new(size::mb(16)));
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
        self.address = address.into();
    }
}

impl<T: PhysicalMemory> Read for PhysicalMemoryCursor<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.phys_mem
            .phys_read_raw_into(self.address, buf)
            .map_err(|err| Error::new(ErrorKind::UnexpectedEof, err))?;
        self.address = (self.address.as_u64() + buf.len() as u64).into();
        Ok(buf.len())
    }
}

impl<T: PhysicalMemory> Write for PhysicalMemoryCursor<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.phys_mem
            .phys_write_raw(self.address, buf)
            .map_err(|err| Error::new(ErrorKind::UnexpectedEof, err))?;
        self.address = (self.address.as_u64() + buf.len() as u64).into();
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<T: PhysicalMemory> Seek for PhysicalMemoryCursor<T> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let target_pos = match pos {
            SeekFrom::Start(offs) => offs,
            SeekFrom::End(offs) => (self.metadata.size as u64).wrapping_add(offs as u64),
            SeekFrom::Current(offs) => self.address.as_u64().wrapping_add(offs as u64),
        };

        self.address = target_pos.into();
        Ok(target_pos)
    }
}

/// VirtualMemoryCursor implments a Cursor around the [`VirtualMemory`] trait.
///
/// The cursor provides the [`Read`](https://doc.rust-lang.org/std/io/trait.Read.html),
/// [`Write`](https://doc.rust-lang.org/std/io/trait.Write.html) and [`Seek`](https://doc.rust-lang.org/std/io/trait.Seek.html) traits
/// for the underlying [`VirtualMemory`] object.
///
/// # Examples:
///
/// ```
/// use std::io::{self, Read, Write, Seek};
///
/// use memflow::dummy::{DummyOs, DummyMemory};
/// use memflow::types::size;
/// use memflow::mem::{DirectTranslate, VirtualDma, VirtualMemoryCursor};
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
///     let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
///
///     // create the actual cursor and seek it to the dummy virt_base
///     let mut cursor = VirtualMemoryCursor::new(&mut virt_mem);
///     cursor.seek(io::SeekFrom::Start(virt_base.as_u64()))?;
///
///     // read up to 10 bytes
///     let mut buffer = [0; 10];
///     cursor.read(&mut buffer)?;
///
///     // write the previously read 10 bytes again
///     cursor.seek(io::SeekFrom::Start(virt_base.as_u64()))?;
///     cursor.write(&buffer)?;
///
///     Ok(())
/// }
/// ```
pub struct VirtualMemoryCursor<T> {
    virt_mem: T,
    address: Address,
}

impl<T: VirtualMemory> VirtualMemoryCursor<T> {
    /// Creates a new VirtualMemoryCursor by wrapping around a [`VirtualMemory`] object.
    ///
    /// Cursor initial position is `0`.
    ///
    /// # Examples:
    ///
    /// Borrowing a [`VirtualMemory`] object:
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::VirtualMemoryCursor;
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
    /// let mut cursor = VirtualMemoryCursor::new(&mut virt_mem);
    /// ```
    ///
    /// Taking (temporary) ownership of a [`VirtualMemory`] object:
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::VirtualMemoryCursor;
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
    /// let mut cursor = VirtualMemoryCursor::new(virt_mem);
    /// ```
    pub fn new(virt_mem: T) -> Self {
        Self {
            virt_mem,
            address: Address::NULL,
        }
    }

    /// Creates a new VirtualMemoryCursor by wrapping around a [`VirtualMemory`] object
    /// at the desired starting position.
    ///
    /// Cursor initial position is * `address`.
    ///
    /// # Examples:
    ///
    /// Borrowing a [`VirtualMemory`] object:
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::VirtualMemoryCursor;
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
    /// let mut cursor = VirtualMemoryCursor::at(&mut virt_mem, 0x1000.into());
    /// ```
    ///
    /// Taking (temporary) ownership of a [`VirtualMemory`] object:
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::VirtualMemoryCursor;
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
    /// let mut cursor = VirtualMemoryCursor::at(virt_mem, 0x1000.into());
    /// ```
    pub fn at(virt_mem: T, address: Address) -> Self {
        Self { virt_mem, address }
    }

    /// Consumes this cursor, returning the underlying [`VirtualMemory`] object.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::VirtualMemoryCursor;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::mem::{DirectTranslate, VirtualDma};
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, _) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut cursor = VirtualMemoryCursor::new(VirtualDma::new(phys_mem, x64::ARCH, translator));
    ///
    /// let phys_mem = cursor.into_inner();
    /// ```
    pub fn into_inner(self) -> T {
        self.virt_mem
    }

    /// Gets a reference to the underlying [`VirtualMemory`] object in this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::VirtualMemoryCursor;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::mem::{DirectTranslate, VirtualDma};
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, _) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut cursor = VirtualMemoryCursor::new(VirtualDma::new(phys_mem, x64::ARCH, translator));
    ///
    /// let reference = cursor.get_ref();
    /// ```
    pub fn get_ref(&self) -> &T {
        &self.virt_mem
    }

    /// Gets a mutable reference to the underlying [`VirtualMemory`] object in this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::dummy::DummyMemory;
    /// use memflow::types::size;
    /// use memflow::mem::VirtualMemoryCursor;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::mem::{DirectTranslate, VirtualDma};
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, _) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut cursor = VirtualMemoryCursor::new(VirtualDma::new(phys_mem, x64::ARCH, translator));
    ///
    /// let reference = cursor.get_mut();
    /// ```
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.virt_mem
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
    /// use memflow::mem::VirtualMemoryCursor;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::mem::{DirectTranslate, VirtualDma};
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, _) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut cursor = VirtualMemoryCursor::new(VirtualDma::new(phys_mem, x64::ARCH, translator));
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
    /// use memflow::mem::VirtualMemoryCursor;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::mem::{DirectTranslate, VirtualDma};
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, _) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut cursor = VirtualMemoryCursor::new(VirtualDma::new(phys_mem, x64::ARCH, translator));
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

impl<T: VirtualMemory> Read for VirtualMemoryCursor<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.virt_mem
            .virt_read_raw_into(self.address, buf)
            .map_err(|err| Error::new(ErrorKind::UnexpectedEof, err))?;
        self.address = (self.address.as_u64() + buf.len() as u64).into();
        Ok(buf.len())
    }
}

impl<T: VirtualMemory> Write for VirtualMemoryCursor<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.virt_mem
            .virt_write_raw(self.address, buf)
            .map_err(|err| Error::new(ErrorKind::UnexpectedEof, err))?;
        self.address = (self.address.as_u64() + buf.len() as u64).into();
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<T: VirtualMemory> Seek for VirtualMemoryCursor<T> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let target_pos = match pos {
            SeekFrom::Start(offs) => offs,
            SeekFrom::End(offs) => u64::MAX.wrapping_add(offs as u64),
            SeekFrom::Current(offs) => self.address.as_u64().wrapping_add(offs as u64),
        };

        self.address = target_pos.into();
        Ok(target_pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::architecture::x86::{x64, X86ScopedVirtualTranslate};
    use crate::dummy::{DummyMemory, DummyOs};
    use crate::mem::{DirectTranslate, VirtualDma};
    use crate::types::size;

    fn dummy_phys_mem() -> DummyMemory {
        DummyMemory::new(size::mb(1))
    }

    #[test]
    fn physical_seek() {
        let mut phys_mem = dummy_phys_mem();
        let mut cursor = PhysicalMemoryCursor::new(&mut phys_mem);

        assert_eq!(cursor.seek(SeekFrom::Current(0)).unwrap(), 0);
        assert_eq!(cursor.seek(SeekFrom::Current(1024)).unwrap(), 1024);
        assert_eq!(cursor.seek(SeekFrom::Current(1024)).unwrap(), 2048);
        assert_eq!(cursor.seek(SeekFrom::Current(-1024)).unwrap(), 1024);

        assert_eq!(cursor.seek(SeekFrom::Start(512)).unwrap(), 512);

        assert_eq!(
            cursor.seek(SeekFrom::End(-512)).unwrap(),
            size::mb(1) as u64 - 512
        );
    }

    #[test]
    fn physical_read_write() {
        let mut phys_mem = dummy_phys_mem();
        let mut cursor = PhysicalMemoryCursor::new(&mut phys_mem);

        let write_buf = [0xAu8, 0xB, 0xC, 0xD];
        assert_eq!(cursor.write(&write_buf).unwrap(), 4); // write 4 bytes from the start
        assert_eq!(cursor.seek(SeekFrom::Current(0)).unwrap(), 4); // check if cursor moved 4 bytes

        let mut read_buf = [0u8; 4];
        assert_eq!(cursor.seek(SeekFrom::Start(0)).unwrap(), 0); // roll back cursor to start
        assert_eq!(cursor.read(&mut read_buf).unwrap(), 4); // read 4 bytes from the start
        assert_eq!(read_buf, write_buf); // compare buffers
    }

    #[test]
    fn physical_read_write_seek() {
        let mut phys_mem = dummy_phys_mem();
        let mut cursor = PhysicalMemoryCursor::new(&mut phys_mem);

        assert_eq!(cursor.seek(SeekFrom::Start(512)).unwrap(), 512); // seek to 512th byte

        let write_buf = [0xAu8, 0xB, 0xC, 0xD];
        assert_eq!(cursor.write(&write_buf).unwrap(), 4); // write 4 bytes from 512th byte
        assert_eq!(cursor.seek(SeekFrom::Current(0)).unwrap(), 512 + 4); // check if cursor moved 4 bytes

        let mut read_buf = [0u8; 4];
        assert_eq!(cursor.seek(SeekFrom::Start(512)).unwrap(), 512); // roll back cursor to 512th byte
        assert_eq!(cursor.read(&mut read_buf).unwrap(), 4); // read 4 bytes from the 512th byte
        assert_eq!(read_buf, write_buf); // compare buffers
    }

    fn dummy_virt_mem() -> (
        VirtualDma<DummyMemory, DirectTranslate, X86ScopedVirtualTranslate>,
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
        let mut cursor = VirtualMemoryCursor::new(virt_mem);

        assert_eq!(cursor.seek(SeekFrom::Current(0)).unwrap(), 0);
        assert_eq!(cursor.seek(SeekFrom::Current(1024)).unwrap(), 1024);
        assert_eq!(cursor.seek(SeekFrom::Current(1024)).unwrap(), 2048);
        assert_eq!(cursor.seek(SeekFrom::Current(-1024)).unwrap(), 1024);

        assert_eq!(cursor.seek(SeekFrom::Start(512)).unwrap(), 512);
    }

    #[test]
    fn virtual_read_write() {
        let (virt_mem, virt_base) = dummy_virt_mem();
        let mut cursor = VirtualMemoryCursor::new(virt_mem);

        let write_buf = [0xAu8, 0xB, 0xC, 0xD];
        assert_eq!(
            cursor.seek(SeekFrom::Start(virt_base.as_u64())).unwrap(),
            virt_base.as_u64()
        );
        assert_eq!(cursor.write(&write_buf).unwrap(), 4); // write 4 bytes from the start
        assert_eq!(
            cursor.seek(SeekFrom::Current(0)).unwrap(),
            virt_base.as_u64() + 4
        ); // check if cursor moved 4 bytes

        let mut read_buf = [0u8; 4];
        assert_eq!(
            cursor.seek(SeekFrom::Start(virt_base.as_u64())).unwrap(),
            virt_base.as_u64()
        ); // roll back cursor to start
        assert_eq!(cursor.read(&mut read_buf).unwrap(), 4); // read 4 bytes from the start
        assert_eq!(read_buf, write_buf); // compare buffers
    }

    #[test]
    fn virtual_read_write_seek() {
        let (virt_mem, virt_base) = dummy_virt_mem();
        let mut cursor = VirtualMemoryCursor::new(virt_mem);

        assert_eq!(
            cursor
                .seek(SeekFrom::Start(virt_base.as_u64() + 512))
                .unwrap(),
            virt_base.as_u64() + 512
        ); // seek to 512th byte

        let write_buf = [0xAu8, 0xB, 0xC, 0xD];
        assert_eq!(cursor.write(&write_buf).unwrap(), 4); // write 4 bytes from 512th byte
        assert_eq!(
            cursor.seek(SeekFrom::Current(0)).unwrap(),
            virt_base.as_u64() + 512 + 4
        ); // check if cursor moved 4 bytes

        let mut read_buf = [0u8; 4];
        assert_eq!(
            cursor
                .seek(SeekFrom::Start(virt_base.as_u64() + 512))
                .unwrap(),
            virt_base.as_u64() + 512
        ); // roll back cursor to 512th byte
        assert_eq!(cursor.read(&mut read_buf).unwrap(), 4); // read 4 bytes from the 512th byte
        assert_eq!(read_buf, write_buf); // compare buffers
    }
}
