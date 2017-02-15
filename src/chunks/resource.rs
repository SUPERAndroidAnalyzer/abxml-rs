use chunks::{Chunk, ChunkHeader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;

pub struct ResourceDecoder;

impl ResourceDecoder {
    pub fn decode<'a>(cursor: &mut Cursor<&'a [u8]>, header: &ChunkHeader)  -> Result<Chunk<'a>> {
        let ttw = ResourceWrapper::new(cursor.get_ref(), *header);
        Ok(Chunk::Resource(ttw))
     }
}

#[allow(dead_code)]
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

    pub fn get_resources(&self) -> Result<Vec<u32>> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(4));

        let count = cursor.read_u32::<LittleEndian>()?;
        let mut resources = Vec::with_capacity(count as usize);

        for _ in 0..(count / 4) {
            resources.push(cursor.read_u32::<LittleEndian>()?);
        }

        Ok(resources)
    }
}

#[allow(dead_code)]
pub struct Resource<'a> {
    wrapper: ResourceWrapper<'a>,
}

impl<'a> Resource<'a> {
    pub fn new(wrapper: ResourceWrapper<'a>) -> Self {
        Resource {
            wrapper: wrapper,
        }
    }

    pub fn get_resources(&self) -> Result<Vec<u32>> {
        self.wrapper.get_resources()
    }
}
