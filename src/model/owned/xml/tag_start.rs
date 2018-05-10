use byteorder::{LittleEndian, WriteBytesExt};
use failure::Error;

use chunks::*;
use model::owned::{AttributeBuf, OwnedBuf};
use model::{AttributeTrait, TagStart};

/// Representation of a XML Tag start chunk
#[derive(Debug)]
pub struct XmlTagStartBuf {
    /// Attributes of the tag
    attributes: Vec<AttributeBuf>,
    /// Index of the string on the main string table
    name: u32,
    /// Index of the namespace
    namespace: u32,
    /// ¿Line of the xml?
    line: u32,
    /// Unknown field
    field1: u32,
    /// Unknown field
    field2: u32,
    /// ¿Class?
    class: u32,
}

impl XmlTagStartBuf {
    /// Creates a new `XmlTagStartBuf` with the given data
    pub fn new(line: u32, field1: u32, namespace: u32, name: u32, field2: u32, class: u32) -> Self {
        Self {
            attributes: Vec::new(),
            name,
            namespace,
            line,
            field1,
            field2,
            class,
        }
    }

    /// Adds a new attribute to the XML tag
    pub fn add_attribute(&mut self, attribute: AttributeBuf) {
        self.attributes.push(attribute);
    }
}

impl TagStart for XmlTagStartBuf {
    type Attribute = AttributeBuf;

    fn get_line(&self) -> Result<u32, Error> {
        Ok(self.line)
    }

    fn get_field1(&self) -> Result<u32, Error> {
        Ok(self.field1)
    }

    fn get_namespace_index(&self) -> Result<u32, Error> {
        Ok(self.namespace)
    }

    fn get_element_name_index(&self) -> Result<u32, Error> {
        Ok(self.name)
    }

    fn get_field2(&self) -> Result<u32, Error> {
        Ok(self.field2)
    }

    fn get_attributes_amount(&self) -> Result<u32, Error> {
        Ok(self.attributes.len() as u32)
    }

    fn get_class(&self) -> Result<u32, Error> {
        Ok(self.class)
    }

    fn get_attribute(&self, index: u32) -> Result<Self::Attribute, Error> {
        if let Some(attr) = self.attributes.get(index as usize) {
            Ok(*attr)
        } else {
            bail!("requested attribute out of bounds")
        }
    }
}

impl OwnedBuf for XmlTagStartBuf {
    fn get_token(&self) -> u16 {
        TOKEN_XML_TAG_START
    }

    fn get_body_data(&self) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();

        out.write_u32::<LittleEndian>(self.namespace)?;
        out.write_u32::<LittleEndian>(self.name)?;
        out.write_u32::<LittleEndian>(self.field2)?;
        out.write_u32::<LittleEndian>(self.attributes.len() as u32)?;
        out.write_u32::<LittleEndian>(self.class)?;

        for a in &self.attributes {
            out.write_u32::<LittleEndian>(a.get_namespace()?)?;
            out.write_u32::<LittleEndian>(a.get_name()?)?;
            out.write_u32::<LittleEndian>(a.get_class()?)?;
            out.write_u32::<LittleEndian>(a.get_resource_value()?)?;
            out.write_u32::<LittleEndian>(a.get_data()?)?;
        }

        Ok(out)
    }

    fn get_header(&self) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();

        out.write_u32::<LittleEndian>(self.line)?;
        out.write_u32::<LittleEndian>(self.field1)?;

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chunks::XmlTagStartWrapper;
    use model::owned::AttributeBuf;
    use raw_chunks::EXAMPLE_TAG_START;
    use test::compare_chunks;

    #[test]
    fn it_can_generate_a_chunk_with_the_given_data() {
        let attribute1 = AttributeBuf::new(1, 2, 3, 5, 6);
        let attribute2 = AttributeBuf::new(7, 8, 9, 11, 12);

        let mut tag_start = XmlTagStartBuf::new(10, 22, 0xFFFFFFFF, 7, 5, 3);
        tag_start.add_attribute(attribute1);
        tag_start.add_attribute(attribute2);

        assert_eq!(10, tag_start.get_line().unwrap());
        assert_eq!(22, tag_start.get_field1().unwrap());
        assert_eq!(7, tag_start.get_element_name_index().unwrap());
        assert_eq!(5, tag_start.get_field2().unwrap());
        assert_eq!(3, tag_start.get_class().unwrap());
        assert_eq!(2, tag_start.get_attributes_amount().unwrap());
        assert_eq!(0xFFFFFFFF, tag_start.get_namespace_index().unwrap());
        let first_attribute = tag_start.get_attribute(0).unwrap();
        assert_eq!(1, first_attribute.get_namespace().unwrap());
        let third_attribute = tag_start.get_attribute(2);
        assert!(third_attribute.is_err());
    }

    #[test]
    fn identity() {
        let raw = EXAMPLE_TAG_START;
        let wrapper = XmlTagStartWrapper::new(raw);

        let owned = wrapper.to_buffer().unwrap();
        let new_raw = owned.to_vec().unwrap();

        compare_chunks(&raw, &new_raw);
    }
}
