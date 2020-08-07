pub mod lowstub;
pub mod ntos;
pub mod sysproc;

pub use lowstub::StartBlock;

#[derive(Debug, Clone)]
pub struct Win32GUID {
    pub file_name: String,
    pub guid: String,
}

impl Win32GUID {
    pub fn new(file_name: &str, guid: &str) -> Self {
        Self {
            file_name: file_name.to_string(),
            guid: guid.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Win32BuildNumber(u32);

impl Win32BuildNumber {
    pub fn new(nt_build_number: u32) -> Self {
        Self { 0: nt_build_number }
    }

    pub fn build_number(&self) -> u32 {
        self.0 & 0xFFFF
    }

    pub fn is_checked_build(&self) -> bool {
        (self.0 & 0xF0000000) == 0xC0000000
    }
}
