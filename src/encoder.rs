use quick_xml::{Element, XmlWriter};
use quick_xml::Event::*;
use std::io::Cursor;
use model::Element as AbxmlElement;
use std::ops::Deref;
use std::io::Write;
use std::rc::Rc;
use errors::*;
use visitor::Resources;
use model::{Identifier, Namespaces, Value, Attribute};

pub struct Xml;

impl Xml {
    pub fn encode(namespaces: &Namespaces, element: &AbxmlElement, xml_resources: &[u32], resources: &Resources) -> Result<String> {
        let mut writer = XmlWriter::new(Cursor::new(Vec::new()));

        Self::encode_element(&mut writer, Some(namespaces), element, xml_resources, resources)?;

        let result = writer.into_inner().into_inner();
        let str_result = String::from_utf8(result)?;
        let output = format!("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"no\"?>\n{}", str_result);

        Ok(output)
    }

    fn encode_element<W: Write>(mut writer: &mut XmlWriter<W>, namespaces: Option<&Namespaces>, element: &AbxmlElement, xml_resources: &[u32], resources: &Resources) -> Result<()> {
        let tag = element.get_tag();
        let mut elem = Element::new(tag.deref());

        if let Some(ns) = namespaces {
            let xmlns = Self::namespaces_to_attributes(ns);
            elem.extend_attributes(xmlns);
        }

        for a in element.get_attributes().iter() {
            let rc_name = a.get_name();
            let prefix = a.get_prefix();
            let final_name = Self::attribute_name(rc_name, prefix);


            let val = match *a.get_value() {
                Value::ReferenceId(ref id) => {
                    Self::resolve_reference(*id, resources, "@").ok()
                }
                Value::AttributeReferenceId(ref id) => {
                    Self::resolve_reference(*id, resources, "?").ok()
                },
                Value::Integer(ref value) |
                Value::Flags(ref value) => {
                    let flag_resolution = Self::resolve_flags(*value as u32, a, xml_resources, resources);

                    if flag_resolution.is_none() {
                        Some(a.get_value().to_string())
                    } else {
                        flag_resolution
                    }
                },
                _ => {
                    None
                }
            };

            elem.push_attribute(
                final_name,
                &val.unwrap_or_else(|| a.get_value_as_str()),
            );
        }

        writer.write(Start(elem))?;

        for child in element.get_children() {
            Self::encode_element(&mut writer, None, child, xml_resources, resources)?
        }

        writer.write(End(Element::new(tag.deref())))?;

        Ok(())
    }

    fn resolve_reference(id: u32, resources: &Resources, prefix: &str) -> Result<String> {
        let res_id = id;
        let package_id = id.get_package();

        if id == 0 {
            return Ok("@null".to_string());
        }

        let is_main = resources.is_main_package(package_id);
        let package = resources.get_package(package_id)?;
        let mut package_borrow = package.borrow_mut();

        let entry_key = package_borrow
            .get_entries()
            .get(&res_id)
            .and_then(|e| Some(e.get_key()));

        if let Some(key) = entry_key {
            let namespace = if !is_main {
                package_borrow.get_name()
            } else {
                None
            };

            return package_borrow.format_reference(id, key, namespace, prefix);
        }

        Err("Error resolving reference".into())
    }

    fn resolve_flags(flags: u32, attribute: &Attribute, xml_resources: &[u32], resources: &Resources) -> Option<String> {
        // Check if it's the special value in which the integer is an Enum
        // In that case, we return a crafted string instead of the integer itself
        let name_index = attribute.get_name_index();
        if name_index < xml_resources.len() as u32 {
            let entry_ref = match xml_resources.get(name_index as usize) {
                Some(entry_ref) => entry_ref,
                None => return None,
            };

            let package_id = entry_ref.get_package();
            if let Ok(package) = resources.get_package(package_id as u8) {
                let str_indexes = {
                    let mut strs = Vec::new();
                    let mut masks = Vec::new();

                    let pb = package.borrow();

                    let inner_entries = pb.get_entry(*entry_ref)
                        .and_then(|e| e.complex())
                        .and_then(|c| Ok(c.get_entries().to_vec()))
                        .unwrap_or(Vec::new());
                    let mut sorted = inner_entries.to_vec();

                    sorted.sort_by(|a, b| {
                        let id_a = a.get_value();
                        let id_b = b.get_value();

                        // TODO: This code is to create an exact match with Apktool. A simple descending ordering seems to be also ok.
                        let mut i = id_a;
                        i = i - ((i >> 1) & 0x55555555);
                        i = (i & 0x33333333) + ((i >> 2) & 0x33333333);
                        i = (i + (i >> 4)) & 0x0f0f0f0f;
                        i = i + (i >> 8);
                        i = i + (i >> 16);
                        i = i & 0x3f;

                        let mut j = id_b;
                        j = j - ((j >> 1) & 0x55555555);
                        j = (j & 0x33333333) + ((j >> 2) & 0x33333333);
                        j = (j + (j >> 4)) & 0x0f0f0f0f;
                        j = j + (j >> 8);
                        j = j + (j >> 16);
                        j = j & 0x3f;

                        j.cmp(&i)
                    });

                    for ie in sorted {
                        let mask = ie.get_value();

                        if (mask & flags) == mask {
                            let maybe_entry = pb.get_entry(ie.get_id());

                            match maybe_entry {
                                Ok(entry) => {
                                    let mut has_to_add = true;

                                    for s in masks.iter() {
                                        if mask & s == mask {
                                            has_to_add = false;
                                            break;
                                        }
                                    }

                                    if has_to_add {
                                        entry.simple()
                                            .and_then(|s| Ok(s.get_key()))
                                            .and_then(|key| {
                                                strs.push(key);
                                                masks.push(mask);
                                                Ok(())
                                            })
                                            .unwrap_or_else(|_| {
                                                error!("Value should be added but there was an issue reading the entry");
                                            });
                                    }
                                },
                                Err(_) => {
                                    info!("Some entry matched but could not found on entries");
                                }
                            }
                        }
                    }

                    strs
                };

                let str_strs: Vec<String> = str_indexes
                    .iter()
                    .map(|si| {
                        let mut pb = package.borrow_mut();

                        match pb.get_entries_string(*si) {
                            Ok(str) => str,
                            Err(_) => {
                                error!("Key not found on the string table");

                                "".to_string()
                            },
                        }
                    })
                    .collect();

                let final_string = str_strs.join("|");

                if str_strs.is_empty() {
                    return None;
                } else {
                    return Some(final_string);
                }
            }

            None
        } else {
            let str = format!("@flags:{}", flags);

            Some(str.to_string())
        }
    }

    pub fn namespaces_to_attributes(namespaces: &Namespaces) -> Vec<(String, String)> {
        let mut output = Vec::new();
        let xmlns = Rc::new(String::from("xmlns"));

        for (namespace, prefix) in namespaces {
            let label = Self::attribute_name(prefix.clone(), Some(xmlns.clone()));

            output.push(
                (label, namespace.deref().clone())
            );
        }

        output
    }

    pub fn attribute_name(label: Rc<String>, prefix: Option<Rc<String>>) -> String {
        let name = label.deref();

        prefix
            .and_then(|rc_prefix| {
                let p = rc_prefix.deref();

                let mut s = String::new();
                s.push_str(p);
                s.push_str(":");
                s.push_str(name);

                Some(s)
            })
            .unwrap_or(name.to_owned())
    }
}
