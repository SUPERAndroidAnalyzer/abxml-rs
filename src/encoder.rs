//! Exports the decoded binary XMLs to string XMLs

use std::io::Write;
use std::ops::Deref;

use failure::{Error, ResultExt};
use xml::common::XmlVersion;
use xml::writer::{EmitterConfig, EventWriter, XmlEvent};

use model::{Element as AbxmlElement, Namespaces};

pub struct Xml;

impl Xml {
    pub fn encode(namespaces: &Namespaces, element: &AbxmlElement) -> Result<String, Error> {
        let target: Vec<u8> = Vec::new();
        let mut writer = EmitterConfig::new()
            .perform_indent(true)
            .create_writer(target);

        let version = XmlVersion::Version10;
        writer.write(XmlEvent::StartDocument {
            version: version,
            encoding: None,
            standalone: Some(false),
        })?;
        Self::encode_element(&mut writer, namespaces, element)
            .context("error decoding an element")?;

        let inner = writer.into_inner();
        Ok(String::from_utf8(inner).context("could not export XML")?)
    }

    fn encode_element<W: Write>(
        writer: &mut EventWriter<W>,
        namespaces: &Namespaces,
        element: &AbxmlElement,
    ) -> Result<(), Error> {
        let tag = element.get_tag();
        let tag_name = tag.get_name();
        let prefixes = tag.get_prefixes();
        let mut xml_element = XmlEvent::start_element(tag_name.deref().as_str());

        for (k, v) in element.get_attributes() {
            xml_element = xml_element.attr(k.as_str(), v);
        }

        for uri in prefixes {
            let prefix = namespaces.get(&uri.deref().clone());
            if let Some(p) = prefix {
                xml_element = xml_element.ns(p.as_str(), uri.as_str());
            }
        }

        writer.write(xml_element)?;

        for child in element.get_children() {
            Self::encode_element(writer, namespaces, child)?;
        }

        writer.write(XmlEvent::end_element())?;

        Ok(())
    }
}
