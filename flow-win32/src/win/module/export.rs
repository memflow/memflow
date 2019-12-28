use flow_core::*;
use flow_core::address::Length;
use flow_core::process::ExportTrait;

// TODO: debug trait impl
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
    fn name(&self) -> String {
        self.name.clone()
    }

    fn offset(&self) -> Length {
        self.offset
    }
}
