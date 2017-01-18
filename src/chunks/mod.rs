use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;

pub mod string_table;
mod chunk_header;
mod package;
pub mod table_type;
mod resource;
mod table_type_spec;
mod xml;

pub use self::string_table::StringTableDecoder as StringTableDecoder;
pub use self::string_table::StringTableWrapper as StringTableWrapper;
pub use self::string_table::StringTable as StringTable;
pub use self::chunk_header::ChunkHeader as ChunkHeader;
pub use self::package::PackageWrapper as PackageWrapper;
pub use self::package::Package as Package;
pub use self::table_type_spec::TypeSpecWrapper as TypeSpecWrapper;
pub use self::table_type_spec::TypeSpec as TypeSpec;
pub use self::table_type::TableTypeWrapper as TableTypeWrapper;
pub use self::table_type::TableType as TableType;
pub use self::table_type::Entry as Entry;

pub use self::resource::ResourceWrapper as ResourceWrapper;
pub use self::resource::Resource as Resource;
pub use self::xml::XmlNamespaceStart as XmlNamespaceStart;
pub use self::xml::XmlNamespaceStartWrapper as XmlNamespaceStartWrapper;
pub use self::xml::XmlNamespaceEnd as XmlNamespaceEnd;
pub use self::xml::XmlNamespaceEndWrapper as XmlNamespaceEndWrapper;
pub use self::xml::XmlTagStart as XmlTagStart;
pub use self::xml::XmlTagStartWrapper as XmlTagStartWrapper;
pub use self::xml::XmlTagEnd as XmlTagEnd;
pub use self::xml::XmlTagEndWrapper as XmlTagEndWrapper;

use self::package::PackageDecoder;
use self::table_type_spec::TableTypeSpecDecoder;
use self::table_type::TableTypeDecoder;
use self::xml::XmlDecoder;
use self::resource::ResourceDecoder;

use errors::*;
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

pub enum Chunk<'a>   {
    StringTable(StringTableWrapper<'a>),
    Package(PackageWrapper<'a>),
    TableTypeSpec(TypeSpecWrapper<'a>),
    TableType(TableTypeWrapper<'a>),
    XmlNamespaceStart(XmlNamespaceStartWrapper<'a>),
    XmlNamespaceEnd(XmlNamespaceEndWrapper<'a>),
    XmlTagStart(XmlTagStartWrapper<'a>),
    XmlTagEnd(XmlTagEndWrapper<'a>),
    Resource(ResourceWrapper<'a>),
    Unknown,
}

pub struct ChunkLoaderStream<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> ChunkLoaderStream<'a> {
    pub fn new(cursor: Cursor<&'a [u8]>) -> Self {
        ChunkLoaderStream {
            cursor: cursor,
        }
    }

    fn read_one(&mut self) -> Result<Chunk<'a>> {
        let initial_position = self.cursor.position();
        let token = self.cursor.read_u16::<LittleEndian>()?;
        let header_size = self.cursor.read_u16::<LittleEndian>()?;
        let chunk_size = self.cursor.read_u32::<LittleEndian>()?;
        let chunk_header = ChunkHeader::new(initial_position, header_size, chunk_size, token);

        let chunk = match token {
            TOKEN_STRING_TABLE => StringTableDecoder::decode(&mut self.cursor, &chunk_header)?,
            TOKEN_PACKAGE => PackageDecoder::decode(&mut self.cursor, &chunk_header)?,
            TOKEN_TABLE_SPEC => TableTypeSpecDecoder::decode(&mut self.cursor, &chunk_header)?,
            TOKEN_TABLE_TYPE => TableTypeDecoder::decode(&mut self.cursor, &chunk_header)?,
            TOKEN_XML_START_NAMESPACE => XmlDecoder::decode_xml_namespace_start(&mut self.cursor, &chunk_header)?,
            TOKEN_XML_END_NAMESPACE => XmlDecoder::decode_xml_namespace_end(&mut self.cursor, &chunk_header)?,
            TOKEN_XML_TAG_START => XmlDecoder::decode_xml_tag_start(&mut self.cursor, &chunk_header)?,
            TOKEN_XML_TAG_END => XmlDecoder::decode_xml_tag_end(&mut self.cursor, &chunk_header)?,
            TOKEN_RESOURCE => ResourceDecoder::decode(&mut self.cursor, &chunk_header)?,
            t => {
                println!("{:X}", t);

                Chunk::Unknown
            },
        };

        if let Chunk::Package(_) = chunk {
            self.cursor.set_position(chunk_header.get_data_offset());
        } else {
            self.cursor.set_position(chunk_header.get_chunk_end());
        }
        Ok(chunk)
    }
}

impl<'a> Iterator for ChunkLoaderStream<'a> {
    type Item = Result<Chunk<'a>>;

    fn next(&mut self) -> Option<Result<Chunk<'a>>> {
        if self.cursor.position() == self.cursor.get_ref().len() as u64 {
            return None;
        }

        Some(self.read_one())
    }
}
