mod data;

use std::prelude::v1::*;

use data::TypeSet;
use std::collections::HashMap;
use std::{fmt, io, result};

use pdb::{FallibleIterator, Result, Source, SourceSlice, SourceView, TypeData, PDB};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdbField {
    pub type_name: String,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdbStruct {
    field_map: HashMap<String, PdbField>,
}

impl PdbStruct {
    pub fn with(pdb_slice: &[u8], class_name: &str) -> Result<Self> {
        let pdb_buffer = PdbSourceBuffer::new(pdb_slice);
        let mut pdb = PDB::open(pdb_buffer)?;

        let type_information = pdb.type_information()?;
        let mut type_finder = type_information.finder();

        let mut needed_types = TypeSet::new();
        let mut data = data::Data::new();

        let mut type_iter = type_information.iter();
        while let Some(typ) = type_iter.next()? {
            // keep building the index
            type_finder.update(&type_iter);

            if let Ok(TypeData::Class(class)) = typ.parse() {
                if class.name.as_bytes() == class_name.as_bytes()
                    && !class.properties.forward_reference()
                {
                    data.add(&type_finder, typ.index(), &mut needed_types)?;
                    break;
                }
            }
        }

        // add all the needed types iteratively until we're done
        loop {
            // get the last element in needed_types without holding an immutable borrow
            let last = match needed_types.iter().next_back() {
                Some(n) => Some(*n),
                None => None,
            };

            if let Some(type_index) = last {
                // remove it
                needed_types.remove(&type_index);

                // add the type
                data.add(&type_finder, type_index, &mut needed_types)?;
            } else {
                break;
            }
        }

        let mut field_map = HashMap::new();
        for class in &data.classes {
            class.fields.iter().for_each(|f| {
                field_map.insert(
                    f.name.to_string().into_owned(),
                    PdbField {
                        type_name: f.type_name.clone(),
                        offset: f.offset as usize,
                    },
                );
            });
        }

        Ok(Self { field_map })
    }

    pub fn find_field(&self, name: &str) -> Option<&PdbField> {
        self.field_map.get(name)
    }
}

pub struct PdbSourceBuffer<'a> {
    bytes: &'a [u8],
}

impl<'a> PdbSourceBuffer<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
}

impl<'a> fmt::Debug for PdbSourceBuffer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PdbSourceBuffer({} bytes)", self.bytes.len())
    }
}

impl<'a, 's> Source<'s> for PdbSourceBuffer<'a> {
    fn view(
        &mut self,
        slices: &[SourceSlice],
    ) -> result::Result<Box<dyn SourceView<'s>>, io::Error> {
        let len = slices.iter().fold(0 as usize, |acc, s| acc + s.size);

        let mut v = PdbSourceBufferView {
            bytes: Vec::with_capacity(len),
        };
        v.bytes.resize(len, 0);

        let bytes = v.bytes.as_mut_slice();
        let mut output_offset: usize = 0;
        for slice in slices {
            bytes[output_offset..(output_offset + slice.size)].copy_from_slice(
                &self.bytes[slice.offset as usize..(slice.offset as usize + slice.size)],
            );
            output_offset += slice.size;
        }

        Ok(Box::new(v))
    }
}

#[derive(Clone)]
struct PdbSourceBufferView {
    bytes: Vec<u8>,
}

impl fmt::Debug for PdbSourceBufferView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PdbSourceBufferView({} bytes)", self.bytes.len())
    }
}

impl SourceView<'_> for PdbSourceBufferView {
    fn as_slice(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

impl Drop for PdbSourceBufferView {
    fn drop(&mut self) {
        // no-op
    }
}
