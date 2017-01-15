use chunks::*;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use document::{HeaderStringTable, Value, Attribute, Element};
use errors::*;
use std::clone::Clone;

pub struct XmlDecoder;

const TOKEN_VOID: u32 = 0xFFFFFFFF;

impl XmlDecoder {
    pub fn decode_xml_namespace_start<'a>(cursor: &mut Cursor<&'a [u8]>, header: &ChunkHeader)  -> Result<Chunk<'a>> {
        /*let _line = cursor.read_u32::<LittleEndian>()?;
        let _unknown = cursor.read_u32::<LittleEndian>()?;
        let prefix_idx = cursor.read_u32::<LittleEndian>()?;
        let uri_idx = cursor.read_u32::<LittleEndian>()?;

        let (prefix, uri) = {
            let st = decoder.get_string_table();
            let rc_st = match st {
                &Some(ref rc_st) => {rc_st.clone()},
                &None => {return Err("No string table found".into());}
            };

            let prefix = rc_st.get_string(prefix_idx as usize).unwrap().clone();
            let uri = rc_st.get_string(uri_idx as usize).unwrap().clone();

            (prefix, uri)
        };*/

        // decoder.push_namespace(prefix.clone(), uri.clone());

        let xnsw = XmlNamespaceStartWrapper::new(cursor.get_ref(), (*header).clone());
        Ok(Chunk::XmlNamespaceStart(xnsw))
        // Ok(Chunk::XmlStartNamespace(prefix, uri))
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
/*
     pub fn decode_xml_tag_start(mut decoder: &mut Decoder, cursor: &mut Cursor<&[u8]>, header: &ChunkHeader)  -> Result<Chunk> {
         let _line = cursor.read_u32::<LittleEndian>()?;
         let _unknown = cursor.read_u32::<LittleEndian>()?;
         let _ns_uri = cursor.read_u32::<LittleEndian>()?;
         let element_name_idx = cursor.read_u32::<LittleEndian>()?;
         let _unknwon2 = cursor.read_u32::<LittleEndian>()?;
         let attributes_amount = cursor.read_u32::<LittleEndian>()? as usize;
         let _unknwon3 = cursor.read_u32::<LittleEndian>()?;

         let (attributes, element_name) = {
             let st = decoder.get_string_table();
             let rc_st = match st {
                 &Some(ref rc_st) => {rc_st.clone()},
                 &None => {return Err("No string table found".into());}
             };
             let element_name = rc_st.get_string(element_name_idx as usize).unwrap().clone();
             let mut attributes = Vec::new();
             for _ in 0..attributes_amount {
                 let attribute = Self::decode_attribute(decoder, cursor, &rc_st)?;
                 attributes.push(attribute);
             }

             (attributes, element_name)
         };

         if attributes.len() != attributes_amount {
             return Err("Excptected a distinct amount of elements".into());
         }

         decoder.get_mut_element_container().start_element(Element::new(element_name.clone(), attributes));

         Ok(Chunk::XmlStartTag)
     }

     pub fn decode_xml_tag_end(mut decoder: &mut Decoder, cursor: &mut Cursor<&[u8]>, header: &ChunkHeader)  -> Result<Chunk> {
         decoder.get_mut_element_container().end_element();

         Ok(Chunk::XmlEndTag)
     }

     fn decode_attribute(decoder: &Decoder, cursor: &mut Cursor<&[u8]>, string_table: &StringTable) -> Result<Attribute> {
         let attr_ns_idx = cursor.read_u32::<LittleEndian>()?;
         let attr_name_idx = cursor.read_u32::<LittleEndian>()?;
         let attr_value_idx = cursor.read_u32::<LittleEndian>()?;
         let attr_type_idx = cursor.read_u32::<LittleEndian>()?;
         let attr_data = cursor.read_u32::<LittleEndian>()?;

         let mut namespace = None;
         let mut prefix = None;

         if attr_ns_idx != TOKEN_VOID {
             let uri = string_table.get_string(attr_ns_idx as usize).unwrap().clone();

             // TODO: Load namespace
             match decoder.get_namespaces().get(&uri) {
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
             Value::String(string_table.get_string(attr_value_idx as usize).unwrap().clone())
         };

         let element_name = string_table.get_string(attr_name_idx as usize).unwrap().clone();

         Ok(Attribute::new(element_name, value, namespace, prefix))
     }*/
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
        println!("PRefix index: {}", index);
        let string = string_table.get_string(index).unwrap();

        Ok(string)
    }

    pub fn get_namespace(&self, string_table: &mut StringTable) -> Result<Rc<String>> {
        let index = self.get_namespace_index();
        println!("Namespace index: {}", index);
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

    /*fn decode_namespace_start(&self, string_table: &mut StringTable) -> Result<(Vec<Attribute>, Rc<String>)> {
        let (attributes, element_name) = {
            let rc_st = match string_table {
                &Some(ref rc_st) => {rc_st.clone()},
                &None => {return Err("No string table found".into());}
            };

            let element_name = rc_st.get_string(element_name_idx as usize).unwrap().clone();
            let mut attributes = Vec::new();
            for _ in 0..attributes_amount {
                let attribute = self.decode_attribute(decoder, cursor, &rc_st)?;
                attributes.push(attribute);
            }

            (attributes, element_name)
        }
    }
    }*/

    /*fn decode_attribute(decoder: &Decoder, cursor: &mut Cursor<&[u8]>, string_table: &StringTable) -> Result<Attribute> {
        let attr_ns_idx = cursor.read_u32::<LittleEndian>()?;
        let attr_name_idx = cursor.read_u32::<LittleEndian>()?;
        let attr_value_idx = cursor.read_u32::<LittleEndian>()?;
        let attr_type_idx = cursor.read_u32::<LittleEndian>()?;
        let attr_data = cursor.read_u32::<LittleEndian>()?;

        let mut namespace = None;
        let mut prefix = None;

        if attr_ns_idx != TOKEN_VOID {
            let uri = string_table.get_string(attr_ns_idx as usize).unwrap().clone();

            // TODO: Load namespace
            match decoder.get_namespaces().get(&uri) {
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
            Value::String(string_table.get_string(attr_value_idx as usize).unwrap().clone())
        };

        let element_name = string_table.get_string(attr_name_idx as usize).unwrap().clone();

        Ok(Attribute::new(element_name, value, namespace, prefix))
    }*/
}

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
}

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
