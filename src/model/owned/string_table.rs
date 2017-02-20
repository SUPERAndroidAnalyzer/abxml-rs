use model::owned::OwnedBuf;
use byteorder::{LittleEndian, WriteBytesExt};
use errors::*;
use chunks::*;
use model::StringTable as StringTableTrait;
use std::rc::Rc;

pub enum StringMode {
    Utf8,
    Utf16Ext,
    Utf16,
}

pub struct StringTableBuf {
    strings: Vec<(StringMode, String)>,
    styles: Vec<(StringMode, String)>,
}

impl StringTableBuf {
    pub fn new() -> Self {
        StringTableBuf {
            strings: Vec::new(),
            styles: Vec::new(),
        }
    }
}

impl OwnedBuf for StringTableBuf {
    fn to_vec(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();

        out.write_u16::<LittleEndian>(TOKEN_STRING_TABLE)?;

        Ok(out)
    }
}

impl StringTableTrait for StringTableBuf {
    fn get_strings_len(&self) -> u32 {
        0
    }

    fn get_styles_len(&self) -> u32 {
        12
    }

    fn get_string(&self, idx: u32) -> Result<Rc<String>> {
        Ok(Rc::new("string".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chunks::*;

    #[test]
    fn it_can_generate_an_empty_chunk() {
        let string_table = StringTableBuf::new();
        let out = string_table.to_vec().unwrap();

        let chunk_header = ChunkHeader::new(0, 8, (0*8)+8, 0x0180);
        let wrapper = StringTableWrapper::new(&out, chunk_header);

        assert_eq!(0, string_table.get_strings_len());
    }

    /*
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
    fn identity() {
        let raw = vec![128, 1, 8, 0, 24, 0, 0, 0, 160, 0, 1, 1, 158, 0, 1, 1, 31, 3, 1, 1, 165, 1, 1, 1];
        let chunk_header = ChunkHeader::new(0, 8, 24, 0x180);

        let wrapper = ResourceWrapper::new(&raw, chunk_header);
        let owned = wrapper.to_owned().unwrap();
        let new_raw = owned.to_vec().unwrap();

        assert_eq!(&raw, &new_raw);
    }*/
}