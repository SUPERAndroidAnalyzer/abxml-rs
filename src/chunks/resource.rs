use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;
use model::owned::ResourcesBuf;

#[allow(dead_code)]
pub struct ResourceWrapper<'a> {
    raw_data: &'a [u8],
}

impl<'a> ResourceWrapper<'a> {
    pub fn new(raw_data: &'a [u8]) -> Self {
        ResourceWrapper { raw_data: raw_data }
    }

    pub fn get_resources(&self) -> Result<Vec<u32>> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(4);

        let count = cursor.read_u32::<LittleEndian>()?;
        let mut resources = Vec::with_capacity(count as usize);

        for _ in 0..(count / 4) - 2 {
            resources.push(cursor.read_u32::<LittleEndian>()?);
        }

        Ok(resources)
    }


    pub fn to_buffer(&self) -> Result<ResourcesBuf> {
        let mut owned = ResourcesBuf::default();

        for r in &self.get_resources()? {
            owned.push_resource(*r);
        }

        Ok(owned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::owned::ResourcesBuf;
    use model::owned::OwnedBuf;

    #[test]
    fn it_can_not_decode_a_buffer_with_an_invalid_size() {
        let mut resources = ResourcesBuf::default();
        resources.push_resource(111);
        resources.push_resource(222);
        let mut out = resources.to_vec().unwrap();

        out[4] = 255;

        let wrapper = ResourceWrapper::new(&out);

        let result = wrapper.get_resources();
        assert!(result.is_err());
        assert_eq!("failed to fill whole buffer",
                   result.err().unwrap().to_string());
    }
}
