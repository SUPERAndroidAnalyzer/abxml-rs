use chunks::{Chunk, ChunkHeader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use document::{HeaderStringTable, StringTable};
use errors::*;
use parser::Decoder;
use std::clone::Clone;

pub struct ResourceDecoder;

impl ResourceDecoder {
    pub fn decode(cursor: &mut Cursor<&[u8]>, header: &ChunkHeader)  -> Result<Chunk> {
        cursor.set_position(header.get_data_offset());
        let amount = (header.get_chunk_end() - header.get_data_offset()) / 4;

        let resource_table = (1..amount)
            .into_iter()
            .map(|_| cursor.read_u32::<LittleEndian>().unwrap())
            .collect::<Vec<u32>>();

        Ok(Chunk::ResourceTable(resource_table))
     }
}
