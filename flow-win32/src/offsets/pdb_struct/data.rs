// https://github.com/willglynn/pdb/blob/master/examples/pdb2hpp.rs

use std::prelude::v1::*;

use log::{info, trace};
use std::collections::BTreeSet;

pub type TypeSet = BTreeSet<pdb::TypeIndex>;

pub fn type_name<'p>(
    type_finder: &pdb::TypeFinder<'p>,
    type_index: pdb::TypeIndex,
    needed_types: &mut TypeSet,
) -> pdb::Result<String> {
    let mut name = match type_finder.find(type_index)?.parse()? {
        pdb::TypeData::Primitive(data) => {
            let mut name = match data.kind {
                pdb::PrimitiveKind::Void => "void".to_string(),
                pdb::PrimitiveKind::Char => "char".to_string(),
                pdb::PrimitiveKind::UChar => "unsigned char".to_string(),

                pdb::PrimitiveKind::I8 => "int8_t".to_string(),
                pdb::PrimitiveKind::U8 => "uint8_t".to_string(),
                pdb::PrimitiveKind::I16 => "int16_t".to_string(),
                pdb::PrimitiveKind::U16 => "uint16_t".to_string(),
                pdb::PrimitiveKind::I32 => "int32_t".to_string(),
                pdb::PrimitiveKind::U32 => "uint32_t".to_string(),
                pdb::PrimitiveKind::I64 => "int64_t".to_string(),
                pdb::PrimitiveKind::U64 => "uint64_t".to_string(),

                pdb::PrimitiveKind::F32 => "float".to_string(),
                pdb::PrimitiveKind::F64 => "double".to_string(),

                pdb::PrimitiveKind::Bool8 => "bool".to_string(),

                _ => format!("unhandled_primitive.kind /* {:?} */", data.kind),
            };

            if data.indirection.is_some() {
                name.push_str(" *");
            }

            name
        }

        pdb::TypeData::Class(data) => {
            needed_types.insert(type_index);
            data.name.to_string().into_owned()
        }

        pdb::TypeData::Enumeration(data) => {
            needed_types.insert(type_index);
            data.name.to_string().into_owned()
        }

        pdb::TypeData::Union(data) => {
            needed_types.insert(type_index);
            data.name.to_string().into_owned()
        }

        pdb::TypeData::Pointer(data) => format!(
            "{}*",
            type_name(type_finder, data.underlying_type, needed_types)?
        ),

        pdb::TypeData::Modifier(data) => {
            if data.constant {
                format!(
                    "const {}",
                    type_name(type_finder, data.underlying_type, needed_types)?
                )
            } else if data.volatile {
                format!(
                    "volatile {}",
                    type_name(type_finder, data.underlying_type, needed_types)?
                )
            } else {
                // ?
                type_name(type_finder, data.underlying_type, needed_types)?
            }
        }

        pdb::TypeData::Array(data) => {
            let mut name = type_name(type_finder, data.element_type, needed_types)?;
            for size in data.dimensions {
                name = format!("{}[{}]", name, size);
            }
            name
        }

        _ => format!("Type{} /* TODO: figure out how to name it */", type_index),
    };

    // TODO: search and replace std:: patterns
    if name == "std::basic_string<char,std::char_traits<char>,std::allocator<char> >" {
        name = "std::string".to_string();
    }

    Ok(name)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Class<'p> {
    pub kind: pdb::ClassKind,
    pub name: pdb::RawString<'p>,
    pub base_classes: Vec<BaseClass>,
    pub fields: Vec<Field<'p>>,
    pub instance_methods: Vec<Method<'p>>,
    pub static_methods: Vec<Method<'p>>,
}

impl<'p> Class<'p> {
    fn add_derived_from(
        &mut self,
        _: &pdb::TypeFinder<'p>,
        _: pdb::TypeIndex,
        _: &mut TypeSet,
    ) -> pdb::Result<()> {
        // TODO
        Ok(())
    }

    fn add_fields(
        &mut self,
        type_finder: &pdb::TypeFinder<'p>,
        type_index: pdb::TypeIndex,
        needed_types: &mut TypeSet,
    ) -> pdb::Result<()> {
        match type_finder.find(type_index)?.parse()? {
            pdb::TypeData::FieldList(data) => {
                for field in &data.fields {
                    self.add_field(type_finder, field, needed_types)?;
                }

                if let Some(continuation) = data.continuation {
                    // recurse
                    self.add_fields(type_finder, continuation, needed_types)?;
                }
            }
            other => {
                info!(
                    "trying to Class::add_fields() got {} -> {:?}",
                    type_index, other
                );
                panic!("unexpected type in Class::add_fields()");
            }
        }

        Ok(())
    }

    fn add_field(
        &mut self,
        type_finder: &pdb::TypeFinder<'p>,
        field: &pdb::TypeData<'p>,
        needed_types: &mut TypeSet,
    ) -> pdb::Result<()> {
        match *field {
            pdb::TypeData::Member(ref data) => {
                // TODO: attributes (static, virtual, etc.)
                self.fields.push(Field {
                    type_name: type_name(type_finder, data.field_type, needed_types)?,
                    name: data.name,
                    offset: data.offset,
                });
            }

            pdb::TypeData::Method(ref data) => {
                let method = Method::find(
                    data.name,
                    data.attributes,
                    type_finder,
                    data.method_type,
                    needed_types,
                )?;
                if data.attributes.is_static() {
                    self.static_methods.push(method);
                } else {
                    self.instance_methods.push(method);
                }
            }

            pdb::TypeData::OverloadedMethod(ref data) => {
                // this just means we have more than one method with the same name
                // find the method list
                match type_finder.find(data.method_list)?.parse()? {
                    pdb::TypeData::MethodList(method_list) => {
                        for pdb::MethodListEntry {
                            attributes,
                            method_type,
                            ..
                        } in method_list.methods
                        {
                            // hooray
                            let method = Method::find(
                                data.name,
                                attributes,
                                type_finder,
                                method_type,
                                needed_types,
                            )?;

                            if attributes.is_static() {
                                self.static_methods.push(method);
                            } else {
                                self.instance_methods.push(method);
                            }
                        }
                    }
                    other => {
                        info!(
                            "processing OverloadedMethod, expected MethodList, got {} -> {:?}",
                            data.method_list, other
                        );
                        panic!("unexpected type in Class::add_field()");
                    }
                }
            }

            pdb::TypeData::BaseClass(ref data) => self.base_classes.push(BaseClass {
                type_name: type_name(type_finder, data.base_class, needed_types)?,
                offset: data.offset,
            }),

            pdb::TypeData::VirtualBaseClass(ref data) => self.base_classes.push(BaseClass {
                type_name: type_name(type_finder, data.base_class, needed_types)?,
                offset: data.base_pointer_offset,
            }),

            _ => {
                // ignore everything else even though that's sad
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseClass {
    pub type_name: String,
    pub offset: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field<'p> {
    pub type_name: String,
    pub name: pdb::RawString<'p>,
    pub offset: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Method<'p> {
    pub name: pdb::RawString<'p>,
    pub return_type_name: String,
    pub arguments: Vec<String>,
    pub is_virtual: bool,
}

impl<'p> Method<'p> {
    fn find(
        name: pdb::RawString<'p>,
        attributes: pdb::FieldAttributes,
        type_finder: &pdb::TypeFinder<'p>,
        type_index: pdb::TypeIndex,
        needed_types: &mut TypeSet,
    ) -> pdb::Result<Method<'p>> {
        match type_finder.find(type_index)?.parse()? {
            pdb::TypeData::MemberFunction(data) => Ok(Method {
                name,
                return_type_name: type_name(type_finder, data.return_type, needed_types)?,
                arguments: argument_list(type_finder, data.argument_list, needed_types)?,
                is_virtual: attributes.is_virtual(),
            }),

            other => {
                info!("other: {:?}", other);
                Err(pdb::Error::UnimplementedFeature("that"))
            }
        }
    }
}

fn argument_list<'p>(
    type_finder: &pdb::TypeFinder<'p>,
    type_index: pdb::TypeIndex,
    needed_types: &mut TypeSet,
) -> pdb::Result<Vec<String>> {
    match type_finder.find(type_index)?.parse()? {
        pdb::TypeData::ArgumentList(data) => {
            let mut args: Vec<String> = Vec::new();
            for arg_type in data.arguments {
                args.push(type_name(type_finder, arg_type, needed_types)?);
            }
            Ok(args)
        }
        _ => Err(pdb::Error::UnimplementedFeature(
            "argument list of non-argument-list type",
        )),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enum<'p> {
    name: pdb::RawString<'p>,
    underlying_type_name: String,
    values: Vec<EnumValue<'p>>,
}

impl<'p> Enum<'p> {
    fn add_fields(
        &mut self,
        type_finder: &pdb::TypeFinder<'p>,
        type_index: pdb::TypeIndex,
        needed_types: &mut TypeSet,
    ) -> pdb::Result<()> {
        match type_finder.find(type_index)?.parse()? {
            pdb::TypeData::FieldList(data) => {
                for field in &data.fields {
                    self.add_field(type_finder, field, needed_types)?;
                }

                if let Some(continuation) = data.continuation {
                    // recurse
                    self.add_fields(type_finder, continuation, needed_types)?;
                }
            }
            other => {
                info!(
                    "trying to Enum::add_fields() got {} -> {:?}",
                    type_index, other
                );
                panic!("unexpected type in Enum::add_fields()");
            }
        }

        Ok(())
    }

    fn add_field(
        &mut self,
        _: &pdb::TypeFinder<'p>,
        field: &pdb::TypeData<'p>,
        _: &mut TypeSet,
    ) -> pdb::Result<()> {
        // ignore everything else even though that's sad
        if let pdb::TypeData::Enumerate(ref data) = field {
            self.values.push(EnumValue {
                name: data.name,
                value: data.value,
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumValue<'p> {
    name: pdb::RawString<'p>,
    value: pdb::Variant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForwardReference<'p> {
    kind: pdb::ClassKind,
    name: pdb::RawString<'p>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data<'p> {
    pub forward_references: Vec<ForwardReference<'p>>,
    pub classes: Vec<Class<'p>>,
    pub enums: Vec<Enum<'p>>,
}

impl<'p> Data<'p> {
    pub fn new() -> Data<'p> {
        Data {
            forward_references: Vec::new(),
            classes: Vec::new(),
            enums: Vec::new(),
        }
    }

    pub fn add(
        &mut self,
        type_finder: &pdb::TypeFinder<'p>,
        type_index: pdb::TypeIndex,
        needed_types: &mut TypeSet,
    ) -> pdb::Result<()> {
        match type_finder.find(type_index)?.parse()? {
            pdb::TypeData::Class(data) => {
                if data.properties.forward_reference() {
                    self.forward_references.push(ForwardReference {
                        kind: data.kind,
                        name: data.name,
                    });

                    return Ok(());
                }

                let mut class = Class {
                    kind: data.kind,
                    name: data.name,
                    fields: Vec::new(),
                    base_classes: Vec::new(),
                    instance_methods: Vec::new(),
                    static_methods: Vec::new(),
                };

                if let Some(derived_from) = data.derived_from {
                    class.add_derived_from(type_finder, derived_from, needed_types)?;
                }

                if let Some(fields) = data.fields {
                    class.add_fields(type_finder, fields, needed_types)?;
                }

                self.classes.insert(0, class);
            }

            pdb::TypeData::Enumeration(data) => {
                let mut e = Enum {
                    name: data.name,
                    underlying_type_name: type_name(
                        type_finder,
                        data.underlying_type,
                        needed_types,
                    )?,
                    values: Vec::new(),
                };

                e.add_fields(type_finder, data.fields, needed_types)?;

                self.enums.insert(0, e);
            }

            // ignore
            other => trace!("don't know how to add {:?}", other),
        }

        Ok(())
    }
}
