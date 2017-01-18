use quick_xml::{AsStr, Element, Event, XmlReader, XmlWriter};
use quick_xml::Event::*;
use std::io::Cursor;
use std::iter;
use document::Element as AbxmlElement;
use document::{Namespaces, Value, Entries};
use std::ops::Deref;
use std::io::Write;
use std::rc::Rc;
use errors::*;
use chunks::StringTable;

pub struct Xml;

impl Xml {
    pub fn encode(namespaces: &Namespaces, element: &AbxmlElement, entries: &Entries, string_table: &StringTable, entries_string_table: &StringTable) -> Result<String> {
        let mut writer = XmlWriter::new(Cursor::new(Vec::new()));

        Self::encode_element(&mut writer, Some(namespaces), element, entries, string_table, entries_string_table);

        let result = writer.into_inner().into_inner();
        let str_result = String::from_utf8(result).unwrap();
        let output = format!("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"no\"?>\n{}", str_result);

        Ok(output)
    }

    fn encode_element<W: Write>(mut writer: &mut XmlWriter<W>, namespaces: Option<&Namespaces>, element: &AbxmlElement, entries: &Entries, string_table: &StringTable, entries_string_table: &StringTable) {
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

            let val = match a.get_value() {
                &Value::ReferenceId(ref id) => {
                    Some(Self::resolve_reference(*id, string_table, &entries, &entries_string_table))
                },
                &Value::AttributeReferenceId(ref id) => {
                    Some(Self::resolve_reference(*id, string_table, &entries, &entries_string_table))
                },
                _ => {
                    None
                }
            };

            elem.push_attribute(
                final_name,
                &val.unwrap_or(a.get_value_as_str()),
            );
        }

        writer.write(Start(elem)).unwrap();

        for child in element.get_children() {
            Self::encode_element(&mut writer, None, child, entries, string_table, entries_string_table)
        }

        writer.write(End(Element::new(tag.deref()))).unwrap();
    }

    fn resolve_reference(id: u32, string_table: &StringTable, entries: &Entries, entries_string_table: &StringTable) -> String {
        match entries.get(&id) {
            Some(e) => {
                println!("Attribute ref found: {:?}", e);
            },
            None => {
                println!("None attribute ref found");

                match string_table.get_uncached_string(id) {
                    Ok(s) => {
                        println!("Found on st: {}", s);
                    },
                    Err(_) => {
                        println!("Attr not found on st");
                    }
                }
            },
        };

        format!("attr ref: #{}", id)
    }

    pub fn namespaces_to_attributes(namespaces: &Namespaces) -> Vec<(String, String)> {
        let mut output = Vec::new();
        let xmlns = Rc::new(String::from("xmlns"));

        for (prefix, namespace) in namespaces {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    // use xml::writer::EmitterConfig;
    use document::Element;
    use document::Attribute;
    use document::Value;

    #[test]
    fn it_can_encode_an_element() {
        let mut attrs: Vec<Attribute> = Vec::new();
        let at = Attribute::new(
            Rc::new("test".to_string()),
            Value::Float(64.55),
            None,
            None,
        );
        attrs.push(at);
        let e = Element::new(Rc::new("element1".to_string()), attrs);
        let namespaces: Namespaces = BTreeMap::new();
        let mut target: Vec<u8> = Vec::new();
        {
            // let mut event_writer = EmitterConfig::new().create_writer(&mut target);
            let result = Xml::encode(namespaces, &e);
            println!("Result: {}", result.unwrap());
        }
        // let result = String::from_utf8(target).unwrap();

        panic!("");
    }
}
