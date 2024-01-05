//! Helpers for implementing several OS functions.

use crate::error::*;
use crate::mem::MemoryView;
use crate::os::*;
use crate::types::umem;
use cglue::prelude::v1::ReprCString;
use dataview::PodMethods;
use std::vec::Vec;

#[cfg(feature = "goblin")]
use goblin::{
    container::Ctx,
    elf::{dynamic, Dynamic, Elf, ProgramHeader, RelocSection, Symtab},
    mach::{exports::ExportInfo as MachExportInfo, Mach, MachO},
    pe::{options::ParseOptions, PE},
    strtab::Strtab,
    Object,
};

fn aligned_alloc(bytes: usize) -> Vec<u64> {
    vec![0; (bytes + 8 - 1) / 8]
}

#[cfg(feature = "goblin")]
fn parse_elf(bytes: &[u8]) -> goblin::error::Result<Elf<'_>> {
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
fn custom_parse(buf: &[u8]) -> Result<Object<'_>> {
    PE::parse_with_opts(
        buf,
        &ParseOptions {
            resolve_rva: false,
            parse_attribute_certificates: false,
        },
    )
    .map(Object::PE)
    .map_err(|e| {
        log::debug!("PE: {}", e);
        e
    })
    .or_else(|_| parse_elf(buf).map(Object::Elf))
    .map_err(|e| {
        log::debug!("Elf: {}", e);
        e
    })
    .or_else(|_| {
        // Until https://github.com/m4b/goblin/pull/386 is merged
        #[cfg(feature = "unstable_goblin_lossy_macho")]
        return Mach::parse_2(buf, true).map(Object::Mach);
        #[cfg(not(feature = "unstable_goblin_lossy_macho"))]
        return Mach::parse(buf).map(Object::Mach);
    })
    .map_err(|e| {
        log::debug!("Mach: {}", e);
        e
    })
    .map_err(|_| Error(ErrorOrigin::OsLayer, ErrorKind::InvalidExeFile))
}

#[cfg(feature = "goblin")]
fn macho_base(bin: &MachO) -> Option<umem> {
    let s = bin.segments.sections().flatten().next()?.ok()?.0;
    Some(s.addr as umem)
}

#[inline]
pub fn module_import_list_callback(
    mem: &mut impl MemoryView,
    info: &ModuleInfo,
    callback: ImportCallback,
) -> Result<()> {
    import_list_callback(mem, info.base, info.size, callback)
}

pub fn import_list_callback(
    mem: &mut impl MemoryView,
    base: Address,
    size: umem,
    mut callback: ImportCallback,
) -> Result<()> {
    let mut module_image = aligned_alloc(size as usize);
    let module_image = module_image.as_bytes_mut();

    mem.read_raw_into(base, module_image).data_part()?;

    fn import_call(iter: impl Iterator<Item = (umem, ReprCString)>, callback: &mut ImportCallback) {
        iter.take_while(|(offset, name)| {
            callback.call(ImportInfo {
                name: name.clone(),
                offset: *offset,
            })
        })
        .for_each(|_| {});
    }

    let ret = Err(Error::from(ErrorKind::NotImplemented));

    #[cfg(feature = "pelite")]
    let ret = ret.or_else(|_| {
        if let Ok(pe) = pelite::PeView::from_bytes(module_image) {
            use pelite::pe32::imports::Import as Import32;
            use pelite::pe64::imports::Import as Import64;
            use pelite::Wrap::*;

            if let Some(imports) = pe
                .iat()
                .map(Some)
                .or_else(|e| {
                    if let pelite::Error::Null = e {
                        Ok(None)
                    } else {
                        Err(e)
                    }
                })
                .map_err(|_| ErrorKind::InvalidExeFile)?
            {
                let iter = imports
                    .iter()
                    .filter_map(|w| match w {
                        T32((addr, Ok(Import32::ByName { name, .. }))) => {
                            Some((*addr as umem, name))
                        }
                        T64((addr, Ok(Import64::ByName { name, .. }))) => {
                            Some((*addr as umem, name))
                        }
                        _ => None,
                    })
                    .filter_map(|(a, n)| n.to_str().ok().map(|n| (a, n.into())));

                import_call(iter, &mut callback);
            }

            Ok(())
        } else {
            Err(Error::from(ErrorKind::InvalidExeFile))
        }
    });

    #[cfg(feature = "goblin")]
    let ret = ret.or_else(|_| match custom_parse(module_image)? {
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

            import_call(iter, &mut callback);

            Ok(())
        }
        Object::PE(pe) => {
            let iter = pe
                .imports
                .iter()
                .map(|e| (e.offset as umem, e.name.as_ref().into()));

            import_call(iter, &mut callback);

            Ok(())
        }
        Object::Mach(Mach::Binary(bin)) => {
            let mbase = macho_base(&bin).unwrap_or_default();

            let iter = bin
                .imports()
                .ok()
                .into_iter()
                .flatten()
                .map(|v| ((v.address as umem) - mbase + base.to_umem(), v.name.into()));

            import_call(iter, &mut callback);

            Ok(())
        }
        _ => Err(ErrorKind::InvalidExeFile.into()),
    });

    ret
}

#[inline]
pub fn module_export_list_callback(
    mem: &mut impl MemoryView,
    info: &ModuleInfo,
    callback: ExportCallback,
) -> Result<()> {
    export_list_callback(mem, info.base, info.size, callback)
}

pub fn export_list_callback(
    mem: &mut impl MemoryView,
    base: Address,
    size: umem,
    mut callback: ExportCallback,
) -> Result<()> {
    let mut module_image = aligned_alloc(size as usize);
    let module_image = module_image.as_bytes_mut();

    mem.read_raw_into(base, module_image).data_part()?;

    fn export_call(iter: impl Iterator<Item = (umem, ReprCString)>, callback: &mut ExportCallback) {
        iter.take_while(|(offset, name)| {
            callback.call(ExportInfo {
                name: name.clone(),
                offset: *offset,
            })
        })
        .for_each(|_| {});
    }

    let ret = Err(Error::from(ErrorKind::NotImplemented));

    #[cfg(feature = "pelite")]
    let ret = ret.or_else(|_| {
        if let Ok(pe) = pelite::PeView::from_bytes(module_image) {
            use pelite::pe64::exports::Export;

            if let Some(exports) = pe
                .exports()
                .map(Some)
                .or_else(|e| {
                    if let pelite::Error::Null = e {
                        Ok(None)
                    } else {
                        Err(e)
                    }
                })
                .map_err(|e| log::debug!("pelite: {}", e))
                .map_err(|_| ErrorKind::InvalidExeFile)?
            {
                let exports = exports
                    .by()
                    .map_err(|e| log::debug!("pelite: {}", e))
                    .map_err(|_| ErrorKind::InvalidExeFile)?;

                let iter = exports
                    .iter_names()
                    .filter_map(|(n, e)| n.ok().zip(e.ok()))
                    .filter_map(|(n, e)| match e {
                        Export::Symbol(off) => Some((*off as umem, n)),
                        _ => None,
                    })
                    .filter_map(|(o, n)| n.to_str().ok().map(|n| (o, n.into())));

                export_call(iter, &mut callback);
            }

            Ok(())
        } else {
            Err(Error::from(ErrorKind::InvalidExeFile))
        }
    });

    #[cfg(feature = "goblin")]
    let ret = ret.or_else(|_| match custom_parse(module_image)? {
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

            export_call(iter, &mut callback);

            Ok(())
        }
        Object::PE(pe) => {
            let iter = pe.exports.iter().filter_map(|e| {
                e.name
                    .map(|name| (e.offset.unwrap_or(0usize) as umem, name.into()))
            });

            export_call(iter, &mut callback);

            Ok(())
        }
        Object::Mach(Mach::Binary(bin)) => {
            let mbase = macho_base(&bin).unwrap_or_default();

            let iter = bin.exports().ok().into_iter().flatten().filter_map(|v| {
                let MachExportInfo::Regular { address, .. } = v.info else {
                    return None;
                };

                Some(((address as umem) - mbase + base.to_umem(), v.name.into()))
            });

            export_call(iter, &mut callback);

            Ok(())
        }
        _ => Err(ErrorKind::InvalidExeFile.into()),
    });

    ret
}

#[inline]
pub fn module_section_list_callback(
    mem: &mut impl MemoryView,
    info: &ModuleInfo,
    callback: SectionCallback,
) -> Result<()> {
    section_list_callback(mem, info.base, info.size, callback)
}

pub fn section_list_callback(
    mem: &mut impl MemoryView,
    base: Address,
    size: umem,
    mut callback: SectionCallback,
) -> Result<()> {
    let mut module_image = aligned_alloc(size as usize);
    let module_image = module_image.as_bytes_mut();

    mem.read_raw_into(base, module_image).data_part()?;

    fn section_call(
        iter: impl Iterator<Item = (umem, umem, ReprCString)>,
        callback: &mut SectionCallback,
        base: Address,
    ) {
        iter.take_while(|(section_base, section_size, name)| {
            callback.call(SectionInfo {
                name: name.clone(),
                base: base + *section_base,
                size: *section_size,
            })
        })
        .for_each(|_| {});
    }

    let ret = Err(Error::from(ErrorKind::NotImplemented));

    #[cfg(feature = "pelite")]
    let ret = ret.or_else(|_| {
        if let Ok(pe) = pelite::PeView::from_bytes(module_image) {
            let iter = pe.section_headers().iter().filter_map(|sh| {
                sh.name().ok().map(|name| {
                    (
                        sh.virtual_range().start as umem,
                        sh.virtual_range().end as umem,
                        name.into(),
                    )
                })
            });

            section_call(iter, &mut callback, base);

            Ok(())
        } else {
            Err(Error::from(ErrorKind::InvalidExeFile))
        }
    });

    #[cfg(feature = "goblin")]
    let ret = ret.or_else(|_| match custom_parse(module_image)? {
        Object::Elf(elf) => {
            let iter = elf.section_headers.iter().filter_map(|s| {
                elf.shdr_strtab
                    .get_at(s.sh_name)
                    .map(|n| (s.sh_addr as umem, s.sh_size as umem, ReprCString::from(n)))
            });

            section_call(iter, &mut callback, base);

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

            section_call(iter, &mut callback, base);

            Ok(())
        }
        Object::Mach(Mach::Binary(bin)) => {
            let mut base_off = None;

            let iter = bin.segments.sections().flatten().filter_map(|v| {
                let (s, _) = v.ok()?;
                let name = &s.sectname;
                let name = name.split(|&v| v == 0).next()?;
                let name = std::str::from_utf8(name).ok()?;

                let addr = s.addr as umem;

                if base_off.is_none() {
                    base_off = Some(addr);
                }

                Some((addr - base_off.unwrap(), s.size as umem, name.into()))
            });

            section_call(iter, &mut callback, base);

            Ok(())
        }
        _ => Err(ErrorKind::InvalidExeFile.into()),
    });

    ret
}
