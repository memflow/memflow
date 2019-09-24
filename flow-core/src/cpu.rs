use std::io::Result;

// TODO: implement different traits or whatever for diff archs
pub enum Architecture {
    X64,
    X64Pae,
    X86
}

pub struct CPU {
    pub arch: Option<Architecture>,
}
