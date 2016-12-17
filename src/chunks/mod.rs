use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use document::StringTable;

pub mod string_table;
pub mod package;
mod chunk_header;
mod table_type;
mod table_type_spec;

pub use self::string_table::StringTableDecoder as StringTableDecoder;
pub use self::package::PackageDecoder as PackageDecoder;
pub use self::chunk_header::ChunkHeader as ChunkHeader;
pub use self::table_type::TableTypeDecoder as TableTypeDecoder;
pub use self::table_type_spec::TableTypeSpecDecoder as TableTypeSpecDecoder;

use self::table_type::{Entry, ResourceConfiguration};
use errors::*;

const TOKEN_STRING_TABLE: u16 = 0x0001;
const TOKEN_PACKAGE: u16 = 0x0200;
const TOKEN_TABLE_TYPE: u16 = 0x201;
const TOKEN_TABLE_SPEC: u16 = 0x202;

#[derive(Debug)]
pub enum Chunk {
    StringTable(StringTable),
    Package,
    TableType(u8, Box<ResourceConfiguration>, Vec<Entry>),
    TableTypeSpec(u32, Vec<u32>),
    Unknown,
}

pub struct ChunkLoader;

impl ChunkLoader {
    pub fn read_all<'a>(mut cursor: &mut Cursor<&'a [u8]>, ending: u64) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();

        // Loop trough all of the frames
        loop {
             if cursor.position() == ending {
                 break;
             }

             let chunk = Self::read(&mut cursor)?;
             chunks.push(chunk);
        }

        Ok(chunks)
    }

    pub fn read<'a>(mut cursor: &mut Cursor<&'a [u8]>) -> Result<Chunk> {
         let initial_position = cursor.position();
         let token = cursor.read_u16::<LittleEndian>()?;
         let header_size = cursor.read_u16::<LittleEndian>()?;
         let chunk_size = cursor.read_u32::<LittleEndian>()?;
         let chunk_header = ChunkHeader::new(initial_position, header_size, chunk_size, token);

         let chunk = match token {
             TOKEN_STRING_TABLE => StringTableDecoder::decode(&mut cursor, &chunk_header)?,
             TOKEN_PACKAGE => PackageDecoder::decode(&mut cursor, &chunk_header)?,
             TOKEN_TABLE_TYPE => TableTypeDecoder::decode(&mut cursor, &chunk_header)?,
             TOKEN_TABLE_SPEC => TableTypeSpecDecoder::decode(&mut cursor, &chunk_header)?,
             t => {
                 println!("{:X}", t);

                 Chunk::Unknown
             },
         };

         cursor.set_position(chunk_header.get_chunk_end());
         Ok(chunk)
    }
}
