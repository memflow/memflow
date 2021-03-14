use crate::connector::*;
use crate::error::*;
use crate::mem::{PhysicalMemory, PhysicalMemoryMetadata, PhysicalReadData, PhysicalWriteData};

use super::{Args, CArc, COptArc, GenericBaseTable, Loadable, OpaqueBaseTable, PluginDescriptor};

use crate::types::ReprCStr;

use std::ffi::c_void;
use std::mem::MaybeUninit;

pub mod connectorcpustate;
pub use connectorcpustate::{
    ArcPluginCpuState, ConnectorCpuStateFunctionTable, OpaqueConnectorCpuStateFunctionTable,
    PluginCpuState,
};

use libloading::Library;

// Type aliases needed for &mut MaybeUninit<T> to work with bindgen
pub type MUPluginCpuState<'a> = MaybeUninit<PluginCpuState<'a>>;
pub type MUArcPluginCpuState = MaybeUninit<ArcPluginCpuState>;
pub type MUConnectorInstance = MaybeUninit<ConnectorInstance>;

/// Subtrait of Plugin where `Self`, and `OSKeyboard::IntoKeyboardType` are `Clone`
pub trait PluginConnectorCpuState<T: CpuState + Clone>:
    'static + Clone + for<'a> ConnectorCpuStateInner<'a, IntoCpuStateType = T>
{
}
impl<
        T: CpuState + Clone,
        K: 'static + Clone + for<'a> ConnectorCpuStateInner<'a, IntoCpuStateType = T>,
    > PluginConnectorCpuState<T> for K
{
}

pub fn create_with_logging<T: 'static + PhysicalMemory + Clone>(
    args: &ReprCStr,
    log_level: i32,
    out: &mut MUConnectorInstance,
    create_fn: impl Fn(&Args, log::Level) -> Result<T>,
) -> i32 {
    super::util::create_with_logging(args, log_level, out, |a, l| {
        Ok(create_fn(&a, l).map(ConnectorInstance::builder)?.build())
    })
}

pub fn create_without_logging<T: 'static + PhysicalMemory + Clone>(
    args: &ReprCStr,
    out: &mut MUConnectorInstance,
    create_fn: impl Fn(&Args) -> Result<T>,
) -> i32 {
    super::util::create_without_logging(args, out, |a| {
        Ok(create_fn(&a).map(ConnectorInstance::builder)?.build())
    })
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ConnectorFunctionTable {
    /// The vtable for object creation and cloning
    pub base: &'static OpaqueBaseTable,
    /// The vtable for all physical memory function calls to the connector.
    pub phys: &'static OpaquePhysicalMemoryFunctionTable,
    // The vtable for cpu state if available
    pub cpu_state: Option<&'static OpaqueConnectorCpuStateFunctionTable>,
}

impl ConnectorFunctionTable {
    pub fn create_vtable<T: 'static + PhysicalMemory + Clone>() -> Self {
        Self {
            base: <&GenericBaseTable<T>>::default().as_opaque(),
            phys: <&PhysicalMemoryFunctionTable<T>>::default().as_opaque(),
            cpu_state: None,
        }
    }
}

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
}

impl<T: PhysicalMemory> Default for &'static PhysicalMemoryFunctionTable<T> {
    fn default() -> Self {
        &PhysicalMemoryFunctionTable {
            phys_write_raw_list: c_phys_write_raw_list::<T>,
            phys_read_raw_list: c_phys_read_raw_list::<T>,
            metadata: c_metadata::<T>,
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
        .as_int_result()
}

extern "C" fn c_phys_read_raw_list<T: PhysicalMemory>(
    phys_mem: &mut T,
    read_data: *mut PhysicalReadData,
    read_data_count: usize,
) -> i32 {
    let read_data_slice = unsafe { std::slice::from_raw_parts_mut(read_data, read_data_count) };
    phys_mem.phys_read_raw_list(read_data_slice).as_int_result()
}

extern "C" fn c_metadata<T: PhysicalMemory>(phys_mem: &T) -> PhysicalMemoryMetadata {
    phys_mem.metadata()
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
    library: COptArc<Library>,
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
    /// Enables the optional Keyboard feature for the OSInstance.
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

pub type ConnectorDescriptor = PluginDescriptor<LoadableConnector>;

pub struct LoadableConnector {
    descriptor: PluginDescriptor<Self>,
}

impl Loadable for LoadableConnector {
    type Instance = ConnectorInstance;
    type InputArg = Option<&'static mut c_void>;
    type CInputArg = Option<&'static mut c_void>;

    fn ident(&self) -> &str {
        self.descriptor.name
    }

    fn export_prefix() -> &'static str {
        "MEMFLOW_CONNECTOR_"
    }

    fn plugin_type() -> &'static str {
        "Connector"
    }

    fn new(descriptor: PluginDescriptor<Self>) -> Self {
        Self { descriptor }
    }

    /// Creates a new connector instance from this library.
    ///
    /// The connector is initialized with the arguments provided to this function.
    fn instantiate(
        &self,
        library: Option<CArc<Library>>,
        input: Self::InputArg,
        args: &Args,
    ) -> Result<ConnectorInstance> {
        let cstr = ReprCStr::from(args.to_string());
        let mut out = MUConnectorInstance::uninit();
        let res = (self.descriptor.create)(&cstr, input, log::max_level() as i32, &mut out);
        result_from_int(res, out).map(|mut c| {
            c.library = library.into();
            c
        })
    }
}
