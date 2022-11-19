//! Describes optional cpu state for a connector

use crate::cglue::*;
use crate::prelude::v1::Result;

#[cfg_attr(feature = "plugins", cglue_trait)]
#[int_result]
pub trait ConnectorCpuState: Send {
    #[wrap_with_obj(crate::connector::cpu_state::CpuState)]
    type CpuStateType<'a>: crate::connector::cpu_state::CpuState + 'a
    where
        Self: 'a;
    #[wrap_with_group(crate::connector::cpu_state::IntoCpuState)]
    type IntoCpuStateType: crate::connector::cpu_state::CpuState + 'static;

    fn cpu_state(&mut self) -> Result<Self::CpuStateType<'_>>;
    fn into_cpu_state(self) -> Result<Self::IntoCpuStateType>;
}

#[cfg(feature = "plugins")]
cglue_trait_group!(IntoCpuState, { CpuState, Clone }, {});

#[cfg_attr(feature = "plugins", cglue_trait)]
#[int_result]
#[cglue_forward]
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
