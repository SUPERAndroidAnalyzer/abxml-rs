use chunks::{Chunk, ChunkLoader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;

pub struct ArscDecoder;

impl ArscDecoder {
    pub fn decode(&self, raw_data: &[u8]) -> Result<Vec<Chunk>> {
        let mut cursor = Cursor::new(raw_data);

        let token = cursor.read_u16::<LittleEndian>()?;
        let header_size = cursor.read_u16::<LittleEndian>()?;
        let chunk_size = cursor.read_u32::<LittleEndian>()?;
        let package_amount = cursor.read_u32::<LittleEndian>()?;

        ChunkLoader::read_all(&mut cursor, chunk_size as u64)
    }
}
