//! Exports the decoded binary XMLs to string XMLs

use crate::model::{Element as AbxmlElement, Namespaces};
use anyhow::{Context, Result};
use quick_xml::{
    events::{BytesDecl, BytesEnd, BytesStart, Event},
    Writer,
};
use std::{io::Write, ops::Deref};

#[derive(Debug, Copy, Clone)]
pub struct Xml;

impl Xml {
    pub fn encode(namespaces: &Namespaces, element: &AbxmlElement) -> Result<String> {
        let target: Vec<u8> = Vec::new();
        let mut writer = Writer::new_with_indent(target, b' ', 2);

        writer.write_event(Event::Decl(BytesDecl::new(b"1.0", None, None)))?;
        Self::encode_element(&mut writer, namespaces, element)
            .context("error decoding an element")?;

        let inner = writer.into_inner();
        Ok(String::from_utf8(inner).context("could not export XML")?)
    }

    fn encode_element<W>(
        writer: &mut Writer<W>,
        namespaces: &Namespaces,
        element: &AbxmlElement,
    ) -> Result<()>
    where
        W: Write,
    {
        let tag = element.tag();
        let tag_name = tag.name();
        let prefixes = tag.prefixes();
        let mut start_bytes = BytesStart::borrowed_name(tag_name.as_bytes());

        start_bytes = start_bytes.with_attributes(
            element
                .attributes()
                .into_iter()
                .map(|(a, v)| (a.as_bytes(), v.as_bytes())),
        );

        for uri in prefixes {
            let prefix = namespaces.get(&uri.deref().clone());
            if let Some(p) = prefix {
                start_bytes = start_bytes.ns(p.as_str(), uri.as_str());
            }
        }

        writer.write_event(Event::Start(start_bytes))?;

        for child in element.children() {
            Self::encode_element(writer, namespaces, child)?;
        }

        writer.write_event(Event::End(BytesEnd::borrowed(tag_name.as_bytes())))?;

        Ok(())
    }
}
