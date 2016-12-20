use chunks::{Chunk, ChunkLoader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;
use document::{StringTable, ElementContainer};
use std::rc::Rc;

pub struct Decoder {
    string_table: Option<Rc<StringTable>>,
    element_container: ElementContainer,
}

impl Decoder {
    pub fn new() -> Self {
        Decoder {
            string_table: None,
            element_container: ElementContainer::new(),
        }
    }

    pub fn decode_arsc(&mut self, raw_data: &[u8]) -> Result<Vec<Chunk>> {
        let mut cursor = Cursor::new(raw_data);

        let token = cursor.read_u16::<LittleEndian>()?;
        let header_size = cursor.read_u16::<LittleEndian>()?;
        let chunk_size = cursor.read_u32::<LittleEndian>()?;
        let package_amount = cursor.read_u32::<LittleEndian>()?;

        info!("Parsing resources.arsc. Buffer size: {}", raw_data.len());

        let chunk = ChunkLoader::read(self, &mut cursor)?;

        match chunk {
            Chunk::StringTable(st_rc) => {
                self.string_table = Some(st_rc.clone());
            },
            _ => return Err("First chunk should be a string table".into()),
        }

        ChunkLoader::read_all(self, &mut cursor, chunk_size as u64)
    }

    pub fn decode_xml(&mut self, raw_data: &[u8]) -> Result<Vec<Chunk>> {
        let mut cursor = Cursor::new(raw_data);

        let token = cursor.read_u16::<LittleEndian>()?;
        let header_size = cursor.read_u16::<LittleEndian>()?;
        let chunk_size = cursor.read_u32::<LittleEndian>()?;

        info!("Parsing resources.arsc. Buffer size: {}", raw_data.len());

        let chunk = ChunkLoader::read(self, &mut cursor)?;

        match chunk {
            Chunk::StringTable(st_rc) => {
                self.string_table = Some(st_rc.clone());
            },
            _ => return Err("First chunk should be a string table".into()),
        }

        ChunkLoader::read_all(self, &mut cursor, chunk_size as u64)
    }

    pub fn get_string_table(&self) -> &Option<Rc<StringTable>> {
        &self.string_table
    }

    pub fn get_element_container(&mut self) -> &mut ElementContainer {
        &mut self.element_container
    }
}
