use flow_core::address::Length;
use flow_core::process::ExportTrait;
use flow_core::*;

#[derive(Debug, Clone)]
pub struct Export {
    pub name: String,
    pub offset: Length,
    pub rva: Length,
    pub size: Length,
    // reexport
}

impl<'a> From<&goblin::pe::export::Export<'a>> for Export {
    fn from(e: &goblin::pe::export::Export<'a>) -> Self {
        Self {
            name: e.name.unwrap_or_default().to_owned(),
            offset: len!(e.offset),
            rva: len!(e.rva),
            size: len!(e.size),
        }
    }
}

impl ExportTrait for Export {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn offset(&self) -> Length {
        self.offset
    }
}
