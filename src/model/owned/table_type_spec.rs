use model::owned::OwnedBuf;
use errors::*;
use model::TypeSpec;
use byteorder::{LittleEndian, WriteBytesExt};

pub struct TableTypeSpecBuf {
    id: u16,
    flags: Vec<u32>,
}

impl TableTypeSpecBuf {
    pub fn new(id: u16) -> Self {
        TableTypeSpecBuf {
            id: id,
            flags: Vec::new(),
        }
    }

    pub fn push_flag(&mut self, flag: u32) {
        self.flags.push(flag)
    }
}

impl OwnedBuf for TableTypeSpecBuf {
    fn get_token(&self) -> u16 {
        0x202
    }

    fn get_body_data(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();

        for flag in &self.flags {
            out.write_u32::<LittleEndian>(*flag)?;
        }

        Ok(out)
    }

    fn get_header(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();

        out.write_u32::<LittleEndian>(self.id as u32)?;
        out.write_u32::<LittleEndian>(self.flags.len() as u32)?;

        Ok(out)
    }
}

impl TypeSpec for TableTypeSpecBuf {
    fn get_id(&self) -> Result<u16> {
        Ok(self.id)
    }
    fn get_amount(&self) -> Result<u32> {
        Ok(self.flags.len() as u32)
    }

    fn get_flag(&self, index: u32) -> Result<u32> {
        self.flags
            .get(index as usize)
            .cloned()
            .ok_or_else(|| "Flag out of bounds".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::owned::OwnedBuf;
    use test::compare_chunks;
    use raw_chunks;
    use chunks::*;

    #[test]
    fn it_can_generate_a_chunk_with_the_given_data() {
        let type_spec = TableTypeSpecBuf::new(14);

        assert_eq!(14, type_spec.get_id().unwrap());
    }

    #[test]
    fn identity() {
        let header = ChunkHeader::new(0, 16, raw_chunks::EXAMPLE_TYPE_SPEC.len() as u32, 0x202);
        let wrapper = TypeSpecWrapper::new(raw_chunks::EXAMPLE_TYPE_SPEC, header);

        let owned = wrapper.to_owned().unwrap();
        let new_raw = owned.to_vec().unwrap();

        compare_chunks(&new_raw, &raw_chunks::EXAMPLE_TYPE_SPEC);
    }
}
