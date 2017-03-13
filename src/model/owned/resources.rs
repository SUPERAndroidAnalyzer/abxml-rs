use model::owned::OwnedBuf;
use byteorder::{LittleEndian, WriteBytesExt};
use chunks::*;
use errors::*;

#[derive(Default)]
pub struct ResourcesBuf {
    resources: Vec<u32>,
}

impl ResourcesBuf {
    pub fn push_resource(&mut self, resource: u32) {
        self.resources.push(resource);
    }

    pub fn pop_resource(&mut self) -> Option<u32> {
        self.resources.pop()
    }
}

impl OwnedBuf for ResourcesBuf {
    fn get_token(&self) -> u16 {
        TOKEN_RESOURCE
    }

    fn get_body_data(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();

        for r in &self.resources {
            out.write_u32::<LittleEndian>(*r)?;
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::compare_chunks;
    use raw_chunks;

    #[test]
    fn it_can_generate_a_chunk_with_the_given_data() {
        let mut resources = ResourcesBuf::default();
        resources.push_resource(111);
        resources.push_resource(222);

        let out = resources.to_vec().unwrap();

        let chunk_header = ChunkHeader::new(0, 8, 2 * 8, 0x0180);
        let wrapper = ResourceWrapper::new(&out, chunk_header);

        let expected_resources: Vec<u32> = vec![111, 222];

        assert_eq!(expected_resources, wrapper.get_resources().unwrap());
    }

    #[test]
    fn it_can_generate_an_empty_chunk() {
        let resources = ResourcesBuf::default();
        let out = resources.to_vec().unwrap();

        let chunk_header = ChunkHeader::new(0, 8, (0 * 8) + 8, 0x0180);
        let wrapper = ResourceWrapper::new(&out, chunk_header);

        let expected_resources: Vec<u32> = vec![];

        assert_eq!(expected_resources, wrapper.get_resources().unwrap());
    }

    #[test]
    fn identity() {
        let raw = raw_chunks::EXAMPLE_RESOURCES;
        let chunk_header = ChunkHeader::new(0, 8, 24, 0x180);

        let wrapper = ResourceWrapper::new(&raw, chunk_header);
        let owned = wrapper.to_owned().unwrap();
        let new_raw = owned.to_vec().unwrap();

        compare_chunks(&raw, &new_raw);
    }
}
