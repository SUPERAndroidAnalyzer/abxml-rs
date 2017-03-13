use model::owned::OwnedBuf;
use model::TagStart;
use byteorder::{LittleEndian, WriteBytesExt};
use chunks::*;
use errors::*;
use model::attribute::Attribute;
use std::rc::Rc;

pub struct XmlTagStartBuf {
    /// Attributes of the tag
    attributes: Vec<Attribute>,
    /// Index of the string on the main string table
    name: u32,
    /// Optional index of the namespace
    namespace: Option<u32>,
}

impl XmlTagStartBuf {
    pub fn new(name_index: u32, namespace: Option<u32>) -> Self {
        XmlTagStartBuf {
            attributes: Vec::new(),
            name: name_index,
            namespace: namespace,
        }
    }
}

impl TagStart for XmlTagStartBuf {
    fn get_tag_start(&self) -> Result<(Vec<Attribute>, Rc<String>)> {
        Err("Unimplemented!".into())
    }
}

impl OwnedBuf for XmlTagStartBuf {
    fn get_token(&self) -> u16 {
        TOKEN_XML_TAG_START
    }

    fn get_body_data(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();

        // Line
        out.write_u32::<LittleEndian>(0)?;

        // Field1
        out.write_u32::<LittleEndian>(0)?;

        // Namespace
        out.write_u32::<LittleEndian>(self.namespace.unwrap_or(0))?;

        // Element name
        out.write_u32::<LittleEndian>(self.name)?;

        // Field2
        out.write_u32::<LittleEndian>(0)?;

        // Num attributes
        out.write_u32::<LittleEndian>(self.attributes.len() as u32)?;

        // Get class
        out.write_u32::<LittleEndian>(0)?;

        // Field2
        out.write_u32::<LittleEndian>(0)?;

        // TODO: Start writing attributes

        Ok(out)
    }

    fn get_header_size(&self) -> u16 {
        8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_generate_an_empty_chunk() {
        let tag_start = XmlTagStartBuf::new(2, None);
        let out = tag_start.to_vec().unwrap();
        let expected = vec![2, 1, 8, 0, 40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            0, 0, 0, 0, 0, 0];

        assert_eq!(expected, out);
    }

    #[test]
    fn it_can_generate_a_chunk_with_the_given_data() {}

    #[test]
    fn identity() {}
}
