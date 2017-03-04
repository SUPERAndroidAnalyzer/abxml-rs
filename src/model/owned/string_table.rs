use model::owned::OwnedBuf;
use byteorder::{LittleEndian, WriteBytesExt};
use errors::*;
use chunks::*;
use model::StringTable as StringTableTrait;
use std::rc::Rc;
use encoding::codec::{utf_16, utf_8};
use encoding::Encoding as EncodingTrait;

#[derive(Clone, Copy, PartialEq)]
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

    pub fn add_string(&mut self, new_string: String) {
        self.strings.push(Rc::new(new_string));
    }
}

impl OwnedBuf for StringTableBuf {
    fn get_token(&self) -> u16 {
        TOKEN_STRING_TABLE
    }

    fn get_body_data(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        let flags = if self.encoding == Encoding::Utf8 {
            0x00000100
        } else {
            0
        };

        out.write_u32::<LittleEndian>(self.strings.len() as u32)?;
        out.write_u32::<LittleEndian>(self.styles.len() as u32)?;
        out.write_u32::<LittleEndian>(flags)?;

        let string_offset = self.get_header_size() as u32 +
            (self.get_strings_len() as u32 * 4) +
            (self.get_styles_len() as u32 * 4);

        let mut string_offsets: Vec<u32> = Vec::new();
        let mut style_offsets: Vec<u32> = Vec::new();
        let mut string_buffer: Vec<u8> = Vec::new();
        let style_buffer: Vec<u8> = Vec::new();

        let mut current_offset = 0;
        let mut encoder = if self.encoding == Encoding::Utf8 {
            utf_8::UTF8Encoder::new()
        } else {
            utf_16::UTF_16LE_ENCODING.raw_encoder()
        };

        // TODO: Calculate properly the offset as we are calculating on the decoding part
        let style_offset = 0;
        out.write_u32::<LittleEndian>(string_offset as u32)?;
        out.write_u32::<LittleEndian>(style_offset as u32)?;

        // Encode strings and save offsets
        for string in self.strings.iter() {
            string_offsets.push(current_offset);
            let mut encoded_string = Vec::new();
            let (size, error) = encoder.raw_feed(string, &mut encoded_string);

            if let Some(_) = error {
                return Err("Error encoding string".into());
            }

            // Write size
            let low = (size & 0xFF) as u8;
            let high = ((size & 0xFF00) >> 8) as u8;
            string_buffer.push(low);
            string_buffer.push(high);
            string_buffer.extend(&encoded_string);
            string_buffer.push(0x00);
            string_buffer.push(0x00);

            current_offset = string_buffer.len() as u32;
            // println!("Current offset: {}", current_offset);
        }

        // Encode styles and save offsets
        for _ in self.styles.iter() {
            style_offsets.push(current_offset);
            // Encode with utf8/utf16 depending on the flag
        }

        for offset in string_offsets {
            out.write_u32::<LittleEndian>(offset)?;
        }

        for offset in style_offsets {
            out.write_u32::<LittleEndian>(offset)?;
        }

        out.extend(string_buffer);
        out.extend(style_buffer);

        Ok(out)
    }

    fn get_header_size(&self) -> u16 {
        7*4
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

    #[test]
    fn it_can_generate_an_empty_chunk() {
        let string_table = StringTableBuf::new();

        assert_eq!(0, string_table.get_strings_len());
        assert_eq!(0, string_table.get_styles_len());
        assert!(string_table.get_string(0).is_err());
    }

    #[test]
    fn it_can_generate_a_chunk_with_the_given_data() {
        let mut string_table = StringTableBuf::new();
        string_table.add_string("some string".to_string());
        string_table.add_string("忠犬ハチ公".to_string());

        assert_eq!(2, string_table.get_strings_len());
        assert_eq!(0, string_table.get_styles_len());
        assert_eq!("some string", *(string_table.get_string(0).unwrap()));
        assert_eq!("忠犬ハチ公", *(string_table.get_string(1).unwrap()));
        assert!(string_table.get_string(2).is_err());
    }


    #[test]
    fn identity() {
        let raw = vec![1, 0, 28, 0, 44, 2, 0, 0, 22, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 22, 0, 0, 0, 42, 0, 0, 0, 66, 0, 0, 0, 92, 0, 0, 0, 106, 0, 0, 0, 136, 0, 0, 0, 150, 0, 0, 0, 166, 0, 0, 0, 178, 0, 0, 0, 188, 0, 0, 0, 202, 0, 0, 0, 218, 0, 0, 0, 236, 0, 0, 0, 68, 1, 0, 0, 72, 1, 0, 0, 86, 1, 0, 0, 102, 1, 0, 0, 114, 1, 0, 0, 134, 1, 0, 0, 152, 1, 0, 0, 166, 1, 0, 0, 9, 0, 105, 0, 110, 0, 115, 0, 101, 0, 116, 0, 76, 0, 101, 0, 102, 0, 116, 0, 0, 0, 8, 0, 105, 0, 110, 0, 115, 0, 101, 0, 116, 0, 84, 0, 111, 0, 112, 0, 0, 0, 10, 0, 105, 0, 110, 0, 115, 0, 101, 0, 116, 0, 82, 0, 105, 0, 103, 0, 104, 0, 116, 0, 0, 0, 11, 0, 105, 0, 110, 0, 115, 0, 101, 0, 116, 0, 66, 0, 111, 0, 116, 0, 116, 0, 111, 0, 109, 0, 0, 0, 5, 0, 99, 0, 111, 0, 108, 0, 111, 0, 114, 0, 0, 0, 13, 0, 115, 0, 116, 0, 97, 0, 116, 0, 101, 0, 95, 0, 101, 0, 110, 0, 97, 0, 98, 0, 108, 0, 101, 0, 100, 0, 0, 0, 5, 0, 115, 0, 104, 0, 97, 0, 112, 0, 101, 0, 0, 0, 6, 0, 114, 0, 97, 0, 100, 0, 105, 0, 117, 0, 115, 0, 0, 0, 4, 0, 108, 0, 101, 0, 102, 0, 116, 0, 0, 0, 3, 0, 116, 0, 111, 0, 112, 0, 0, 0, 5, 0, 114, 0, 105, 0, 103, 0, 104, 0, 116, 0, 0, 0, 6, 0, 98, 0, 111, 0, 116, 0, 116, 0, 111, 0, 109, 0, 0, 0, 7, 0, 97, 0, 110, 0, 100, 0, 114, 0, 111, 0, 105, 0, 100, 0, 0, 0, 42, 0, 104, 0, 116, 0, 116, 0, 112, 0, 58, 0, 47, 0, 47, 0, 115, 0, 99, 0, 104, 0, 101, 0, 109, 0, 97, 0, 115, 0, 46, 0, 97, 0, 110, 0, 100, 0, 114, 0, 111, 0, 105, 0, 100, 0, 46, 0, 99, 0, 111, 0, 109, 0, 47, 0, 97, 0, 112, 0, 107, 0, 47, 0, 114, 0, 101, 0, 115, 0, 47, 0, 97, 0, 110, 0, 100, 0, 114, 0, 111, 0, 105, 0, 100, 0, 0, 0, 0, 0, 0, 0, 5, 0, 105, 0, 110, 0, 115, 0, 101, 0, 116, 0, 0, 0, 6, 0, 114, 0, 105, 0, 112, 0, 112, 0, 108, 0, 101, 0, 0, 0, 4, 0, 105, 0, 116, 0, 101, 0, 109, 0, 0, 0, 8, 0, 115, 0, 101, 0, 108, 0, 101, 0, 99, 0, 116, 0, 111, 0, 114, 0, 0, 0, 7, 0, 99, 0, 111, 0, 114, 0, 110, 0, 101, 0, 114, 0, 115, 0, 0, 0, 5, 0, 115, 0, 111, 0, 108, 0, 105, 0, 100, 0, 0, 0, 7, 0, 112, 0, 97, 0, 100, 0, 100, 0, 105, 0, 110, 0, 103, 0, 0, 0];
        let chunk_header = ChunkHeader::new(0, 8, raw.len() as u32, TOKEN_STRING_TABLE);
        let wrapper = StringTableWrapper::new(&raw, chunk_header);

        let owned = wrapper.to_owned().unwrap();
        let owned_as_vec = owned.to_vec().unwrap();

        let len = if raw.len() < owned_as_vec.len() {
            raw.len()
        } else {
            owned_as_vec.len()
        };

        for i in 0..len {
            if raw[i] != owned_as_vec[i] {
                println!("Difference @{}: {} <-> {}", i, raw[i], owned_as_vec[i])
            }
        }
        assert_eq!(owned_as_vec, raw);
    }

    #[test]
    fn identity_utf8() {
        // TODO: Test with UTF8 encoding
    }
}