use std::io::Result;

// TODO: implement different traits or whatever for diff archs
#[derive(Debug, Eq, PartialEq)]
pub enum Architecture {
    X64,
    X86Pae,
    X86
}

pub struct CPU {
    pub arch: Option<Architecture>,
}
