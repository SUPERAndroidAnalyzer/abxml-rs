use chunks::{Chunk, ChunkLoader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;
use document::StringTable;

pub struct Decoder {
    string_table: Option<StringTable>,
}

impl Decoder {
    pub fn new() -> Self {
        Decoder {
            string_table: None,
        }
    }

    pub fn decode_arsc(&self, raw_data: &[u8]) -> Result<Vec<Chunk>> {
        let mut cursor = Cursor::new(raw_data);

        let token = cursor.read_u16::<LittleEndian>()?;
        let header_size = cursor.read_u16::<LittleEndian>()?;
        let chunk_size = cursor.read_u32::<LittleEndian>()?;
        let package_amount = cursor.read_u32::<LittleEndian>()?;

        info!("Parsing resources.arsc. Buffer size: {}", raw_data.len());

        ChunkLoader::read_all(&mut cursor, chunk_size as u64)
    }

    pub fn decode_xml(&self, raw_data: &[u8]) -> Result<Vec<Chunk>> {
        let mut cursor = Cursor::new(raw_data);

        let token = cursor.read_u16::<LittleEndian>()?;
        let header_size = cursor.read_u16::<LittleEndian>()?;
        let chunk_size = cursor.read_u32::<LittleEndian>()?;
        let package_amount = cursor.read_u32::<LittleEndian>()?;

        info!("Parsing resources.arsc. Buffer size: {}", raw_data.len());

        ChunkLoader::read_all(&mut cursor, chunk_size as u64)
    }
}
