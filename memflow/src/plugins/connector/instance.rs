use super::super::COptArc;
use super::{
    ArcPluginCpuState, ConnectorCpuStateFunctionTable, ConnectorFunctionTable, CpuState,
    MuArcPluginCpuState, MuPluginCpuState, PluginConnectorCpuState, PluginCpuState,
};
use crate::connector::cpu_state::ConnectorCpuStateInner;
use crate::error::*;
use crate::mem::{
    MemoryMap, PhysicalMemory, PhysicalMemoryMapping, PhysicalMemoryMetadata, PhysicalReadData,
    PhysicalWriteData,
};
use crate::types::Address;

use std::ffi::c_void;

use libloading::Library;

pub type OpaquePhysicalMemoryFunctionTable = PhysicalMemoryFunctionTable<c_void>;

impl Copy for OpaquePhysicalMemoryFunctionTable {}

impl Clone for OpaquePhysicalMemoryFunctionTable {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct PhysicalMemoryFunctionTable<T> {
    pub phys_read_raw_list: extern "C" fn(
        phys_mem: &mut T,
        read_data: *mut PhysicalReadData,
        read_data_count: usize,
    ) -> i32,
    pub phys_write_raw_list: extern "C" fn(
        phys_mem: &mut T,
        write_data: *const PhysicalWriteData,
        write_data_count: usize,
    ) -> i32,
    pub metadata: extern "C" fn(phys_mem: &T) -> PhysicalMemoryMetadata,
    pub set_mem_map: extern "C" fn(
        phys_mem: &mut T,
        mem_maps: *const PhysicalMemoryMapping,
        mem_maps_count: usize,
    ),
}

impl<T: PhysicalMemory> Default for &'static PhysicalMemoryFunctionTable<T> {
    fn default() -> Self {
        &PhysicalMemoryFunctionTable {
            phys_write_raw_list: c_phys_write_raw_list::<T>,
            phys_read_raw_list: c_phys_read_raw_list::<T>,
            metadata: c_metadata::<T>,
            set_mem_map: c_set_mem_map::<T>,
        }
    }
}

impl<T: PhysicalMemory> PhysicalMemoryFunctionTable<T> {
    pub fn as_opaque(&'static self) -> &'static OpaquePhysicalMemoryFunctionTable {
        unsafe { &*(self as *const Self as *const OpaquePhysicalMemoryFunctionTable) }
    }
}

extern "C" fn c_phys_write_raw_list<T: PhysicalMemory>(
    phys_mem: &mut T,
    write_data: *const PhysicalWriteData,
    write_data_count: usize,
) -> i32 {
    let write_data_slice = unsafe { std::slice::from_raw_parts(write_data, write_data_count) };
    phys_mem
        .phys_write_raw_list(write_data_slice)
        .into_int_result()
}

extern "C" fn c_phys_read_raw_list<T: PhysicalMemory>(
    phys_mem: &mut T,
    read_data: *mut PhysicalReadData,
    read_data_count: usize,
) -> i32 {
    let read_data_slice = unsafe { std::slice::from_raw_parts_mut(read_data, read_data_count) };
    phys_mem
        .phys_read_raw_list(read_data_slice)
        .into_int_result()
}

extern "C" fn c_metadata<T: PhysicalMemory>(phys_mem: &T) -> PhysicalMemoryMetadata {
    phys_mem.metadata()
}

extern "C" fn c_set_mem_map<T: PhysicalMemory>(
    phys_mem: &mut T,
    mem_maps: *const PhysicalMemoryMapping,
    mem_maps_count: usize,
) {
    let mem_maps_slice = unsafe { std::slice::from_raw_parts(mem_maps, mem_maps_count) };

    let mut mem_map = MemoryMap::new();
    mem_maps_slice.iter().for_each(|m| {
        mem_map.push_remap(m.base, m.size, m.real_base);
    });

    phys_mem.set_mem_map(mem_map)
}

/// Describes initialized connector instance
///
/// This structure is returned by `Connector`. It is needed to maintain reference
/// counts to the loaded connector library.
#[repr(C)]
pub struct ConnectorInstance {
    instance: &'static mut c_void,
    vtable: ConnectorFunctionTable,

    /// Internal library arc.
    ///
    /// This will keep the library loaded in memory as long as the connector instance is alive.
    /// This has to be the last member of the struct so the library will be unloaded _after_
    /// the instance is destroyed.
    ///
    /// If the library is unloaded prior to the instance this will lead to a SIGSEGV.
    pub(super) library: COptArc<Library>,
}

impl ConnectorInstance {
    pub fn builder<T: 'static + PhysicalMemory + Clone>(
        instance: T,
    ) -> ConnectorInstanceBuilder<T> {
        ConnectorInstanceBuilder {
            instance,
            vtable: ConnectorFunctionTable::create_vtable::<T>(),
        }
    }
}

/// Builder for the os instance structure.
pub struct ConnectorInstanceBuilder<T> {
    instance: T,
    vtable: ConnectorFunctionTable,
}

impl<T> ConnectorInstanceBuilder<T> {
    /// Enables the optional Keyboard feature for the OsInstance.
    pub fn enable_cpu_state<C>(mut self) -> Self
    where
        C: 'static + CpuState + Clone,
        T: PluginConnectorCpuState<C>,
    {
        self.vtable.cpu_state =
            Some(<&ConnectorCpuStateFunctionTable<C, T>>::default().as_opaque());
        self
    }

    /// Build the ConnectorInstance
    pub fn build(self) -> ConnectorInstance {
        ConnectorInstance {
            instance: unsafe {
                Box::into_raw(Box::new(self.instance))
                    .cast::<c_void>()
                    .as_mut()
            }
            .unwrap(),
            vtable: self.vtable,
            library: None.into(),
        }
    }
}

impl ConnectorInstance {
    pub fn has_cpu_state(&self) -> bool {
        self.vtable.cpu_state.is_some()
    }
}

impl PhysicalMemory for ConnectorInstance {
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()> {
        (self.vtable.phys.phys_read_raw_list)(self.instance, data.as_mut_ptr(), data.len());
        Ok(())
    }

    fn phys_write_raw_list(&mut self, data: &[PhysicalWriteData]) -> Result<()> {
        (self.vtable.phys.phys_write_raw_list)(self.instance, data.as_ptr(), data.len());
        Ok(())
    }

    fn metadata(&self) -> PhysicalMemoryMetadata {
        (self.vtable.phys.metadata)(self.instance)
    }

    fn set_mem_map(&mut self, mem_map: MemoryMap<(Address, usize)>) {
        let mem_maps_slice = mem_map
            .iter()
            .map(|m| PhysicalMemoryMapping {
                base: m.base(),
                size: m.output().1,
                real_base: m.output().0,
            })
            .collect::<Vec<_>>();

        (self.vtable.phys.set_mem_map)(self.instance, mem_maps_slice.as_ptr(), mem_maps_slice.len())
    }
}

/// Optional Cpu State feature implementation
impl<'a> ConnectorCpuStateInner<'a> for ConnectorInstance {
    type CpuStateType = PluginCpuState<'a>;
    type IntoCpuStateType = ArcPluginCpuState;

    fn cpu_state(&'a mut self) -> Result<Self::CpuStateType> {
        let cpu_state = self.vtable.cpu_state.ok_or(Error(
            ErrorOrigin::OsLayer,
            ErrorKind::UnsupportedOptionalFeature,
        ))?;
        let mut out = MuPluginCpuState::uninit();
        // Shorten the lifetime of instance
        let instance = unsafe { (self.instance as *mut c_void).as_mut() }.unwrap();
        let res = (cpu_state.cpu_state)(instance, self.library.clone(), &mut out);
        result_from_int(res, out)
    }

    fn into_cpu_state(mut self) -> Result<Self::IntoCpuStateType> {
        let cpu_state = self.vtable.cpu_state.ok_or(Error(
            ErrorOrigin::OsLayer,
            ErrorKind::UnsupportedOptionalFeature,
        ))?;
        let mut out = MuArcPluginCpuState::uninit();
        let res = (cpu_state.into_cpu_state)(self.instance, self.library.take(), &mut out);
        std::mem::forget(self);
        result_from_int(res, out)
    }
}

impl Clone for ConnectorInstance {
    fn clone(&self) -> Self {
        let instance =
            (self.vtable.base.clone.clone)(self.instance).expect("Unable to clone Connector");
        Self {
            instance,
            vtable: self.vtable,
            library: self.library.clone(),
        }
    }
}

impl Drop for ConnectorInstance {
    fn drop(&mut self) {
        unsafe {
            (self.vtable.base.drop)(self.instance);
        }
    }
}
