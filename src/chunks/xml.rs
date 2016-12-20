use chunks::{Chunk, ChunkHeader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use document::{HeaderStringTable, StringTable, Value, Attribute};
use errors::*;
use parser::Decoder;
use std::clone::Clone;

pub struct XmlDecoder;

const TOKEN_VOID: u32 = 0xFFFFFFFF;

impl XmlDecoder {
    pub fn decode_xml_namespace_start(mut decoder: &mut Decoder, cursor: &mut Cursor<&[u8]>, header: &ChunkHeader)  -> Result<Chunk> {
        let _line = cursor.read_u32::<LittleEndian>()?;
        let _unknown = cursor.read_u32::<LittleEndian>()?;
        let prefix_idx = cursor.read_u32::<LittleEndian>()?;
        let uri_idx = cursor.read_u32::<LittleEndian>()?;

        let st = decoder.get_string_table();
        let rc_st = match st {
            &Some(ref rc_st) => {println!("Has string table"); rc_st.clone()},
            &None => {return Err("No string table found".into());}
        };

        let prefix = rc_st.get_string(prefix_idx as usize).unwrap().clone();
        let uri = rc_st.get_string(uri_idx as usize).unwrap().clone();

        Ok(Chunk::XmlStartNamespace(prefix, uri))
     }

     pub fn decode_xml_namespace_end(cursor: &mut Cursor<&[u8]>, header: &ChunkHeader) -> Result<Chunk> {
         Ok(Chunk::XmlEndNamespace)
     }

     pub fn decode_xml_tag_start(mut decoder: &mut Decoder, cursor: &mut Cursor<&[u8]>, header: &ChunkHeader)  -> Result<Chunk> {
         let _line = cursor.read_u32::<LittleEndian>()?;
         let _unknown = cursor.read_u32::<LittleEndian>()?;
         let _ns_uri = cursor.read_u32::<LittleEndian>()?;
         let element_name_idx = cursor.read_u32::<LittleEndian>()?;
         let _unknwon2 = cursor.read_u32::<LittleEndian>()?;
         let attributes_amount = cursor.read_u32::<LittleEndian>()? as usize;
         let _unknwon3 = cursor.read_u32::<LittleEndian>()?;

         let st = decoder.get_string_table();
         let rc_st = match st {
             &Some(ref rc_st) => {println!("Has string table"); rc_st.clone()},
             &None => {return Err("No string table found".into());}
         };
         let element_name = rc_st.get_string(element_name_idx as usize).unwrap().clone();
         let mut attributes = Vec::new();
         for _ in 0..attributes_amount {
             let attribute = Self::decode_attribute(cursor, &rc_st);
             attributes.push(attribute);
         }

         if attributes.len() != attributes_amount {
             return Err("Excptected a distinct amount of elements".into());
         }

         // element_container.start_element(Element::new(element_name.clone(), attributes));

         Ok(Chunk::XmlStartTag)
     }

     fn decode_attribute(cursor: &mut Cursor<&[u8]>, string_table: &StringTable) -> Result<Attribute> {
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
             /*match document.resources.get(&uri) {
                 Some(uri_prefix) => {
                     namespace = Some(uri);
                     prefix = Some(uri_prefix.clone());
                 }
                 None =>(),
             };*/
         }

         let value = if attr_value_idx == TOKEN_VOID {
             Value::new(attr_type_idx as u8, attr_data, string_table)?
         } else {
             Value::String(string_table.get_string(attr_value_idx as usize).unwrap().clone())
         };

         let element_name = string_table.get_string(attr_name_idx as usize).unwrap().clone();

         Ok(Attribute::new(element_name, value, namespace, prefix))
     }
}
