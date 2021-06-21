//! Describes optional cpu state for a connector

use crate::cglue::*;
use crate::prelude::v1::Result;

/// ConnectorCpuState supertrait for all possible lifetimes
///
/// Use this for convenience. Chances are, once GAT are implemented, only `ConnectorCpuState` will be kept.
///
/// It naturally provides all `ConnectorCpuStateInner` functions.
pub trait ConnectorCpuState: for<'a> ConnectorCpuStateInner<'a> {}
impl<T: for<'a> ConnectorCpuStateInner<'a>> ConnectorCpuState for T {}

#[cfg_attr(feature = "plugins", cglue_trait)]
#[cfg_attr(feature = "plugins", int_result)]
pub trait ConnectorCpuStateInner<'a>: Send {
    #[cfg_attr(feature = "plugins", wrap_with_obj(crate::connector::cpu_state::CpuState))]
    type CpuStateType: crate::connector::cpu_state::CpuState + 'a;
    #[cfg_attr(feature = "plugins", wrap_with_group(crate::connector::cpu_state::IntoCpuState))]
    type IntoCpuStateType: crate::connector::cpu_state::CpuState + 'static;

    fn cpu_state(&'a mut self) -> Result<Self::CpuStateType>;
    fn into_cpu_state(self) -> Result<Self::IntoCpuStateType>;
}

cglue_trait_group!(IntoCpuState, { CpuState, Clone }, {});

#[cfg_attr(feature = "plugins", cglue_trait)]
#[cfg_attr(feature = "plugins", int_result)]
#[cfg_attr(feature = "plugins", cglue_forward)]
pub trait CpuState {
    // TODO:
    // max cpu index
    // read_register(s)
    // write_register(s)
    // pause
    // resume
    // single-step
    // breakpoints

    fn pause(&mut self);
    fn resume(&mut self);
}
