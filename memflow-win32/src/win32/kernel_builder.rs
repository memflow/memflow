use std::prelude::v1::*;

use super::{Kernel, KernelInfo};
use crate::error::Result;
use crate::offsets::Win32Offsets;

#[cfg(feature = "symstore")]
use crate::offsets::SymbolStore;

use memflow::architecture::ArchitectureObj;
use memflow::mem::{
    CachedMemoryAccess, CachedVirtualTranslate, DefaultCacheValidator, DirectTranslate,
    PhysicalMemory, VirtualTranslate,
};
use memflow::types::Address;

/// Builder for a Windows Kernel structure.
///
/// This function encapsulates the entire setup process for a Windows target
/// and will make sure the user gets a properly initialized object at the end.
///
/// This function is a high level abstraction over the individual parts of initialization a Windows target:
/// - Scanning for the ntoskrnl and retrieving the `KernelInfo` struct.
/// - Retrieving the Offsets for the target Windows version.
/// - Creating a struct which implements `VirtualTranslate` for virtual to physical address translations.
/// - Optionally wrapping the Connector or the `VirtualTranslate` object into a cached object.
/// - Initialization of the Kernel structure itself.
///
/// # Examples
///
/// Using the builder with default values:
/// ```
/// use memflow::mem::PhysicalMemory;
/// use memflow_win32::win32::Kernel;
///
/// fn test<T: PhysicalMemory>(connector: T) {
///     let _kernel = Kernel::builder(connector)
///         .build()
///         .unwrap();
/// }
/// ```
///
/// Using the builder with default cache configurations:
/// ```
/// use memflow::mem::PhysicalMemory;
/// use memflow_win32::win32::Kernel;
///
/// fn test<T: PhysicalMemory>(connector: T) {
///     let _kernel = Kernel::builder(connector)
///         .build_default_caches()
///         .build()
///         .unwrap();
/// }
/// ```
///
/// Customizing the caches:
/// ```
/// use memflow::mem::{PhysicalMemory, CachedMemoryAccess, CachedVirtualTranslate};
/// use memflow_win32::win32::Kernel;
///
/// fn test<T: PhysicalMemory>(connector: T) {
///     let _kernel = Kernel::builder(connector)
///     .build_page_cache(|connector, arch| {
///         CachedMemoryAccess::builder(connector)
///             .arch(arch)
///             .build()
///             .unwrap()
///     })
///     .build_vat_cache(|vat, arch| {
///         CachedVirtualTranslate::builder(vat)
///             .arch(arch)
///             .build()
///             .unwrap()
///     })
///     .build()
///     .unwrap();
/// }
/// ```
///
/// # Remarks
///
/// Manual initialization of the above examples would look like the following:
/// ```
/// use memflow::prelude::v1::*;
/// use memflow_win32::prelude::{KernelInfo, Win32Offsets, Kernel};
///
/// fn test<T: PhysicalMemory>(mut connector: T) {
///     // Use the ntoskrnl scanner to find the relevant KernelInfo (start_block, arch, dtb, ntoskrnl, etc)
///     let kernel_info = KernelInfo::scanner(&mut connector).scan().unwrap();
///     // Download the corresponding pdb from the default symbol store
///     let offsets = Win32Offsets::builder().kernel_info(&kernel_info).build().unwrap();
///
///     // Create a struct for doing virtual to physical memory translations
///     let vat = DirectTranslate::new();
///
///     // Create a Page Cache layer with default values
///     let mut connector_cached = CachedMemoryAccess::builder(connector)
///         .arch(kernel_info.start_block.arch)
///         .build()
///         .unwrap();
///
///     // Create a TLB Cache layer with default values
///     let vat_cached = CachedVirtualTranslate::builder(vat)
///         .arch(kernel_info.start_block.arch)
///         .build()
///         .unwrap();
///
///     // Initialize the final Kernel object
///     let _kernel = Kernel::new(&mut connector_cached, vat_cached, offsets, kernel_info);
/// }
/// ```
pub struct KernelBuilder<T, TK, VK> {
    connector: T,

    arch: Option<ArchitectureObj>,
    kernel_hint: Option<Address>,
    dtb: Option<Address>,

    #[cfg(feature = "symstore")]
    symbol_store: Option<SymbolStore>,

    build_page_cache: Box<dyn FnOnce(T, ArchitectureObj) -> TK>,
    build_vat_cache: Box<dyn FnOnce(DirectTranslate, ArchitectureObj) -> VK>,
}

impl<T> KernelBuilder<T, T, DirectTranslate>
where
    T: PhysicalMemory,
{
    pub fn new(connector: T) -> KernelBuilder<T, T, DirectTranslate> {
        KernelBuilder {
            connector,

            arch: None,
            kernel_hint: None,
            dtb: None,

            #[cfg(feature = "symstore")]
            symbol_store: Some(SymbolStore::default()),

            build_page_cache: Box::new(|connector, _| connector),
            build_vat_cache: Box::new(|vat, _| vat),
        }
    }
}

impl<'a, T, TK, VK> KernelBuilder<T, TK, VK>
where
    T: PhysicalMemory,
    TK: PhysicalMemory,
    VK: VirtualTranslate,
{
    pub fn build(mut self) -> Result<Kernel<TK, VK>> {
        // find kernel_info
        let mut kernel_scanner = KernelInfo::scanner(&mut self.connector);
        if let Some(arch) = self.arch {
            kernel_scanner = kernel_scanner.arch(arch);
        }
        if let Some(kernel_hint) = self.kernel_hint {
            kernel_scanner = kernel_scanner.kernel_hint(kernel_hint);
        }
        if let Some(dtb) = self.dtb {
            kernel_scanner = kernel_scanner.dtb(dtb);
        }
        let kernel_info = kernel_scanner.scan()?;

        // acquire offsets from the symbol store
        let offsets = self.build_offsets(&kernel_info)?;

        // create a vat object
        let vat = DirectTranslate::new();

        // create caches
        let kernel_connector =
            (self.build_page_cache)(self.connector, kernel_info.start_block.arch);
        let kernel_vat = (self.build_vat_cache)(vat, kernel_info.start_block.arch);

        // create the final kernel object
        Ok(Kernel::new(
            kernel_connector,
            kernel_vat,
            offsets,
            kernel_info,
        ))
    }

    #[cfg(feature = "symstore")]
    fn build_offsets(&self, kernel_info: &KernelInfo) -> Result<Win32Offsets> {
        let mut builder = Win32Offsets::builder();
        if let Some(store) = &self.symbol_store {
            builder = builder.symbol_store(store.clone());
        } else {
            builder = builder.no_symbol_store();
        }
        builder.kernel_info(kernel_info).build()
    }

    #[cfg(not(feature = "symstore"))]
    fn build_offsets(&self, kernel_info: &KernelInfo) -> Result<Win32Offsets> {
        Win32Offsets::builder().kernel_info(&kernel_info).build()
    }

    pub fn arch(mut self, arch: ArchitectureObj) -> Self {
        self.arch = Some(arch);
        self
    }

    pub fn kernel_hint(mut self, kernel_hint: Address) -> Self {
        self.kernel_hint = Some(kernel_hint);
        self
    }

    pub fn dtb(mut self, dtb: Address) -> Self {
        self.dtb = Some(dtb);
        self
    }

    /// Configures the symbol store to be used when constructing the Kernel.
    /// This will override the default symbol store that is being used if no other setting is configured.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::mem::PhysicalMemory;
    /// use memflow_win32::prelude::{Kernel, SymbolStore};
    ///
    /// fn test<T: PhysicalMemory>(connector: T) {
    ///     let _kernel = Kernel::builder(connector)
    ///         .symbol_store(SymbolStore::new().no_cache())
    ///         .build()
    ///         .unwrap();
    /// }
    /// ```
    #[cfg(feature = "symstore")]
    pub fn symbol_store(mut self, symbol_store: SymbolStore) -> Self {
        self.symbol_store = Some(symbol_store);
        self
    }

    /// Disables the symbol store when constructing the Kernel.
    /// By default a default symbol store will be used when constructing a kernel.
    /// This option allows the user to disable the symbol store alltogether
    /// and fall back to the built-in offsets table.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::mem::PhysicalMemory;
    /// use memflow_win32::win32::Kernel;
    /// use memflow_win32::offsets::SymbolStore;
    ///
    /// fn test<T: PhysicalMemory>(connector: T) {
    ///     let _kernel = Kernel::builder(connector)
    ///         .no_symbol_store()
    ///         .build()
    ///         .unwrap();
    /// }
    /// ```
    #[cfg(feature = "symstore")]
    pub fn no_symbol_store(mut self) -> Self {
        self.symbol_store = None;
        self
    }

    /// Creates the Kernel structure with default caching enabled.
    ///
    /// If this option is specified, the Kernel structure is generated
    /// with a (page level cache)[../index.html] with default settings.
    /// On top of the page level cache a [vat cache](../index.html) will be setupped.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::mem::PhysicalMemory;
    /// use memflow_win32::win32::Kernel;
    ///
    /// fn test<T: PhysicalMemory>(connector: T) {
    ///     let _kernel = Kernel::builder(connector)
    ///         .build_default_caches()
    ///         .build()
    ///         .unwrap();
    /// }
    /// ```
    pub fn build_default_caches(
        self,
    ) -> KernelBuilder<
        T,
        CachedMemoryAccess<'a, T, DefaultCacheValidator>,
        CachedVirtualTranslate<DirectTranslate, DefaultCacheValidator>,
    > {
        KernelBuilder {
            connector: self.connector,

            arch: self.arch,
            kernel_hint: self.kernel_hint,
            dtb: self.dtb,

            #[cfg(feature = "symstore")]
            symbol_store: self.symbol_store,

            build_page_cache: Box::new(|connector, arch| {
                CachedMemoryAccess::builder(connector)
                    .arch(arch)
                    .build()
                    .unwrap()
            }),
            build_vat_cache: Box::new(|vat, arch| {
                CachedVirtualTranslate::builder(vat)
                    .arch(arch)
                    .build()
                    .unwrap()
            }),
        }
    }

    /// Creates a Kernel structure by constructing the page cache from the given closure.
    ///
    /// This function accepts a `FnOnce` closure that is being evaluated
    /// after the ntoskrnl has been found.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::mem::{PhysicalMemory, CachedMemoryAccess};
    /// use memflow_win32::win32::Kernel;
    ///
    /// fn test<T: PhysicalMemory>(connector: T) {
    ///     let _kernel = Kernel::builder(connector)
    ///         .build_page_cache(|connector, arch| {
    ///             CachedMemoryAccess::builder(connector)
    ///                 .arch(arch)
    ///                 .build()
    ///                 .unwrap()
    ///         })
    ///         .build()
    ///         .unwrap();
    /// }
    /// ```
    pub fn build_page_cache<TKN, F: FnOnce(T, ArchitectureObj) -> TKN + 'static>(
        self,
        func: F,
    ) -> KernelBuilder<T, TKN, VK>
    where
        TKN: PhysicalMemory,
    {
        KernelBuilder {
            connector: self.connector,

            arch: self.arch,
            kernel_hint: self.kernel_hint,
            dtb: self.dtb,

            #[cfg(feature = "symstore")]
            symbol_store: self.symbol_store,

            build_page_cache: Box::new(func),
            build_vat_cache: self.build_vat_cache,
        }
    }

    /// Creates a Kernel structure by constructing the vat cache from the given closure.
    ///
    /// This function accepts a `FnOnce` closure that is being evaluated
    /// after the ntoskrnl has been found.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::mem::{PhysicalMemory, CachedVirtualTranslate};
    /// use memflow_win32::win32::Kernel;
    ///
    /// fn test<T: PhysicalMemory>(connector: T) {
    ///     let _kernel = Kernel::builder(connector)
    ///         .build_vat_cache(|vat, arch| {
    ///             CachedVirtualTranslate::builder(vat)
    ///                 .arch(arch)
    ///                 .build()
    ///                 .unwrap()
    ///         })
    ///         .build()
    ///         .unwrap();
    /// }
    /// ```
    pub fn build_vat_cache<VKN, F: FnOnce(DirectTranslate, ArchitectureObj) -> VKN + 'static>(
        self,
        func: F,
    ) -> KernelBuilder<T, TK, VKN>
    where
        VKN: VirtualTranslate,
    {
        KernelBuilder {
            connector: self.connector,

            arch: self.arch,
            kernel_hint: self.kernel_hint,
            dtb: self.dtb,

            #[cfg(feature = "symstore")]
            symbol_store: self.symbol_store,

            build_page_cache: self.build_page_cache,
            build_vat_cache: Box::new(func),
        }
    }

    // TODO: more builder configurations
    // kernel_info_builder()
    // offset_builder()
}
