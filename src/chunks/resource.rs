use chunks::{Chunk, ChunkHeader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;
use model::owned::ResourcesBuf;

pub struct ResourceDecoder;

impl ResourceDecoder {
    pub fn decode<'a>(cursor: &mut Cursor<&'a [u8]>, header: &ChunkHeader) -> Result<Chunk<'a>> {
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

        for _ in 0..(count / 4) - 2 {
            resources.push(cursor.read_u32::<LittleEndian>()?);
        }

        Ok(resources)
    }

    pub fn to_owned(self) -> Result<ResourcesBuf> {
        let mut owned = ResourcesBuf::default();

        for r in &self.get_resources()? {
            owned.push_resource(*r);
        }

        Ok(owned)
    }
}

#[allow(dead_code)]
pub struct Resource<'a> {
    wrapper: ResourceWrapper<'a>,
}

impl<'a> Resource<'a> {
    pub fn new(wrapper: ResourceWrapper<'a>) -> Self {
        Resource { wrapper: wrapper }
    }

    pub fn get_resources(&self) -> Result<Vec<u32>> {
        self.wrapper.get_resources()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::owned::ResourceBuf;
    use chunks::*;
    use model::owned::OwnedBuf;

    #[test]
    fn it_can_not_decode_a_buffer_with_an_invalid_size() {
        let mut resources = ResourceBuf::new();
        resources.push_resource(111);
        resources.push_resource(222);
        let mut out = resources.to_vec().unwrap();

        out[4] = 255;

        let chunk_header = ChunkHeader::new(0, 8, 255, 0x0180);
        let wrapper = ResourceWrapper::new(&out, chunk_header);

        let result = wrapper.get_resources();
        assert!(result.is_err());
        assert_eq!("failed to fill whole buffer",
                   result.err().unwrap().to_string());
    }

    #[test]
    fn it_can_not_decode_a_buffer_if_chunk_header_is_not_correct() {
        let mut resources = ResourceBuf::new();
        resources.push_resource(111);
        resources.push_resource(222);
        let out = resources.to_vec().unwrap();

        let chunk_header = ChunkHeader::new(3000, 8, 2 * 4, 0x0180);
        let wrapper = ResourceWrapper::new(&out, chunk_header);

        let result = wrapper.get_resources();
        assert!(result.is_err());
        assert_eq!("failed to fill whole buffer",
                   result.err().unwrap().to_string());
    }
}
