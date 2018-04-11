use std::rc::Rc;

use byteorder::{LittleEndian, WriteBytesExt};
use failure::Error;

use chunks::*;
use model::owned::OwnedBuf;
use model::{NamespaceStart, StringTable};

pub struct XmlNamespaceStartBuf {
    line: u32,
    prefix_index: u32,
    namespace_index: u32,
}

impl XmlNamespaceStartBuf {
    pub fn new(line: u32, prefix_index: u32, namespace_index: u32) -> Self {
        Self {
            line,
            prefix_index,
            namespace_index,
        }
    }
}

impl NamespaceStart for XmlNamespaceStartBuf {
    fn get_line(&self) -> Result<u32, Error> {
        Ok(self.line)
    }

    fn get_prefix<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>, Error> {
        let string = string_table.get_string(self.prefix_index)?;

        Ok(string)
    }

    fn get_namespace<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>, Error> {
        let string = string_table.get_string(self.namespace_index)?;

        Ok(string)
    }
}

impl OwnedBuf for XmlNamespaceStartBuf {
    fn get_token(&self) -> u16 {
        TOKEN_XML_START_NAMESPACE
    }

    fn get_body_data(&self) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();

        out.write_u32::<LittleEndian>(self.prefix_index)?;
        out.write_u32::<LittleEndian>(self.namespace_index)?;

        Ok(out)
    }

    fn get_header(&self) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();

        out.write_u32::<LittleEndian>(self.line)?;
        out.write_u32::<LittleEndian>(0xFFFFFFFF)?;

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chunks::XmlNamespaceStartWrapper;
    use raw_chunks::EXAMPLE_NAMESPACE_START;
    use test::{compare_chunks, FakeStringTable};

    #[test]
    fn it_can_generate_a_chunk_with_the_given_data() {
        let fake_string_table = FakeStringTable;
        let namespace_start = XmlNamespaceStartBuf::new(99, 11, 33);

        assert_eq!(
            "Ones",
            &*namespace_start.get_prefix(&fake_string_table).unwrap()
        );
        assert_eq!(
            "Threes",
            &*namespace_start.get_namespace(&fake_string_table).unwrap()
        );
    }

    #[test]
    fn identity() {
        let wrapper = XmlNamespaceStartWrapper::new(EXAMPLE_NAMESPACE_START);

        let owned = wrapper.to_buffer().unwrap();
        let new_raw = owned.to_vec().unwrap();

        compare_chunks(&new_raw, &EXAMPLE_NAMESPACE_START);
    }
}
