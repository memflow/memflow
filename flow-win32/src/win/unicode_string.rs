use crate::error::{Error, Result};

use flow_core::address::{Address, Length};
use flow_core::arch::{InstructionSet, SystemArchitecture};
use flow_core::mem::{VirtualReadHelper, VirtualReadHelperFuncs};

use widestring::U16CString;

pub trait VirtualReadUnicodeString {
    fn virt_read_unicode_string(&mut self, addr: Address) -> Result<String>;
}

// TODO: split up cpu and proc arch in read_helper.rs
impl<T: SystemArchitecture + VirtualReadHelper + VirtualReadHelperFuncs> VirtualReadUnicodeString
    for T
{
    fn virt_read_unicode_string(&mut self, addr: Address) -> Result<String> {
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

        let arch = self.arch()?;

        // length is always the first entry
        let length = self.virt_read_u16(addr + Length::from(0))?;
        if length == 0 {
            return Err(Error::new("unable to read unicode string length"));
        }

        // TODO: chek if length exceeds limit
        // buffer is either aligned at 4 or 8
        let buffer = match arch.instruction_set {
            InstructionSet::X64 => self.virt_read_addr64(addr + Length::from(8))?,
            InstructionSet::X86 => self.virt_read_addr32(addr + Length::from(4))?,
            _ => {
                return Err(Error::new("invalid proc_arch parameter"));
            }
        };
        if buffer.is_null() {
            return Err(Error::new("unable to read unicode string length"));
        }

        // check if buffer length is mod 2 (utf-16)
        if length % 2 != 0 {
            return Err(Error::new("unicode string length is not a multiple of two"));
        }

        // read buffer
        let mut content = self.virt_read(buffer, Length::from(length + 2))?;
        content[length as usize] = 0;
        content[length as usize + 1] = 0;

        // TODO: check length % 2 == 0

        let _content: Vec<u16> = unsafe { std::mem::transmute::<Vec<u8>, Vec<u16>>(content) };
        Ok(U16CString::from_vec_with_nul(_content)?.to_string_lossy())
    }
}
