use std::io::Error;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;

pub mod string_table;

pub use self::string_table::StringTable as StringTable;

const TOKEN_STRING_TABLE: u32 = 0x0001;
const TOKEN_PACKAGE: u32 = 0x0200;

#[derive(Debug)]
pub enum Chunk {
    Unknown,
}

pub struct ChunkLoader;

impl ChunkLoader {
    pub fn read_all<'a>(cursor: &mut Cursor<&'a [u8]>, ending: u64) -> Result<Vec<Chunk>, Error> {
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

         let chunk = match token {
             // TOKEN_STRING_TABLE => StringTableDecoder::decode(),
             _ => Chunk::Unknown,
         };

         chunks.push(chunk);

         println!("Next position {}", initial_position as u64 + chunk_size as u64);
         cursor.set_position(initial_position as u64 + chunk_size as u64);
        }

        Ok(chunks)
    }
}
