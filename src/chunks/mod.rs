use std::io::Error;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use document::StringTable;

pub mod string_table;
pub mod package;
mod chunk_header;
mod table_type;

pub use self::string_table::StringTableDecoder as StringTableDecoder;
pub use self::package::PackageDecoder as PackageDecoder;
pub use self::chunk_header::ChunkHeader as ChunkHeader;
pub use self::table_type::TableTypeDecoder as TableTypeDecoder;

use self::table_type::ResourceConfiguration;

const TOKEN_STRING_TABLE: u16 = 0x0001;
const TOKEN_PACKAGE: u16 = 0x0200;
const TOKEN_TABLE_TYPE: u16 = 0x201;
const TOKEN_TABLE_SPEC: u16 = 0x202;

#[derive(Debug)]
pub enum Chunk {
    StringTable(StringTable),
    Package,
    TableType(Box<ResourceConfiguration>),
    Unknown,
}

pub struct ChunkLoader;

impl ChunkLoader {
    pub fn read_all<'a>(mut cursor: &mut Cursor<&'a [u8]>, ending: u64) -> Result<Vec<Chunk>, Error> {
        let mut chunks = Vec::new();

        // Loop trough all of the frames
        loop {
             let initial_position = cursor.position();
             if initial_position == ending as u64 {
                 // We are at the end of the document. We are done!
                 break;
             }

             let token = cursor.read_u16::<LittleEndian>()?;
             let header_size = cursor.read_u16::<LittleEndian>()?;
             let chunk_size = cursor.read_u32::<LittleEndian>()?;
             let chunk_header = ChunkHeader::new(initial_position, header_size, chunk_size, token);

             let chunk = match token {
                 TOKEN_STRING_TABLE => StringTableDecoder::decode(&mut cursor, &chunk_header)?,
                 TOKEN_PACKAGE => PackageDecoder::decode(&mut cursor, &chunk_header)?,
                 TOKEN_TABLE_TYPE => TableTypeDecoder::decode(&mut cursor, &chunk_header)?,
                 t => {
                     println!("{:X}", t);

                     Chunk::Unknown
                 },
             };

             chunks.push(chunk);

             cursor.set_position(chunk_header.get_chunk_end());
        }

        Ok(chunks)
    }
}
