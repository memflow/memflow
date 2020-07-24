use log::Level;

use memflow_core::connector::ConnectorArgs;
use memflow_core::*;
use memflow_win32::*;

use memflow_coredump::create_connector;

fn main() {
    simple_logger::init_with_level(Level::Debug).unwrap();

    let mut connector = create_connector(&ConnectorArgs::with_default("./coredump.raw")).unwrap();

    let kernel_info = KernelInfo::scanner(&mut connector).scan().unwrap();

    let vat = TranslateArch::new(kernel_info.start_block.arch);
    let offsets = Win32Offsets::try_with_kernel_info(&kernel_info).unwrap();

    Kernel::new(connector, vat, offsets, kernel_info);
}
