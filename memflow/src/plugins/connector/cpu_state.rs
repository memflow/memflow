pub mod plugin;
pub use plugin::{ArcPluginCpuState, PluginCpuState};

use crate::error::*;

use crate::connector::{ConnectorCpuStateInner, CpuState};
use std::ffi::c_void;

use super::super::COptArc;
use super::PluginConnectorCpuState;
use super::{MuArcPluginCpuState, MuPluginCpuState};

use libloading::Library;

pub type OpaqueConnectorCpuStateFunctionTable =
    ConnectorCpuStateFunctionTable<'static, c_void, c_void>;

impl Copy for OpaqueConnectorCpuStateFunctionTable {}

impl Clone for OpaqueConnectorCpuStateFunctionTable {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct ConnectorCpuStateFunctionTable<'a, K, T> {
    pub cpu_state:
        extern "C" fn(os: &'a mut T, lib: COptArc<Library>, out: &mut MuPluginCpuState<'a>) -> i32,
    pub into_cpu_state:
        extern "C" fn(os: &mut T, lib: COptArc<Library>, out: &mut MuArcPluginCpuState) -> i32,
    phantom: std::marker::PhantomData<K>,
}

impl<'a, K: 'static + CpuState + Clone, T: PluginConnectorCpuState<K>> Default
    for &'a ConnectorCpuStateFunctionTable<'a, K, T>
{
    fn default() -> Self {
        &ConnectorCpuStateFunctionTable {
            cpu_state: c_cpu_state,
            into_cpu_state: c_into_cpu_state,
            phantom: std::marker::PhantomData {},
        }
    }
}

impl<'a, P: 'static + CpuState + Clone, T: PluginConnectorCpuState<P>>
    ConnectorCpuStateFunctionTable<'a, P, T>
{
    pub fn as_opaque(&self) -> &OpaqueConnectorCpuStateFunctionTable {
        unsafe { &*(self as *const Self as *const OpaqueConnectorCpuStateFunctionTable) }
    }
}

extern "C" fn c_cpu_state<'a, T: 'a + ConnectorCpuStateInner<'a>>(
    connector: &'a mut T,
    lib: COptArc<Library>,
    out: &mut MuPluginCpuState<'a>,
) -> i32 {
    connector
        .cpu_state()
        .map(|k| PluginCpuState::new(k, lib))
        .into_int_out_result(out)
}

extern "C" fn c_into_cpu_state<
    P: 'static + CpuState + Clone,
    T: 'static + PluginConnectorCpuState<P>,
>(
    connector: &mut T,
    lib: COptArc<Library>,
    out: &mut MuArcPluginCpuState,
) -> i32 {
    let connector = unsafe { Box::from_raw(connector) };
    connector
        .into_cpu_state()
        .map(|k| ArcPluginCpuState::new(k, lib))
        .into_int_out_result(out)
}
