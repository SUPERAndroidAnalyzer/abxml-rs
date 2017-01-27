use chunks::{Chunk, ChunkHeader};
use std::io::Cursor;
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
}
