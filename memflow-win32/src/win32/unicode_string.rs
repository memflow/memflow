use std::prelude::v1::*;

use crate::error::{Error, Result};

use std::convert::TryInto;

use memflow::architecture::{ArchitectureObj, Endianess};
use memflow::mem::VirtualMemory;
use memflow::types::Address;

use widestring::U16CString;

pub trait VirtualReadUnicodeString {
    fn virt_read_unicode_string(
        &mut self,
        proc_arch: ArchitectureObj,
        addr: Address,
    ) -> Result<String>;
}

// TODO: split up cpu and proc arch in read_helper.rs
impl<'a, T: VirtualMemory> VirtualReadUnicodeString for T {
    fn virt_read_unicode_string(
        &mut self,
        proc_arch: ArchitectureObj,
        addr: Address,
    ) -> Result<String> {
        /*
        typedef struct _windows_unicode_string32 {
            uint16_t length;
            uint16_t maximum_length;
            uint32_t pBuffer; // pointer to string contents
        } __attribute__((packed)) win32_unicode_string_t;

        typedef struct _windows_unicode_string64 {
            uint16_t length;
            uint16_t maximum_length;
            uint32_t padding; // align pBuffer
            uint64_t pBuffer; // pointer to string contents
        } __attribute__((packed)) win64_unicode_string_t;
        */

        // length is always the first entry
        let mut length = 0u16;
        self.virt_read_into(addr, &mut length)?;
        if length == 0 {
            return Err(Error::Unicode("unable to read unicode string length"));
        }

        // TODO: chek if length exceeds limit
        // buffer is either aligned at 4 or 8
        let buffer = match proc_arch.bits() {
            64 => self.virt_read_addr64(addr + 8)?,
            32 => self.virt_read_addr32(addr + 4)?,
            _ => {
                return Err(Error::InvalidArchitecture);
            }
        };
        if buffer.is_null() {
            return Err(Error::Unicode("unable to read unicode string length"));
        }

        // check if buffer length is mod 2 (utf-16)
        if length % 2 != 0 {
            return Err(Error::Unicode(
                "unicode string length is not a multiple of two",
            ));
        }

        // read buffer
        let mut content = vec![0; length as usize + 2];
        self.virt_read_raw_into(buffer, &mut content)?;
        content[length as usize] = 0;
        content[length as usize + 1] = 0;

        // TODO: check length % 2 == 0

        let content16 = content
            .chunks_exact(2)
            .map(|b| b[0..2].try_into().map_err(|_| Error::Bounds))
            .filter_map(Result::ok)
            .map(|b| match proc_arch.endianess() {
                Endianess::LittleEndian => u16::from_le_bytes(b),
                Endianess::BigEndian => u16::from_be_bytes(b),
            })
            .collect::<Vec<u16>>();
        Ok(U16CString::from_vec_with_nul(content16)
            .map_err(|_| Error::Encoding)?
            .to_string_lossy())
    }
}
