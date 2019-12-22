// let kbd = Keyboard::with(mem, ...)
// let ks = kbd.keystate()
// ks.get(VK_A)
// kbd.set_keystate(...)

use crate::error::Result;
use crate::win::Windows;

use flow_core::mem::*;

use crate::win::process::*;

use crate::pe;

pub struct Keyboard {
    // kernel.read(key_state_addr)
//key_state_addr: Address,
}

pub struct KeyState {}

impl Keyboard {
    pub fn with<T: VirtualRead>(win: &Windows<T>) -> Result<Self> {
        let kernel_process = win.kernel_process()?;
        let mut kernel_module = kernel_process.module("win32kbase.sys")?;

        // TODO: helper with multiple tries
        let mut user_process = match win.process("winlogon.exe") {
            Ok(p) => p,
            Err(_) => win.process("wininit.exe")?,
        };

        // read a few mb at module base -> parse pe
        // -> find export

        // find_export("gafAsyncKeyState");

        let mut module = user_process.first_module()?;

        println!("user name: {:?}", module.name()?);
        println!("size: {:x}", kernel_module.size()?);
        println!("name: {:?}", kernel_module.name()?);
        println!("name: {:?}", user_process.name()?);

        let base = kernel_module.base()?;
        let size = kernel_module.size()?;

        println!("kernel_module: {:?}", kernel_module.name()?);

        //let memory = &mut win.mem.borrow_mut();
        let buf = user_process.virt_read(base, size)?;

        let export = pe::find_export(buf, "gafAsyncKeyState")?;

        println!("export: {:?}", export);

        Ok(Self {})
    }
}
