use log::trace;

use pdb::{self, FallibleIterator, Result};
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use flow_core::address::Length;

mod data;
use data::TypeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub type_name: String,
    pub offset: Length,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct {
    field_map: HashMap<String, Field>,
}

impl Struct {
    pub fn from(filename: PathBuf, class_name: &str) -> Result<Self> {
        let file = File::open(filename)?;
        let mut pdb = pdb::PDB::open(file)?;

        let type_information = pdb.type_information()?;
        let mut type_finder = type_information.finder();

        let mut needed_types = TypeSet::new();
        let mut data = data::Data::new();

        let mut type_iter = type_information.iter();
        while let Some(typ) = type_iter.next()? {
            // keep building the index
            type_finder.update(&type_iter);

            if let Ok(pdb::TypeData::Class(class)) = typ.parse() {
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

        // TODO: transform this to a hashmap
        let mut field_map = HashMap::new();
        for class in &data.classes {
            // TODO: filter class?
            class.fields.iter().for_each(|f| {
                //println!("found offset: {:?}", f.name.to_string().into_owned());
                field_map.insert(
                    f.name.to_string().into_owned(),
                    Field {
                        type_name: f.type_name.clone(),
                        offset: Length::from(f.offset),
                    },
                );
            });
        }

        Ok(Struct { field_map })
    }

    pub fn get_field(&self, name: &str) -> Option<&Field> {
        self.field_map.get(name)
    }
}

#[derive(Clone)]
pub struct PDB {
    pub file: PathBuf,
    pub structs: HashMap<String, Struct>,
}

impl PDB {
    pub fn new(file: PathBuf) -> Self {
        PDB {
            file,
            structs: HashMap::new(),
        }
    }

    pub fn get_struct(&mut self, name: &str) -> Option<Struct> {
        match self.structs.get(name) {
            Some(s) => return Some(s.clone()),
            None => trace!("struct {} not found in cache", name),
        }

        match Struct::from(self.file.clone(), name) {
            Ok(s) => {
                self.structs.insert(String::from(name), s.clone());
                return Some(s);
            }
            Err(e) => trace!("struct {} not found: {:?}", name, e),
        }

        None
    }
}
