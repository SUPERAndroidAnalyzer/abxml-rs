use chunks::{Chunk, ChunkLoader};
use visitor::ChunkVisitor;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;
use document::{StringTable, ElementContainer, Namespaces};
use std::rc::Rc;

pub struct Decoder {
    string_table: Option<Rc<StringTable>>,
    element_container: ElementContainer,
    namespaces: Namespaces,
    type_specs: Vec<Chunk>,
}

impl Decoder {
    pub fn new() -> Self {
        Decoder {
            string_table: None,
            element_container: ElementContainer::new(),
            namespaces: Namespaces::new(),
            type_specs: Vec::new(),
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

    pub fn get_mut_element_container(&mut self) -> &mut ElementContainer {
        &mut self.element_container
    }

    pub fn get_element_container(&self) -> &ElementContainer {
        &self.element_container
    }

    pub fn push_namespace(&mut self, namespace: Rc<String>, prefix: Rc<String>) {
        self.namespaces.insert(prefix, namespace);
    }

    pub fn push_type_spec(&mut self, type_spec: Chunk) {
        self.type_specs.push(type_spec);
    }

    pub fn get_last_type_spec(&self) -> Option<&Chunk> {
        match self.type_specs.last() {
            Some(c) => Some(&c),
            None => None,
        }
    }

    pub fn get_namespaces(&self) -> &Namespaces {
        &self.namespaces
    }
}
