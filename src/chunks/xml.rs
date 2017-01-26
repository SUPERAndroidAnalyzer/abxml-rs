use chunks::*;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use document::{Value, Attribute, Namespaces};
use errors::*;
use std::clone::Clone;

pub struct XmlDecoder;

const TOKEN_VOID: u32 = 0xFFFFFFFF;

impl XmlDecoder {
    pub fn decode_xml_namespace_start<'a>(cursor: &mut Cursor<&'a [u8]>, header: &ChunkHeader)  -> Result<Chunk<'a>> {
        let xnsw = XmlNamespaceStartWrapper::new(cursor.get_ref(), (*header).clone());
        Ok(Chunk::XmlNamespaceStart(xnsw))
     }

     pub fn decode_xml_namespace_end<'a>(cursor: &mut Cursor<&'a [u8]>, header: &ChunkHeader) -> Result<Chunk<'a>> {
         let xnsw = XmlNamespaceEndWrapper::new(cursor.get_ref(), (*header).clone());
         Ok(Chunk::XmlNamespaceEnd(xnsw))
     }

     pub fn decode_xml_tag_start<'a>(cursor: &mut Cursor<&'a [u8]>, header: &ChunkHeader) -> Result<Chunk<'a>> {
         let xnsw = XmlTagStartWrapper::new(cursor.get_ref(), (*header).clone());
         Ok(Chunk::XmlTagStart(xnsw))
     }

    pub fn decode_xml_tag_end<'a>(cursor: &mut Cursor<&'a [u8]>, header: &ChunkHeader) -> Result<Chunk<'a>> {
        let xnsw = XmlTagEndWrapper::new(cursor.get_ref(), (*header).clone());
        Ok(Chunk::XmlTagEnd(xnsw))
    }

    pub fn decode_xml_text<'a>(cursor: &mut Cursor<&'a [u8]>, header: &ChunkHeader) -> Result<Chunk<'a>> {
        let xnsw = XmlTextWrapper::new(cursor.get_ref(), (*header).clone());
        Ok(Chunk::XmlText(xnsw))
    }
}

pub struct XmlNamespaceStartWrapper<'a> {
    raw_data: &'a [u8],
    header: ChunkHeader,
}

impl<'a> XmlNamespaceStartWrapper<'a> {
    pub fn new(raw_data: &'a [u8], header: ChunkHeader) -> Self {
        XmlNamespaceStartWrapper {
            raw_data: raw_data,
            header: header,
        }
    }

    pub fn get_prefix_index(&self) -> u32 {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(16));

        cursor.read_u32::<LittleEndian>().unwrap()
    }

    pub fn get_namespace_index(&self) -> u32 {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(20));

        cursor.read_u32::<LittleEndian>().unwrap()
    }

    pub fn get_prefix(&self, string_table: &mut StringTable) -> Result<Rc<String>> {
        let index = self.get_prefix_index();
        let string = string_table.get_string(index).unwrap();

        Ok(string)
    }

    pub fn get_namespace(&self, string_table: &mut StringTable) -> Result<Rc<String>> {
        let index = self.get_namespace_index();
        let string = string_table.get_string(index).unwrap();

        Ok(string)
    }

}

pub struct XmlNamespaceStart<'a> {
    wrapper: XmlNamespaceStartWrapper<'a>,
}

impl<'a> XmlNamespaceStart<'a> {
    pub fn new(wrapper: XmlNamespaceStartWrapper<'a>) -> Self {
        XmlNamespaceStart {
            wrapper: wrapper,
        }
    }

    pub fn get_prefix(&self, string_table: &mut StringTable) -> Result<Rc<String>> {
        self.wrapper.get_prefix(string_table)
    }

    pub fn get_namespace(&self, string_table: &mut StringTable) -> Result<Rc<String>> {
        self.wrapper.get_namespace(string_table)
    }
}

#[allow(dead_code)]
pub struct XmlNamespaceEndWrapper<'a> {
    raw_data: &'a [u8],
    header: ChunkHeader,
}

impl<'a> XmlNamespaceEndWrapper<'a> {
    pub fn new(raw_data: &'a [u8], header: ChunkHeader) -> Self {
        XmlNamespaceEndWrapper {
            raw_data: raw_data,
            header: header,
        }
    }
}

#[allow(dead_code)]
pub struct XmlNamespaceEnd<'a> {
    wrapper: XmlNamespaceEndWrapper<'a>,
}

impl<'a> XmlNamespaceEnd<'a> {
    pub fn new(wrapper: XmlNamespaceEndWrapper<'a>) -> Self {
        XmlNamespaceEnd {
            wrapper: wrapper,
        }
    }
}

pub struct XmlTagStartWrapper<'a> {
    raw_data: &'a [u8],
    header: ChunkHeader,
}

impl<'a> XmlTagStartWrapper<'a> {
    pub fn new(raw_data: &'a [u8], header: ChunkHeader) -> Self {
        XmlTagStartWrapper {
            raw_data: raw_data,
            header: header,
        }
    }

    pub fn get_line(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(8));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get line")
    }

    pub fn get_field1(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(12));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }

    pub fn get_ns_uri(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(16));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }

    pub fn get_element_name_index(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(20));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }

    pub fn get_field2(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(24));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }

    pub fn get_attributes_amount(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(28));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }

    pub fn get_field3(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(32));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }

    pub fn get_tag_start(&self, namespaces: &Namespaces, string_table: &mut StringTable) -> Result<(Vec<Attribute>, Rc<String>)> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(36));

        let element_name = string_table.get_string(self.get_element_name_index()?)?;

        let mut attributes = Vec::new();
        for _ in 0..self.get_attributes_amount()? {
            let attribute = self.decode_attribute(&mut cursor, &namespaces, string_table)?;
            attributes.push(attribute);
        }

        if attributes.len() != self.get_attributes_amount()? as usize {
            return Err("Excptected a distinct amount of elements".into());
        }

        Ok((attributes, element_name))
    }

    fn decode_attribute(&self, cursor: &mut Cursor<&[u8]>, namespaces: &Namespaces, string_table: &mut StringTable) -> Result<Attribute> {
        let attr_ns_idx = cursor.read_u32::<LittleEndian>()?;
        let attr_name_idx = cursor.read_u32::<LittleEndian>()?;
        let attr_value_idx = cursor.read_u32::<LittleEndian>()?;

        let _size = cursor.read_u16::<LittleEndian>()?;
        cursor.read_u8()?;
        let attr_type_idx = cursor.read_u8()?;
        let attr_data = cursor.read_u32::<LittleEndian>()?;

        let mut namespace = None;
        let mut prefix = None;

        if attr_ns_idx != TOKEN_VOID {
            let uri = string_table.get_string(attr_ns_idx).unwrap().clone();

            //println!("uri: {}, Namespaces: {:?}", uri, namespaces);
            //panic!("");
            // TODO: Load namespace
            match namespaces.get(&uri) {
                Some(uri_prefix) => {
                    namespace = Some(uri);
                    prefix = Some(uri_prefix.clone());
                }
                None =>(),
            };
        }

        let value = if attr_value_idx == TOKEN_VOID {
            Value::new(attr_type_idx as u8, attr_data, string_table)?
        } else {
            Value::String(string_table.get_string(attr_value_idx).unwrap().clone())
        };

        let element_name = string_table.get_string(attr_name_idx).unwrap().clone();

        Ok(Attribute::new(element_name, value, namespace, prefix))
    }
}

pub struct XmlTagStart<'a> {
    wrapper: XmlTagStartWrapper<'a>,
}

impl<'a> XmlTagStart<'a> {
    pub fn new(wrapper: XmlTagStartWrapper<'a>) -> Self {
        XmlTagStart {
            wrapper: wrapper,
        }
    }

    pub fn get_tag(&self, namespaces: &Namespaces, string_table: &mut StringTable) -> Result<(Vec<Attribute>, Rc<String>)> {
        self.wrapper.get_tag_start(namespaces, string_table)
    }
}

#[allow(dead_code)]
pub struct XmlTagEndWrapper<'a> {
    raw_data: &'a [u8],
    header: ChunkHeader,
}

impl<'a> XmlTagEndWrapper<'a> {
    pub fn new(raw_data: &'a [u8], header: ChunkHeader) -> Self {
        XmlTagEndWrapper {
            raw_data: raw_data,
            header: header,
        }
    }
}

#[allow(dead_code)]
pub struct XmlTagEnd<'a> {
    wrapper: XmlTagEndWrapper<'a>,
}

impl<'a> XmlTagEnd<'a> {
    pub fn new(wrapper: XmlTagEndWrapper<'a>) -> Self {
        XmlTagEnd {
            wrapper: wrapper,
        }
    }
}

#[allow(dead_code)]
pub struct XmlTextWrapper<'a> {
    raw_data: &'a [u8],
    header: ChunkHeader,
}

impl<'a> XmlTextWrapper<'a> {
    pub fn new(raw_data: &'a [u8], header: ChunkHeader) -> Self {
        XmlTextWrapper {
            raw_data: raw_data,
            header: header,
        }
    }

    pub fn get_text_index(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(16));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }
}

#[allow(dead_code)]
pub struct XmlText<'a> {
    wrapper: XmlTextWrapper<'a>,
}

impl<'a> XmlText<'a> {
    pub fn new(wrapper: XmlTextWrapper<'a>) -> Self {
        XmlText {
            wrapper: wrapper,
        }
    }

    pub fn get_text(&self, string_table: &StringTable) -> Result<Rc<String>> {
        let index = self.wrapper.get_text_index()?;
        // println!("{}", string_table);
        // println!("TARGET STR: {} {}", index, string_table.get_uncached_string(index)?);
        string_table.get_uncached_string(index)
    }
}