use model::owned::OwnedBuf;
use errors::*;
use model::TypeSpec;

pub struct TableTypeSpecBuf {
    id: u16,
}

impl TableTypeSpecBuf {
    pub fn new(id: u16) -> Self {
        TableTypeSpecBuf {
            id: id,
        }
    }
}

impl OwnedBuf for TableTypeSpecBuf {
    fn get_token(&self) -> u16 {
        0x202
    }

    fn get_body_data(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();

        Ok(out)
    }

    fn get_header_size(&self) -> u16 {
        8
    }
}

impl TypeSpec for TableTypeSpecBuf {
    fn get_id(&self) -> Result<u16> {
        Ok(self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::owned::OwnedBuf;

    #[test]
    fn it_can_generate_a_chunk_with_the_given_data() {
        let type_spec = TableTypeSpecBuf::new(14);

        assert_eq!(14, type_spec.get_id().unwrap());
    }


    #[test]
    fn identity() {

    }
}