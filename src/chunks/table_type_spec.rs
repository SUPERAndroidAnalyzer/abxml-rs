use chunks::{Chunk, ChunkHeader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use document::{HeaderStringTable, StringTable};
use errors::*;
use parser::Decoder;

pub struct TableTypeSpecDecoder;

impl TableTypeSpecDecoder {
    pub fn decode(decoder: &mut Decoder, cursor: &mut Cursor<&[u8]>, header: &ChunkHeader)  -> Result<Chunk> {
        let id = cursor.read_u32::<LittleEndian>()? & 0xFF;
        let resource_count = cursor.read_u32::<LittleEndian>()?;

        let mut resources = Vec::new();

        for _ in 0..resource_count {
            resources.push(cursor.read_u32::<LittleEndian>()?);
        }

        //println!("{:?}", resources);
        decoder.push_type_spec(Chunk::TableTypeSpec(id, resources.clone()));

        Ok(Chunk::TableTypeSpec(id, resources))
    }
}

struct TypeSpec {
    id: u8,
    resources: Vec<u32>,
}

impl TypeSpec {
    pub fn new(id: u8, resources: Vec<u32>) -> Self {
        TypeSpec {
            id: id,
            resources: resources,
        }
    }
}
