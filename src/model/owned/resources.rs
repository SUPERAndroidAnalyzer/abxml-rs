use model::owned::OwnedBuf;
use byteorder::{LittleEndian, WriteBytesExt};
use errors::*;

pub struct ResourcesBuf {
    resources: Vec<u32>,
}

impl ResourcesBuf {
    pub fn new() -> Self {
        ResourcesBuf {
            resources: Vec::new(),
        }
    }

    pub fn push_resource(&mut self, resource: u32) {
        self.resources.push(resource);
    }

    pub fn pop_resource(&mut self) -> Option<u32> {
        self.resources.pop()
    }
}

impl OwnedBuf for ResourcesBuf {
    fn to_vec(&self) -> Result<Vec<u8>> {
        // Generate a chunk with the current data
        // TODO: Generate with capacity
        let mut out = Vec::new();

        // Token
        // TODO: Use constant on chunks/mod.rs
        out.write_u16::<LittleEndian>(0x0180)?;
        // TODO: Check the header size. Fixed?
        out.write_u16::<LittleEndian>(8)?;
        let chunk_size = (self.resources.len() * 4);
        out.write_u32::<LittleEndian>(chunk_size as u32)?;

        for r in &self.resources {
            out.write_u32::<LittleEndian>(*r)?;
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chunks::*;

    #[test]
    fn it_can_generate_a_chunk_with_the_given_data() {
        let mut resources = ResourcesBuf::new();
        resources.push_resource(111);
        resources.push_resource(222);

        let out = resources.to_vec().unwrap();

        let chunk_header = ChunkHeader::new(0, 8, 2*8, 0x0180);
        let wrapper = ResourceWrapper::new(&out, chunk_header);

        let expected_resources: Vec<u32> = vec![111, 222];

        assert_eq!(expected_resources, wrapper.get_resources().unwrap());
    }

    #[test]
    fn it_can_generate_an_empty_chunk() {
        let resources = ResourcesBuf::new();
        let out = resources.to_vec().unwrap();

        let chunk_header = ChunkHeader::new(0, 8, 0*8, 0x0180);
        let wrapper = ResourceWrapper::new(&out, chunk_header);

        let expected_resources: Vec<u32> = vec![];

        assert_eq!(expected_resources, wrapper.get_resources().unwrap());
    }
}