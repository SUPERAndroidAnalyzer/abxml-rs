use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};

pub mod string_table;
mod chunk_header;
mod package;
pub mod table_type;
mod resource;
mod table_type_spec;
mod xml;

pub use self::string_table::StringTableDecoder;
pub use self::string_table::StringTableWrapper;
pub use self::chunk_header::ChunkHeader;
pub use self::package::PackageWrapper;
pub use self::table_type_spec::TypeSpecWrapper;
pub use self::table_type::TableTypeWrapper;
pub use self::table_type::ConfigurationWrapper;

pub use self::resource::ResourceWrapper;
pub use self::xml::XmlNamespaceStartWrapper;
pub use self::xml::XmlNamespaceEndWrapper;
pub use self::xml::XmlTagStartWrapper;
pub use self::xml::XmlTagEndWrapper;
pub use self::xml::XmlText;
pub use self::xml::XmlTextWrapper;

use self::package::PackageDecoder;
use self::table_type_spec::TableTypeSpecDecoder;
use self::table_type::TableTypeDecoder;
use self::xml::XmlDecoder;
use self::resource::ResourceDecoder;

use errors::*;

pub const TOKEN_STRING_TABLE: u16 = 0x0001;
pub const TOKEN_RESOURCE: u16 = 0x0180;
pub const TOKEN_PACKAGE: u16 = 0x0200;
pub const TOKEN_TABLE_TYPE: u16 = 0x201;
pub const TOKEN_TABLE_SPEC: u16 = 0x202;
pub const TOKEN_XML_START_NAMESPACE: u16 = 0x100;
pub const TOKEN_XML_END_NAMESPACE: u16 = 0x101;
pub const TOKEN_XML_TAG_START: u16 = 0x102;
pub const TOKEN_XML_TAG_END: u16 = 0x103;
pub const TOKEN_XML_TEXT: u16 = 0x104;

pub enum Chunk<'a> {
    StringTable(StringTableWrapper<'a>),
    Package(PackageWrapper<'a>),
    TableTypeSpec(TypeSpecWrapper<'a>),
    TableType(TableTypeWrapper<'a>),
    XmlNamespaceStart(XmlNamespaceStartWrapper<'a>),
    XmlNamespaceEnd(XmlNamespaceEndWrapper<'a>),
    XmlTagStart(XmlTagStartWrapper<'a>),
    XmlTagEnd(XmlTagEndWrapper<'a>),
    XmlText(XmlTextWrapper<'a>),
    Resource(ResourceWrapper<'a>),
    Unknown,
}

pub struct ChunkLoaderStream<'a> {
    cursor: Cursor<&'a [u8]>,
    previous: Option<u64>,
}

impl<'a> ChunkLoaderStream<'a> {
    pub fn new(cursor: Cursor<&'a [u8]>) -> Self {
        ChunkLoaderStream {
            cursor: cursor,
            previous: None,
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
            TOKEN_XML_START_NAMESPACE => {
                XmlDecoder::decode_xml_namespace_start(&mut self.cursor, &chunk_header)?
            }
            TOKEN_XML_END_NAMESPACE => {
                XmlDecoder::decode_xml_namespace_end(&mut self.cursor, &chunk_header)?
            }
            TOKEN_XML_TAG_START => {
                XmlDecoder::decode_xml_tag_start(&mut self.cursor, &chunk_header)?
            }
            TOKEN_XML_TAG_END => XmlDecoder::decode_xml_tag_end(&mut self.cursor, &chunk_header)?,
            TOKEN_XML_TEXT => XmlDecoder::decode_xml_text(&mut self.cursor, &chunk_header)?,
            TOKEN_RESOURCE => ResourceDecoder::decode(&mut self.cursor, &chunk_header)?,
            t => {
                error!("Unknown chunk: 0x{:X}", t);

                Chunk::Unknown
            }
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
        if let Some(prev) = self.previous {
            if prev == self.cursor.position() {
                return None;
            }
        }

        if self.cursor.position() >= self.cursor.get_ref().len() as u64 {
            return None;
        }

        self.previous = Some(self.cursor.position());
        let chunk = self.read_one();

        Some(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use model::owned::{StringTableBuf, ResourcesBuf, OwnedBuf};

    #[test]
    fn it_can_detect_loops() {
        let data = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let cursor: Cursor<&[u8]> = Cursor::new(&data);
        let mut stream = ChunkLoaderStream::new(cursor);

        let _ = stream.next().unwrap();
        assert!(stream.next().is_none());
    }

    #[test]
    fn it_stops_the_iteration_if_out_of_bounds() {
        let data = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut cursor: Cursor<&[u8]> = Cursor::new(&data);
        cursor.set_position(30);
        let mut stream = ChunkLoaderStream::new(cursor);

        assert!(stream.next().is_none());
    }

    #[test]
    fn it_can_iterate_over_chunks() {
        let mut data = vec![0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let st = StringTableBuf::default();
        data.extend(st.to_vec().unwrap());
        let res = ResourcesBuf::default();
        data.extend(res.to_vec().unwrap());

        let mut cursor: Cursor<&[u8]> = Cursor::new(&data);
        // Skip header
        cursor.set_position(12);
        let mut stream = ChunkLoaderStream::new(cursor);

        // Assert string table
        let _ = stream.next().unwrap();
        let _ = stream.next().unwrap();

        assert!(stream.next().is_none());
    }
}
