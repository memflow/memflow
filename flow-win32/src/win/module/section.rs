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
    pub virtual_address: Address,
    pub virtual_size: Length,
    pub size_of_raw_data: Length,
    pub characteristics: u32,
}

impl From<&goblin::pe::section_table::SectionTable> for Section {
    fn from(s: &goblin::pe::section_table::SectionTable) -> Self {
        Self {
            name: String::from_utf8(s.name.to_vec()).unwrap_or_default(),
            virtual_address: addr!(s.virtual_address),
            virtual_size: len!(s.virtual_size),
            size_of_raw_data: len!(s.size_of_raw_data),
            characteristics: s.characteristics,
        }
    }
}

/*
impl ExportTrait for Export {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn offset(&self) -> Length {
        self.offset
    }
}
*/
