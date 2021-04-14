//! Describes optional cpu state for a connector

use crate::prelude::v1::Result;

/// ConnectorCpuState supertrait for all possible lifetimes
///
/// Use this for convenience. Chances are, once GAT are implemented, only `ConnectorCpuState` will be kept.
///
/// It naturally provides all `ConnectorCpuStateInner` functions.
pub trait ConnectorCpuState: for<'a> ConnectorCpuStateInner<'a> {}
impl<T: for<'a> ConnectorCpuStateInner<'a>> ConnectorCpuState for T {}

pub trait ConnectorCpuStateInner<'a>: Send {
    type CpuStateType: CpuState + 'a;
    type IntoCpuStateType: CpuState;

    fn cpu_state(&'a mut self) -> Result<Self::CpuStateType>;
    fn into_cpu_state(self) -> Result<Self::IntoCpuStateType>;
}

pub trait CpuState {
    // TODO:
    // max cpu index
    // read_register(s)
    // write_register(s)
    // pause
    // resume
    // single-step
    // breakpoints
}
