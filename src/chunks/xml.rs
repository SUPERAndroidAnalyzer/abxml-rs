use chunks::{Chunk, ChunkHeader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use document::{HeaderStringTable, StringTable};
use errors::*;
use parser::Decoder;
use std::clone::Clone;

pub struct XmlDecoder;

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
}
