use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};
use failure::Error;

use model::owned::ResourcesBuf;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ResourceWrapper<'a> {
    raw_data: &'a [u8],
}

impl<'a> ResourceWrapper<'a> {
    pub fn new(raw_data: &'a [u8]) -> Self {
        Self { raw_data }
    }

    pub fn get_resources(&self) -> Result<Vec<u32>, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(4);

        let count = cursor.read_u32::<LittleEndian>()?;
        let amount_of_resources = (count / 4) - 2;

        ensure!(
            count <= self.raw_data.len() as u32,
            "there is not enough data on the buffer ({}) to read the reported resources count ({})",
            self.raw_data.len(),
            count
        );

        let mut resources = Vec::with_capacity(amount_of_resources as usize);

        for _ in 0..amount_of_resources {
            resources.push(cursor.read_u32::<LittleEndian>()?);
        }

        Ok(resources)
    }

    pub fn to_buffer(&self) -> Result<ResourcesBuf, Error> {
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
    use model::owned::OwnedBuf;
    use model::owned::ResourcesBuf;

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
        assert_eq!(
            "there is not enough data on the buffer (16) to read the reported resources count \
             (255)",
            result.err().unwrap().to_string()
        );
    }
}
