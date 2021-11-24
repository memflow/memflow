use std::prelude::v1::*;

use crate::offsets::Win32ArchOffsets;
use crate::win32::VirtualReadUnicodeString;

use log::trace;

use memflow::architecture::ArchitectureIdent;
use memflow::error::Result;
use memflow::mem::MemoryView;
use memflow::os::{AddressCallback, ModuleInfo};
use memflow::types::Address;

const MAX_ITER_COUNT: usize = 65536;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct Win32ModuleListInfo {
    module_base: Address,
    offsets: Win32ArchOffsets,
}

impl Win32ModuleListInfo {
    pub fn with_peb(
        mem: &mut impl MemoryView,
        env_block: Address,
        arch: ArchitectureIdent,
    ) -> Result<Self> {
        let offsets = Win32ArchOffsets::from(arch);
        let arch_obj = arch.into();

        trace!("peb_ldr_offs={:x}", offsets.peb_ldr);
        trace!("ldr_list_offs={:x}", offsets.ldr_list);

        let env_block_ldr = mem.read_addr_arch(arch_obj, env_block + offsets.peb_ldr)?;
        trace!("peb_ldr={:x}", env_block_ldr);

        let module_base = mem.read_addr_arch(arch_obj, env_block_ldr + offsets.ldr_list)?;

        Self::with_base(module_base, arch)
    }

    pub fn with_base(module_base: Address, arch: ArchitectureIdent) -> Result<Self> {
        trace!("module_base={:x}", module_base);

        let offsets = Win32ArchOffsets::from(arch);
        trace!("offsets={:?}", offsets);

        Ok(Win32ModuleListInfo {
            module_base,
            offsets,
        })
    }

    pub fn module_base(&self) -> Address {
        self.module_base
    }

    pub fn module_entry_list<V: MemoryView>(
        &self,
        mem: &mut impl AsMut<V>,
        arch: ArchitectureIdent,
    ) -> Result<Vec<Address>> {
        let mut out = vec![];
        self.module_entry_list_callback(mem, arch, (&mut out).into())?;
        Ok(out)
    }

    pub fn module_entry_list_callback<M: AsMut<V>, V: MemoryView>(
        &self,
        mem: &mut M,
        arch: ArchitectureIdent,
        mut callback: AddressCallback,
    ) -> Result<()> {
        let list_start = self.module_base;
        let mut list_entry = list_start;
        let arch_obj = arch.into();
        for _ in 0..MAX_ITER_COUNT {
            if !callback.call(list_entry) {
                break;
            }
            list_entry = mem.as_mut().read_addr_arch(arch_obj, list_entry)?;
            // Break on misaligned entry. On NT 4.0 list end is misaligned, maybe it's a flag?
            if list_entry.is_null()
                || (list_entry.to_umem() & 0b111) != 0
                || list_entry == self.module_base
            {
                break;
            }
        }

        Ok(())
    }

    pub fn module_base_from_entry(
        &self,
        entry: Address,
        mem: &mut impl MemoryView,
        arch: ArchitectureIdent,
    ) -> Result<Address> {
        mem.read_addr_arch(arch.into(), entry + self.offsets.ldr_data_base)
            .map_err(From::from)
    }

    pub fn module_info_from_entry(
        &self,
        entry: Address,
        parent_eprocess: Address,
        mem: &mut impl MemoryView,
        arch: ArchitectureIdent,
    ) -> Result<ModuleInfo> {
        let base = self.module_base_from_entry(entry, mem, arch)?;
        let arch_obj = arch.into();

        trace!("base={:x}", base);

        let size = mem
            .read_addr_arch(arch_obj, entry + self.offsets.ldr_data_size)?
            .to_umem();

        trace!("size={:x}", size);

        let path = mem
            .read_unicode_string(arch_obj, entry + self.offsets.ldr_data_full_name)
            .unwrap_or_else(|_| String::new());
        trace!("path={}", path);

        let name = mem
            .read_unicode_string(arch_obj, entry + self.offsets.ldr_data_base_name)
            .unwrap_or_else(|_| String::new());
        trace!("name={}", name);

        Ok(ModuleInfo {
            address: entry,
            parent_process: parent_eprocess,
            base,
            size,
            path: path.into(),
            name: name.into(),
            arch,
        })
    }
}
