use quick_xml::{AsStr, Element, Event, XmlReader, XmlWriter};
use quick_xml::Event::*;
use std::io::Cursor;
use std::iter;
use document::Element as AbxmlElement;
use std::ops::Deref;
use std::io::Write;

pub struct Xml;

impl Xml {
    pub fn encode(element: &AbxmlElement) -> Result<String, ()> {
        let mut writer = XmlWriter::new(Cursor::new(Vec::new()));

        Self::encode_element(&mut writer, element);

        let result = writer.into_inner().into_inner();
        let str_result = String::from_utf8(result).unwrap();
        let output = format!("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"no\"?>\n{}", str_result);

        Ok(output)
    }

    fn encode_element<W: Write>(mut writer: &mut XmlWriter<W>, element: &AbxmlElement) {
        let tag = element.get_tag();
        let mut elem = Element::new(tag.deref());

        for a in element.get_attributes().iter() {
            let name = a.get_name();

            elem.push_attribute(
                name.deref(),
                a.get_value(),
            );
        }

        writer.write(Start(elem)).unwrap();

        for child in element.get_children() {
            Self::encode_element(&mut writer, child)
        }

        writer.write(End(Element::new(tag.deref()))).unwrap();
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
        let mut target: Vec<u8> = Vec::new();
        {
            // let mut event_writer = EmitterConfig::new().create_writer(&mut target);
            let result = Xml::encode(&e);
            println!("Result: {}", result.unwrap());
        }
        // let result = String::from_utf8(target).unwrap();

        panic!("");
    }
}
