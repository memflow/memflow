//! Helpers for implementing several OS functions.

use crate::error::*;
use crate::mem::MemoryView;
use crate::os::*;
use crate::types::umem;
use cglue::prelude::v1::ReprCString;

#[cfg(feature = "goblin")]
use goblin::{
    container::Ctx,
    elf::{dynamic, Dynamic, Elf, ProgramHeader, RelocSection, Symtab},
    mach::Mach,
    pe::{options::ParseOptions, PE},
    strtab::Strtab,
    Object,
};

#[cfg(feature = "goblin")]
fn parse_elf<'a>(bytes: &'a [u8]) -> goblin::error::Result<Elf<'a>> {
    let header = Elf::parse_header(bytes)?;

    let ctx = Ctx {
        container: header.container()?,
        le: header.endianness()?,
    };

    let program_headers =
        ProgramHeader::parse(bytes, header.e_phoff as usize, header.e_phnum as usize, ctx)?;

    let dynamic = Dynamic::parse(bytes, &program_headers, ctx)?;

    let mut dynsyms = Symtab::default();
    let mut dynstrtab = Strtab::default();
    let mut dynrelas = RelocSection::default();
    let mut dynrels = RelocSection::default();
    let mut pltrelocs = RelocSection::default();

    if let Some(ref dynamic) = dynamic {
        let dyn_info = &dynamic.info;

        dynstrtab = Strtab::parse(bytes, dyn_info.strtab, dyn_info.strsz, 0x0)?;

        /*if dyn_info.soname != 0 {
            // FIXME: warn! here
            soname = dynstrtab.get_at(dyn_info.soname);
        }
        if dyn_info.needed_count > 0 {
            libraries = dynamic.get_libraries(&dynstrtab);
        }*/
        // parse the dynamic relocations
        if let Ok(relas) = RelocSection::parse(bytes, dyn_info.rela, dyn_info.relasz, true, ctx) {
            dynrelas = relas;
            dynrels = RelocSection::parse(bytes, dyn_info.rel, dyn_info.relsz, false, ctx)?;
            let is_rela = dyn_info.pltrel as u64 == dynamic::DT_RELA;
            pltrelocs =
                RelocSection::parse(bytes, dyn_info.jmprel, dyn_info.pltrelsz, is_rela, ctx)?;

            // TODO: support these from goblin
            let mut num_syms = /*if let Some(gnu_hash) = dyn_info.gnu_hash {
                gnu_hash_len(bytes, gnu_hash as usize, ctx)?
            } else if let Some(hash) = dyn_info.hash {
                hash_len(bytes, hash as usize, header.e_machine, ctx)?
            } else*/ {
                0
            };
            let max_reloc_sym = dynrelas
                .iter()
                .chain(dynrels.iter())
                .chain(pltrelocs.iter())
                .fold(0, |num, reloc| core::cmp::max(num, reloc.r_sym));
            if max_reloc_sym != 0 {
                num_syms = core::cmp::max(num_syms, max_reloc_sym + 1);
            }

            dynsyms = Symtab::parse(bytes, dyn_info.symtab, num_syms, ctx)?;
        }
    }

    let mut elf = Elf::lazy_parse(header)?;

    elf.program_headers = program_headers;
    elf.dynamic = dynamic;
    elf.dynsyms = dynsyms;
    elf.dynstrtab = dynstrtab;
    elf.dynrelas = dynrelas;
    elf.dynrels = dynrels;
    elf.pltrelocs = pltrelocs;

    Ok(elf)
}

#[cfg(feature = "goblin")]
fn custom_parse<'a>(buf: &'a [u8]) -> Result<Object<'a>> {
    PE::parse_with_opts(buf, &ParseOptions { resolve_rva: false })
        .map(|pe| Object::PE(pe))
        .or_else(|_| parse_elf(buf).map(|elf| Object::Elf(elf)))
        .or_else(|_| Mach::parse(buf).map(|mach| Object::Mach(mach)))
        .map_err(|_| Error(ErrorOrigin::OsLayer, ErrorKind::InvalidExeFile))
}

#[cfg(feature = "goblin")]
pub fn module_import_list_callback(
    mem: &mut impl MemoryView,
    info: &ModuleInfo,
    callback: ImportCallback,
) -> Result<()> {
    let mut module_image = vec![0u8; info.size as usize];

    mem.read_raw_into(info.base, &mut module_image)
        .data_part()?;

    fn import_call(iter: impl Iterator<Item = (umem, ReprCString)>, mut callback: ImportCallback) {
        iter.take_while(|(offset, name)| {
            callback.call(ImportInfo {
                name: name.clone(),
                offset: *offset,
            })
        })
        .for_each(|_| {});
    }

    match custom_parse(&module_image)? {
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

    fn export_call(iter: impl Iterator<Item = (umem, ReprCString)>, mut callback: ExportCallback) {
        iter.take_while(|(offset, name)| {
            callback.call(ExportInfo {
                name: name.clone(),
                offset: *offset,
            })
        })
        .for_each(|_| {});
    }

    match custom_parse(&module_image)? {
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

    match custom_parse(&module_image)? {
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
