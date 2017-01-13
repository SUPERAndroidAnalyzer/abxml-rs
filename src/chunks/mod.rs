use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use document::StringTable;

pub mod string_table;
pub mod package;
mod chunk_header;
mod table_type;
mod table_type_spec;
mod resource;
mod xml;

pub use self::string_table::StringTableDecoder as StringTableDecoder;
pub use self::package::PackageDecoder as PackageDecoder;
pub use self::chunk_header::ChunkHeader as ChunkHeader;
pub use self::table_type::TableTypeDecoder as TableTypeDecoder;
pub use self::table_type_spec::TableTypeSpecDecoder as TableTypeSpecDecoder;
pub use self::resource::ResourceDecoder as ResourceDecoder;
pub use self::xml::XmlDecoder as XmlDecoder;

use self::table_type::{Entry, ResourceConfiguration};
use errors::*;
use parser::Decoder;
use std::rc::Rc;

const TOKEN_STRING_TABLE: u16 = 0x0001;
const TOKEN_RESOURCE: u16 = 0x0180;
const TOKEN_PACKAGE: u16 = 0x0200;
const TOKEN_TABLE_TYPE: u16 = 0x201;
const TOKEN_TABLE_SPEC: u16 = 0x202;
const TOKEN_XML_START_NAMESPACE: u16 = 0x100;
const TOKEN_XML_END_NAMESPACE: u16 = 0x101;
const TOKEN_XML_TAG_START: u16 = 0x102;
const TOKEN_XML_TAG_END: u16 = 0x103;

#[derive(Debug)]
pub enum Chunk   {
    StringTable(Rc<StringTable>),
    Package,
    TableType(u8, Box<ResourceConfiguration>, Vec<Entry>),
    TableTypeSpec(u32, Vec<u32>),
    ResourceTable(Vec<u32>),
    XmlStartNamespace(Rc<String>, Rc<String>),
    XmlEndNamespace,
    XmlStartTag,
    XmlEndTag,
    Unknown,
}

pub struct ChunkLoader;

impl ChunkLoader {
    pub fn read_all<'a>(mut decoder: &mut Decoder, mut cursor: &mut Cursor<&'a [u8]>, ending: u64) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();

        // Loop trough all of the frames
        loop {
             if cursor.position() == ending {
                 break;
             }

             let chunk = Self::read(decoder, &mut cursor)?;
             chunks.push(chunk);
        }

        Ok(chunks)
    }

    pub fn read<'a>(mut decoder: &mut Decoder, mut cursor: &mut Cursor<&'a [u8]>) -> Result<Chunk> {
         let initial_position = cursor.position();
         let token = cursor.read_u16::<LittleEndian>()?;
         let header_size = cursor.read_u16::<LittleEndian>()?;
         let chunk_size = cursor.read_u32::<LittleEndian>()?;
         let chunk_header = ChunkHeader::new(initial_position, header_size, chunk_size, token);

         let chunk = match token {
             TOKEN_STRING_TABLE => StringTableDecoder::decode(&mut cursor, &chunk_header)?,
             TOKEN_PACKAGE => PackageDecoder::decode(&mut cursor, &chunk_header)?,
             TOKEN_TABLE_TYPE => TableTypeDecoder::decode(decoder, &mut cursor, &chunk_header)?,
             TOKEN_TABLE_SPEC => TableTypeSpecDecoder::decode(decoder, &mut cursor, &chunk_header)?,
             TOKEN_RESOURCE => ResourceDecoder::decode(&mut cursor, &chunk_header)?,
             TOKEN_XML_START_NAMESPACE => XmlDecoder::decode_xml_namespace_start(decoder, &mut cursor, &chunk_header)?,
             TOKEN_XML_END_NAMESPACE => XmlDecoder::decode_xml_namespace_end(&mut cursor, &chunk_header)?,
             TOKEN_XML_TAG_START => XmlDecoder::decode_xml_tag_start(decoder, &mut cursor, &chunk_header)?,
             TOKEN_XML_TAG_END => XmlDecoder::decode_xml_tag_end(decoder, &mut cursor, &chunk_header)?,
             t => {
                 println!("{:X}", t);

                 Chunk::Unknown
             },
         };

         cursor.set_position(chunk_header.get_chunk_end());
         Ok(chunk)
    }
}

pub struct ChunkLoaderStream<'a> {
    cursor: Cursor<&'a [u8]>,
    iterator: Option<Box<Iterator<Item = Chunk>>>,
}

impl<'a> ChunkLoaderStream<'a> {
    pub fn new(cursor: Cursor<&'a [u8]>) -> Self {
        ChunkLoaderStream {
            cursor: cursor,
            iterator: None,
        }
    }

    fn read_one(&mut self) -> Result<Chunk> {
        let initial_position = self.cursor.position();
        println!("Initial position: {}", initial_position);
        let token = self.cursor.read_u16::<LittleEndian>()?;
        let header_size = self.cursor.read_u16::<LittleEndian>()?;
        let chunk_size = self.cursor.read_u32::<LittleEndian>()?;
        let chunk_header = ChunkHeader::new(initial_position, header_size, chunk_size, token);
        println!("Chunk end: {}", chunk_header.get_chunk_end());

        let chunk = match token {
            // TOKEN_STRING_TABLE => StringTableDecoder::decode(&mut self.cursor, &chunk_header)?,
            TOKEN_PACKAGE => PackageDecoder::decode(&mut self.cursor, &chunk_header)?,
            /*TOKEN_PACKAGE => PackageDecoder::decode(decoder, &mut cursor, &chunk_header)?,
            TOKEN_TABLE_TYPE => TableTypeDecoder::decode(decoder, &mut cursor, &chunk_header)?,
            TOKEN_TABLE_SPEC => TableTypeSpecDecoder::decode(decoder, &mut cursor, &chunk_header)?,
            TOKEN_RESOURCE => ResourceDecoder::decode(&mut cursor, &chunk_header)?,
            TOKEN_XML_START_NAMESPACE => XmlDecoder::decode_xml_namespace_start(decoder, &mut cursor, &chunk_header)?,
            TOKEN_XML_END_NAMESPACE => XmlDecoder::decode_xml_namespace_end(&mut cursor, &chunk_header)?,
            TOKEN_XML_TAG_START => XmlDecoder::decode_xml_tag_start(decoder, &mut cursor, &chunk_header)?,
            TOKEN_XML_TAG_END => XmlDecoder::decode_xml_tag_end(decoder, &mut cursor, &chunk_header)?,*/
            t => {
                println!("{:X}", t);

                Chunk::Unknown
            },
        };

        if let Chunk::Package = chunk {
            // In case of package, we set the next position of the cursor inside the decoder
        } else {
            self.cursor.set_position(chunk_header.get_chunk_end());
        }
        Ok(chunk)
    }
}

impl<'a> Iterator for ChunkLoaderStream<'a> {
    type Item = Chunk;

    fn next(&mut self) -> Option<Chunk> {
        if self.cursor.position() == self.cursor.get_ref().len() as u64 {
            return None;
        }

        match self.read_one() {
            Ok(c) => {
                Some(c)
            },
            Err(e) => {
                error!("Error reading one of the chunks");
                println!("Error: {}", e);
                None
            }
        }
    }
}
