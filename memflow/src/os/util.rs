//! Helpers for implementing several OS functions.

use crate::error::*;
use crate::mem::MemoryView;
use crate::os::*;
use crate::types::umem;
use cglue::prelude::v1::ReprCString;

#[cfg(feature = "goblin")]
pub fn module_import_list_callback(
    mem: &mut impl MemoryView,
    info: &ModuleInfo,
    callback: ImportCallback,
) -> Result<()> {
    let mut module_image = vec![0u8; info.size as usize];

    mem.read_raw_into(info.base, &mut module_image)
        .data_part()?;

    use goblin::Object;

    fn import_call(iter: impl Iterator<Item = (umem, ReprCString)>, mut callback: ImportCallback) {
        iter.take_while(|(offset, name)| {
            callback.call(ImportInfo {
                name: name.clone(),
                offset: *offset,
            })
        })
        .for_each(|_| {});
    }

    match Object::parse(&module_image).map_err(|_| ErrorKind::InvalidExeFile)? {
        Object::Elf(elf) => {
            let iter = elf
                .dynsyms
                .iter()
                .filter(|s| s.is_import())
                .filter_map(|s| {
                    elf.dynstrtab
                        .get_at(s.st_name)
                        .map(|n| (s.st_value as umem, ReprCString::from(n)))
                });

            import_call(iter, callback);

            Ok(())
        }
        Object::PE(pe) => {
            let iter = pe
                .imports
                .iter()
                .map(|e| (e.offset as umem, e.name.as_ref().into()));

            import_call(iter, callback);

            Ok(())
        }
        _ => Err(ErrorKind::InvalidExeFile.into()),
    }
}

#[cfg(feature = "goblin")]
pub fn module_export_list_callback(
    mem: &mut impl MemoryView,
    info: &ModuleInfo,
    callback: ExportCallback,
) -> Result<()> {
    let mut module_image = vec![0u8; info.size as usize];

    mem.read_raw_into(info.base, &mut module_image)
        .data_part()?;

    use goblin::Object;

    fn export_call(iter: impl Iterator<Item = (umem, ReprCString)>, mut callback: ExportCallback) {
        iter.take_while(|(offset, name)| {
            callback.call(ExportInfo {
                name: name.clone(),
                offset: *offset,
            })
        })
        .for_each(|_| {});
    }

    match Object::parse(&module_image).map_err(|_| ErrorKind::InvalidExeFile)? {
        Object::Elf(elf) => {
            let iter = elf
                .dynsyms
                .iter()
                .filter(|s| !s.is_import())
                .filter_map(|s| {
                    elf.dynstrtab
                        .get_at(s.st_name)
                        .map(|n| (s.st_value as umem, ReprCString::from(n)))
                });

            export_call(iter, callback);

            Ok(())
        }
        Object::PE(pe) => {
            let iter = pe
                .exports
                .iter()
                .filter_map(|e| e.name.map(|name| (e.offset as umem, name.into())));

            export_call(iter, callback);

            Ok(())
        }
        _ => Err(ErrorKind::InvalidExeFile.into()),
    }
}

#[cfg(feature = "goblin")]
pub fn module_section_list_callback(
    mem: &mut impl MemoryView,
    info: &ModuleInfo,
    callback: SectionCallback,
) -> Result<()> {
    let mut module_image = vec![0u8; info.size as usize];

    mem.read_raw_into(info.base, &mut module_image)
        .data_part()?;

    use goblin::Object;

    fn section_call(
        iter: impl Iterator<Item = (umem, umem, ReprCString)>,
        mut callback: SectionCallback,
    ) {
        iter.take_while(|(base, size, name)| {
            callback.call(SectionInfo {
                name: name.clone(),
                base: Address::from(*base),
                size: *size,
            })
        })
        .for_each(|_| {});
    }

    match Object::parse(&module_image).map_err(|_| ErrorKind::InvalidExeFile)? {
        Object::Elf(elf) => {
            let iter = elf.section_headers.iter().filter_map(|s| {
                elf.shdr_strtab
                    .get_at(s.sh_name)
                    .map(|n| (s.sh_addr as umem, s.sh_size as umem, ReprCString::from(n)))
            });

            section_call(iter, callback);

            Ok(())
        }
        Object::PE(pe) => {
            let iter = pe.sections.iter().filter_map(|e| {
                e.real_name.as_ref().map(|name| {
                    (
                        e.virtual_address as umem,
                        e.virtual_size as umem,
                        name.as_str().into(),
                    )
                })
            });

            section_call(iter, callback);

            Ok(())
        }
        _ => Err(ErrorKind::InvalidExeFile.into()),
    }
}
