use memflow::types::Address;
use memflow_win32::kernel;

#[repr(C)]
pub struct StartBlock {
    pub kernel_hint: Address,
    pub dtb: Address,
}

impl From<kernel::StartBlock> for StartBlock {
    fn from(o: kernel::StartBlock) -> StartBlock {
        StartBlock {
            kernel_hint: o.kernel_hint,
            dtb: o.dtb,
        }
    }
}
