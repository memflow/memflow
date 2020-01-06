/*pub struct SectionTable {
    pub name: [u8; 8],
    pub real_name: Option<String>,
    pub virtual_size: u32,
    pub virtual_address: u32,
    pub size_of_raw_data: u32,
    pub pointer_to_raw_data: u32,
    pub pointer_to_relocations: u32,
    pub pointer_to_linenumbers: u32,
    pub number_of_relocations: u16,
    pub number_of_linenumbers: u16,
    pub characteristics: u32,
}*/

use flow_core::address::Length;
//use flow_core::process::ExportTrait;
use flow_core::*;

#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub virt_addr: Address,
    pub virt_size: Length,
    pub size_of_raw_data: Length,
    pub characteristics: u32,
}

// TODO: with

impl SectionTrait for Section {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn virt_addr(&self) -> Address {
        self.virt_addr
    }

    fn virt_size(&self) -> Length {
        self.virt_size
    }
}
