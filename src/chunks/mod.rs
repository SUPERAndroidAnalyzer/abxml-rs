//! Structs to represent chunks and iterate them
use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};
use failure::Error;

mod chunk_header;
mod package;
mod resource;
pub mod string_table;
pub mod table_type;
mod table_type_spec;
mod xml;

pub use self::{
    chunk_header::ChunkHeader,
    package::PackageWrapper,
    resource::ResourceWrapper,
    string_table::{StringTableCache, StringTableWrapper},
    table_type::{ConfigurationWrapper, TableTypeWrapper},
    table_type_spec::TypeSpecWrapper,
    xml::{
        XmlNamespaceEndWrapper, XmlNamespaceStartWrapper, XmlTagEndWrapper, XmlTagStartWrapper,
        XmlTextWrapper,
    },
};

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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct ChunkLoaderStream<'a> {
    cursor: Cursor<&'a [u8]>,
    previous: Option<u64>,
}

impl<'a> ChunkLoaderStream<'a> {
    pub fn new(cursor: Cursor<&'a [u8]>) -> Self {
        Self {
            cursor,
            previous: None,
        }
    }

    fn read_one(&mut self) -> Result<Chunk<'a>, Error> {
        let initial_position = self.cursor.position();
        let token = self.cursor.read_u16::<LittleEndian>()?;
        let header_size = self.cursor.read_u16::<LittleEndian>()?;
        let chunk_size = self.cursor.read_u32::<LittleEndian>()?;
        let chunk_header = ChunkHeader::new(initial_position, header_size, chunk_size, token);

        let chunk = self.get_chunk(&chunk_header);

        if let Chunk::Package(_) = chunk {
            self.cursor.set_position(chunk_header.get_data_offset());
        } else {
            self.cursor.set_position(chunk_header.get_chunk_end());
        }

        Ok(chunk)
    }

    fn get_chunk(&self, header: &ChunkHeader) -> Chunk<'a> {
        let raw_data = self.cursor.get_ref();
        let slice = &raw_data[header.get_offset() as usize..header.get_chunk_end() as usize];

        match header.get_token() {
            TOKEN_STRING_TABLE => Chunk::StringTable(StringTableWrapper::new(slice)),
            TOKEN_PACKAGE => Chunk::Package(PackageWrapper::new(slice)),
            TOKEN_TABLE_SPEC => Chunk::TableTypeSpec(TypeSpecWrapper::new(slice)),
            TOKEN_TABLE_TYPE => {
                let current_chunk_data_offset = header.get_data_offset() - header.get_offset();
                Chunk::TableType(TableTypeWrapper::new(slice, current_chunk_data_offset))
            }
            TOKEN_XML_START_NAMESPACE => {
                Chunk::XmlNamespaceStart(XmlNamespaceStartWrapper::new(slice))
            }
            TOKEN_XML_END_NAMESPACE => Chunk::XmlNamespaceEnd(XmlNamespaceEndWrapper::new(slice)),
            TOKEN_XML_TAG_START => Chunk::XmlTagStart(XmlTagStartWrapper::new(slice)),
            TOKEN_XML_TAG_END => Chunk::XmlTagEnd(XmlTagEndWrapper::new(slice)),
            TOKEN_XML_TEXT => Chunk::XmlText(XmlTextWrapper::new(slice)),
            TOKEN_RESOURCE => Chunk::Resource(ResourceWrapper::new(slice)),
            t => {
                error!("Unknown chunk: 0x{:X}", t);

                Chunk::Unknown
            }
        }
    }
}

impl<'a> Iterator for ChunkLoaderStream<'a> {
    type Item = Result<Chunk<'a>, Error>;

    fn next(&mut self) -> Option<Result<Chunk<'a>, Error>> {
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
    use model::owned::{OwnedBuf, ResourcesBuf, StringTableBuf};
    use std::io::Cursor;

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
