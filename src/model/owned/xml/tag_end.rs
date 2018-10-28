use byteorder::{LittleEndian, WriteBytesExt};
use failure::Error;

use chunks::TOKEN_XML_TAG_END;
use model::{owned::OwnedBuf, TagEnd};

#[derive(Debug, Copy, Clone)]
pub struct XmlTagEndBuf {
    id: u32,
}

impl XmlTagEndBuf {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

impl OwnedBuf for XmlTagEndBuf {
    fn get_token(&self) -> u16 {
        TOKEN_XML_TAG_END
    }

    fn get_body_data(&self) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();

        // ??
        out.write_u32::<LittleEndian>(0xFFFF_FFFF)?;
        // Id
        out.write_u32::<LittleEndian>(self.id)?;

        Ok(out)
    }

    fn get_header(&self) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();

        // Amount of writes
        out.write_u32::<LittleEndian>(3)?;
        // ??
        out.write_u32::<LittleEndian>(0xFFFF_FFFF)?;

        Ok(out)
    }
}

impl TagEnd for XmlTagEndBuf {
    fn get_id(&self) -> Result<u32, Error> {
        Ok(self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chunks::*;
    use raw_chunks;
    use test::compare_chunks;

    #[test]
    fn it_can_generate_an_empty_chunk() {
        let tag_end = XmlTagEndBuf::new(0);
        let out = tag_end.to_vec().unwrap();
        let expected = [
            3, 1, 16, 0, 24, 0, 0, 0, 3, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0,
            0,
        ];

        assert_eq!(expected, out.as_slice());
    }

    #[test]
    fn identity() {
        let raw = raw_chunks::EXAMPLE_TAG_END;
        let wrapper = XmlTagEndWrapper::new(&raw);

        let owned = wrapper.to_buffer().unwrap();
        let new_raw = owned.to_vec().unwrap();

        compare_chunks(&raw, &new_raw);
    }
}
