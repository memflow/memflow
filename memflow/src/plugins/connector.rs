use crate::error::*;
use crate::mem::{PhysicalMemory, PhysicalMemoryMetadata, PhysicalReadData, PhysicalWriteData};

use super::{Args, CArc, COptArc, GenericBaseTable, Loadable, OpaqueBaseTable, PluginDescriptor};

use crate::types::ReprCStr;

use std::ffi::c_void;
use std::mem::MaybeUninit;

use libloading::Library;

pub type MUConnectorInstance = MaybeUninit<ConnectorInstance>;

pub fn create_with_logging<T: 'static + PhysicalMemory + Clone>(
    args: &ReprCStr,
    log_level: i32,
    out: &mut MUConnectorInstance,
    create_fn: impl Fn(&Args, log::Level) -> Result<T>,
) -> i32 {
    super::util::create_with_logging(args, log_level, out, |a, l| {
        create_fn(&a, l).map(ConnectorInstance::new)
    })
}

pub fn create_without_logging<T: 'static + PhysicalMemory + Clone>(
    args: &ReprCStr,
    out: &mut MUConnectorInstance,
    create_fn: impl Fn(&Args) -> Result<T>,
) -> i32 {
    super::util::create_without_logging(args, out, |a| create_fn(&a).map(ConnectorInstance::new))
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ConnectorFunctionTable {
    /// The vtable for object creation and cloning
    pub base: OpaqueBaseTable,
    /// The vtable for all physical memory function calls to the connector.
    pub phys: OpaquePhysicalMemoryFunctionTable,
    // further optional table expansion with Option<&'static SomeFunctionTable>
    // ...
}

impl ConnectorFunctionTable {
    pub fn create_vtable<T: 'static + PhysicalMemory + Clone>() -> Self {
        Self {
            base: GenericBaseTable::<T>::default().into_opaque(),
            phys: PhysicalMemoryFunctionTable::<T>::default().into_opaque(),
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

impl<T: PhysicalMemory> Default for PhysicalMemoryFunctionTable<T> {
    fn default() -> Self {
        Self {
            phys_write_raw_list: phys_write_raw_list_internal::<T>,
            phys_read_raw_list: phys_read_raw_list_internal::<T>,
            metadata: metadata_internal::<T>,
        }
    }
}

impl<T: PhysicalMemory> PhysicalMemoryFunctionTable<T> {
    pub fn into_opaque(self) -> OpaquePhysicalMemoryFunctionTable {
        unsafe { std::mem::transmute(self) }
    }
}

extern "C" fn phys_write_raw_list_internal<T: PhysicalMemory>(
    phys_mem: &mut T,
    write_data: *const PhysicalWriteData,
    write_data_count: usize,
) -> i32 {
    let write_data_slice = unsafe { std::slice::from_raw_parts(write_data, write_data_count) };
    phys_mem
        .phys_write_raw_list(write_data_slice)
        .as_int_result()
}

extern "C" fn phys_read_raw_list_internal<T: PhysicalMemory>(
    phys_mem: &mut T,
    read_data: *mut PhysicalReadData,
    read_data_count: usize,
) -> i32 {
    let read_data_slice = unsafe { std::slice::from_raw_parts_mut(read_data, read_data_count) };
    phys_mem.phys_read_raw_list(read_data_slice).as_int_result()
}

extern "C" fn metadata_internal<T: PhysicalMemory>(phys_mem: &T) -> PhysicalMemoryMetadata {
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
    pub fn new<T: 'static + PhysicalMemory + Clone>(mem: T) -> Self {
        Self {
            instance: unsafe { Box::into_raw(Box::new(mem)).cast::<c_void>().as_mut() }.unwrap(),
            vtable: ConnectorFunctionTable::create_vtable::<T>(),
            library: None.into(),
        }
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
