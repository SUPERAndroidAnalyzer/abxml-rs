use std::io::Error;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use document::StringTable;

pub mod string_table;

pub use self::string_table::StringTableDecoder as StringTableDecoder;

const TOKEN_STRING_TABLE: u16 = 0x0001;
const TOKEN_PACKAGE: u16 = 0x0200;

#[derive(Debug)]
pub enum Chunk {
    StringTable(StringTable),
    Unknown,
}

pub struct ChunkLoader;

impl ChunkLoader {
    pub fn read_all<'a>(raw_data:&[u8], cursor: &mut Cursor<&'a [u8]>, ending: u64) -> Result<Vec<Chunk>, Error> {
        let mut chunks = Vec::new();
        // Loop trough all of the frames
        loop {
             let initial_position = cursor.position();
             println!("Initial position {}", initial_position);
             if initial_position == ending as u64 {
                 // We are at the end of the document. We are done!
                 break;
             }

             let token = cursor.read_u16::<LittleEndian>()?;
             let header_size = cursor.read_u16::<LittleEndian>()?;
             let chunk_size = cursor.read_u32::<LittleEndian>()?;
             println!("Chunk ID: {:X}; HeaderSize: {}; Size: {}", token, header_size, chunk_size);

             let from = cursor.position() as usize;
             let to = (from + chunk_size as usize - 8) as usize;

             let slice = &raw_data[from..to];
             let mut chunk_cursor = Cursor::new(slice);

             let chunk = match token {
                 TOKEN_STRING_TABLE => StringTableDecoder::decode(raw_data, &mut chunk_cursor, initial_position as u32)?,
                 _ => Chunk::Unknown,
             };

             chunks.push(chunk);

             println!("Next position {}", initial_position as u64 + chunk_size as u64);
             cursor.set_position(initial_position as u64 + chunk_size as u64);
        }

        Ok(chunks)
    }
}
