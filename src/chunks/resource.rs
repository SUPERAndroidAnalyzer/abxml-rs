use chunks::{Chunk, ChunkHeader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use document::{HeaderStringTable, StringTable};
use errors::*;
use std::clone::Clone;

pub struct ResourceDecoder;

impl ResourceDecoder {
    pub fn decode<'a>(cursor: &mut Cursor<&'a [u8]>, header: &ChunkHeader)  -> Result<Chunk<'a>> {
/*        cursor.set_position(header.get_data_offset());
        let amount = (header.get_chunk_end() - header.get_data_offset()) / 4;

        let resource_table = (1..amount)
            .into_iter()
            .map(|_| cursor.read_u32::<LittleEndian>().unwrap())
            .collect::<Vec<u32>>();
*/
        let ttw = ResourceWrapper::new(cursor.get_ref(), (*header).clone());
        Ok(Chunk::Resource(ttw))
     }
}

pub struct ResourceWrapper<'a> {
    raw_data: &'a [u8],
    header: ChunkHeader,
}

impl<'a> ResourceWrapper<'a> {
    pub fn new(raw_data: &'a [u8], header: ChunkHeader) -> Self {
        ResourceWrapper {
            raw_data: raw_data,
            header: header,
        }
    }
}

pub struct Resource<'a> {
    wrapper: ResourceWrapper<'a>,
}

impl<'a> Resource<'a> {
    pub fn new(wrapper: ResourceWrapper<'a>) -> Self {
        Resource {
            wrapper: wrapper,
        }
    }
}
