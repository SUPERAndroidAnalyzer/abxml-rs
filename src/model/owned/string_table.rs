use model::owned::OwnedBuf;
use byteorder::{LittleEndian, WriteBytesExt};
use errors::*;
use chunks::*;
use model::StringTable as StringTableTrait;
use std::rc::Rc;

#[derive(Clone, Copy)]
pub enum Encoding {
    Utf8,
    Utf16,
}

pub struct StringTableBuf {
    strings: Vec<Rc<String>>,
    styles: Vec<Rc<String>>,
    encoding: Encoding,
}

impl StringTableBuf {
    pub fn new() -> Self {
        StringTableBuf {
            strings: Vec::new(),
            styles: Vec::new(),
            encoding: Encoding::Utf8,
        }
    }

    pub fn set_encoding(&mut self, encoding: Encoding) {
        self.encoding = encoding;
    }

    pub fn get_encoding(&self) -> Encoding {
        self.encoding
    }
}

impl OwnedBuf for StringTableBuf {
    fn get_token(&self) -> u16 {
        TOKEN_STRING_TABLE
    }

    fn get_body_data(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();

        Ok(out)
    }
}

impl StringTableTrait for StringTableBuf {
    fn get_strings_len(&self) -> u32 {
        self.strings.len() as u32
    }

    fn get_styles_len(&self) -> u32 {
        self.styles.len() as u32
    }

    fn get_string(&self, idx: u32) -> Result<Rc<String>> {
        match self.strings.get(idx as usize) {
            None => Err("String not found".into()),
            Some(s) => Ok(s.clone()),
        }
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
        assert_eq!(0, string_table.get_styles_len());
        assert!(string_table.get_string(0).is_err());
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