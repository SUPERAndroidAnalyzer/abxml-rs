use quick_xml::{Element, XmlWriter};
use quick_xml::Event::*;
use std::io::Cursor;
use model::Element as AbxmlElement;
use std::ops::Deref;
use std::io::Write;
use std::rc::Rc;
use errors::*;
use visitor::Resources;
use model::{Namespaces, Value};

pub struct Xml;

impl Xml {
    pub fn encode(namespaces: &Namespaces,
                  element: &AbxmlElement,
                  xml_resources: &[u32],
                  resources: &Resources)
                  -> Result<String> {
        let mut writer = XmlWriter::new(Cursor::new(Vec::new()));

        Self::encode_element(&mut writer,
                             Some(namespaces),
                             element,
                             xml_resources,
                             resources).chain_err(|| "Error decoding an element")?;

        let result = writer.into_inner().into_inner();
        let str_result = String::from_utf8(result).chain_err(|| "Could not encode to UTF-8")?;
        let output = format!("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"no\"?>\n{}",
                             str_result);

        Ok(output)
    }

    fn encode_element<W: Write>(mut writer: &mut XmlWriter<W>,
                                namespaces: Option<&Namespaces>,
                                element: &AbxmlElement,
                                xml_resources: &[u32],
                                resources: &Resources)
                                -> Result<()> {
        let tag = element.get_tag();
        let mut elem = Element::new(tag.deref());

        if let Some(ns) = namespaces {
            let xmlns = Self::namespaces_to_attributes(ns);
            elem.extend_attributes(xmlns);
        }

        for (k, v) in element.get_attributes() {
            elem.push_attribute(k, v);
        }

        writer.write(Start(elem)).chain_err(|| "Error while writ ing start element")?;

        for child in element.get_children() {
            Self::encode_element(&mut writer, None, child, xml_resources, resources)
                .chain_err(|| "Error while writing a children")?;
        }

        writer.write(End(Element::new(tag.deref())))
            .chain_err(|| "Error while writing end element")?;

        Ok(())
    }

    pub fn namespaces_to_attributes(namespaces: &Namespaces) -> Vec<(String, String)> {
        let mut output = Vec::new();
        let xmlns = Rc::new(String::from("xmlns"));

        for (namespace, prefix) in namespaces {
            let label = Self::attribute_name((*prefix).clone(), Some(xmlns.clone()));

            output.push((label, (*namespace).clone()));
        }

        output
    }

    pub fn attribute_name(label: String, prefix: Option<Rc<String>>) -> String {
        let name = label;

        prefix.and_then(|rc_prefix| {
                let p = rc_prefix.deref();

                let mut s = String::new();
                s.push_str(p);
                s.push_str(":");
                s.push_str(&name);

                Some(s)
            })
            .unwrap_or_else(|| name.to_owned())
    }
}
