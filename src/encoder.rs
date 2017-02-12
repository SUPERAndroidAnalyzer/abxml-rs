use quick_xml::{Element,    XmlWriter};
use quick_xml::Event::*;
use std::io::Cursor;
use document::Element as AbxmlElement;
use document::{Namespaces, Value};
use std::ops::Deref;
use std::io::Write;
use std::rc::Rc;
use errors::*;
use visitor::Resources;

pub struct Xml;

impl Xml {
    pub fn encode(namespaces: &Namespaces, element: &AbxmlElement, resources: &Resources) -> Result<String> {
        let mut writer = XmlWriter::new(Cursor::new(Vec::new()));

        Self::encode_element(&mut writer, Some(namespaces), element, resources);

        let result = writer.into_inner().into_inner();
        let str_result = String::from_utf8(result).unwrap();
        let output = format!("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"no\"?>\n{}", str_result);

        Ok(output)
    }

    fn encode_element<W: Write>(mut writer: &mut XmlWriter<W>, namespaces: Option<&Namespaces>, element: &AbxmlElement, resources: &Resources) {
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
                Value::ReferenceId(ref id) |
                Value::AttributeReferenceId(ref id)=> {
                    println!("Reference: {}", id);
                    Self::resolve_reference(*id, resources)
                },
                Value::Integer(ref id) => {
                    // Check if it's the special value in which the integer is an Enum
                    // In that case, we return a crafted string instead of the integer itself
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

        writer.write(Start(elem)).unwrap();

        for child in element.get_children() {
            Self::encode_element(&mut writer, None, child, resources)
        }

        writer.write(End(Element::new(tag.deref()))).unwrap();
    }

    fn resolve_reference(id: u32, resources: &Resources) -> Option<String> {
        let mut res_id = id;
        let mut package_id = (id >> 24) as u8;

        if package_id == 0 {
            res_id = ((0xFF & 1) << 24) | id;
            package_id = 1;
            info!("Resource with package id 0 found. Recreate id with current package id");
        }

        let is_main = resources.is_main_package(package_id);
        let package = resources.get_package(package_id);
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

            return Some(package_borrow.format_reference(id, key, namespace).unwrap());
        }

        None

/*        info!("Reference not found on Resources");
        println!("Reference: {} {} {}", res_id, id, id >> 24);
        panic!("We end on a invalid reference");

        "UNKNOWN".to_string()*/
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

        if prefix.is_some() {
            let rc_prefix = prefix.unwrap();
            let p = rc_prefix.deref();

            let mut s = String::new();
            s.push_str(p);
            s.push_str(":");
            s.push_str(name);

            s
        } else {
            let mut s = String::new();
            s.push_str(name);

            s
        }
    }
}
