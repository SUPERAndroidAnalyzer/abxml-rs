use model::owned::OwnedBuf;
use byteorder::{LittleEndian, WriteBytesExt};
use errors::*;
use chunks::*;
use model::StringTable;
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

impl Default for StringTableBuf {
    fn default() -> StringTableBuf {
        StringTableBuf {
            strings: Vec::new(),
            styles: Vec::new(),
            encoding: Encoding::Utf8,
        }
    }
}

impl StringTableBuf {
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

    fn get_header(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();

        let flags = if self.encoding == Encoding::Utf8 {
            0x00000100
        } else {
            0
        };

        let header_size = 7 * 4;
        let string_offset = header_size + (self.get_strings_len() as u32 * 4) +
                            (self.get_styles_len() as u32 * 4);

        out.write_u32::<LittleEndian>(self.strings.len() as u32)?;
        out.write_u32::<LittleEndian>(self.styles.len() as u32)?;
        out.write_u32::<LittleEndian>(flags)?;

        // TODO: Calculate properly the offset as we are calculating on the decoding part
        let style_offset = 0;
        out.write_u32::<LittleEndian>(string_offset as u32)?;
        out.write_u32::<LittleEndian>(style_offset as u32)?;

        Ok(out)
    }

    fn get_body_data(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();

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


        // Encode strings and save offsets
        for string in &self.strings {
            string_offsets.push(current_offset);
            let mut encoded_string = Vec::new();
            let (size, error) = encoder.raw_feed(string, &mut encoded_string);

            if error.is_some() {
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
        for _ in &self.styles {
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
}

impl StringTable for StringTableBuf {
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
    use test::compare_chunks;
    use raw_chunks;

    #[test]
    fn it_can_generate_an_empty_chunk() {
        let string_table = StringTableBuf::default();

        assert_eq!(0, string_table.get_strings_len());
        assert_eq!(0, string_table.get_styles_len());
        assert!(string_table.get_string(0).is_err());
    }

    #[test]
    fn it_can_generate_a_chunk_with_the_given_data() {
        let mut string_table = StringTableBuf::default();
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
        let raw = raw_chunks::EXAMPLE_STRING_TABLE;
        let wrapper = StringTableWrapper::new(&raw);

        let owned = wrapper.to_buffer().unwrap();
        let owned_as_vec = owned.to_vec().unwrap();

        compare_chunks(&owned_as_vec, &raw);
    }

    #[test]
    fn identity_utf8() {
        // TODO: Test with UTF8 encoding
    }
}
