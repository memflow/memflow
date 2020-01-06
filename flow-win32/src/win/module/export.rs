use flow_core::address::Length;
use flow_core::process::ExportTrait;
use flow_core::*;

#[derive(Debug, Clone)]
pub struct Export {
    pub name: String,
    pub offset: Length,
    //pub rva: Length,
    //pub size: Length,
    // reexport
}

impl Export {
    pub fn with(name: &str, offset: Length) -> Self {
        Self {
            name: name.to_owned(),
            offset,
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
