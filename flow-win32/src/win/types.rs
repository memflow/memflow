// TODO: custom errors
use pdb::{FallibleIterator, Result, PDB};
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

mod data;
use data::TypeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub type_name: String,
    pub offset: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct {
    field_map: HashMap<String, Field>,
}

impl Struct {
    // TODO: wrap this in win->struct("asdf")
    pub fn from(filename: PathBuf, class_name: &str) -> Result<Self> {
        let file = File::open(filename)?;
        let mut pdb = PDB::open(file)?;

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
                    //println!("{:?}", class);
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

        if data.classes.is_empty() {
            eprintln!("sorry, class {} was not found", class_name);
        } else {
            println!("{}", data);
        }

        println!("--------------------");

        //data.classes.iter()
        //    .inspect()

        // TODO: transform this to a hashmap
        let mut field_map = HashMap::new();
        for class in &data.classes {
            //println!("{:?}", class.name);
            // TODO: filter class?
            /*
                    class
                        .fields
                        .iter()
                        .filter(|f| f.name.to_string() == "UniqueProcessId")
                        .for_each(|f| println!("{:?}", f));
            */
            class.fields.iter().for_each(|f| {
                field_map.insert(
                    f.name.to_string().into_owned(),
                    Field {
                        type_name: f.type_name.clone(),
                        offset: f.offset,
                    },
                );
            });
        }

        Ok(Struct {
            field_map: field_map,
        })
    }

    pub fn get_field(&self, name: &str) -> Option<&Field> {
        self.field_map.get(name)
    }
}
