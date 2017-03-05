use model::owned::OwnedBuf;
use chunks::TOKEN_XML_TAG_END;
use byteorder::{LittleEndian, WriteBytesExt};
use errors::*;
use model::TagEnd;

pub struct XmlTagEndBuf {
    id: u32,
}

impl XmlTagEndBuf {
    pub fn new(id: u32) -> Self {
        XmlTagEndBuf {
            id: id,
        }
    }
}

impl OwnedBuf for XmlTagEndBuf {
    fn get_token(&self) -> u16 {
        TOKEN_XML_TAG_END
    }

    fn get_body_data(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();

        // Amount of writes
        out.write_u32::<LittleEndian>(3)?;

        // ??
        out.write_u32::<LittleEndian>(0xFFFFFFFF)?;
        // ??
        out.write_u32::<LittleEndian>(0xFFFFFFFF)?;
        // Id
        out.write_u32::<LittleEndian>(self.id)?;

        Ok(out)
    }

    fn get_header_size(&self) -> u16 {
        16
    }
}

impl TagEnd for XmlTagEndBuf {
    fn get_id(&self) -> Result<u32> {
        Ok(self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chunks::*;

    #[test]
    fn it_can_generate_an_empty_chunk() {
        let tag_end = XmlTagEndBuf::new(0);
        let out = tag_end.to_vec().unwrap();
        let expected = [3, 1, 16, 0, 24, 0, 0, 0, 3, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0];

        assert_eq!(expected, out.as_slice());
    }

    #[test]
    fn identity() {
        let raw = [3, 1, 16, 0, 24, 0, 0, 0, 3, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 44, 0, 0, 0];
        let chunk_header = ChunkHeader::new(0, 8, 24, 0x103);
        let wrapper = XmlTagEndWrapper::new(&raw, chunk_header);

        let owned = wrapper.to_owned().unwrap();
        let new_raw = owned.to_vec().unwrap();

        assert_eq!(new_raw, raw);
    }
}
