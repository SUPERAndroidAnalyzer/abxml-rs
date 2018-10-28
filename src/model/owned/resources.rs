use byteorder::{LittleEndian, WriteBytesExt};
use failure::Error;

use crate::{chunks::*, model::owned::OwnedBuf};

#[derive(Default, Debug)]
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

    fn get_body_data(&self) -> Result<Vec<u8>, Error> {
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
    use crate::{raw_chunks, test::compare_chunks};

    #[test]
    fn it_can_generate_a_chunk_with_the_given_data() {
        let mut resources = ResourcesBuf::default();
        resources.push_resource(111);
        resources.push_resource(222);

        let out = resources.to_vec().unwrap();

        let wrapper = ResourceWrapper::new(&out);

        let expected_resources: Vec<u32> = vec![111, 222];

        assert_eq!(expected_resources, wrapper.get_resources().unwrap());
    }

    #[test]
    fn it_can_generate_an_empty_chunk() {
        let resources = ResourcesBuf::default();
        let out = resources.to_vec().unwrap();

        let wrapper = ResourceWrapper::new(&out);

        let expected_resources: Vec<u32> = vec![];

        assert_eq!(expected_resources, wrapper.get_resources().unwrap());
    }

    #[test]
    fn identity() {
        let raw = raw_chunks::EXAMPLE_RESOURCES;

        let wrapper = ResourceWrapper::new(&raw);
        let owned = wrapper.to_buffer().unwrap();
        let new_raw = owned.to_vec().unwrap();

        compare_chunks(&raw, &new_raw);
    }
}
