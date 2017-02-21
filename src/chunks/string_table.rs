use chunks::{Chunk, ChunkHeader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use errors::*;
use std::clone::Clone;
use std::collections::hash_map::{HashMap, Entry};
use std::fmt::{Display, Formatter};
use std::result::Result as StdResult;
use std::fmt::Error as FmtError;
use model::StringTable as StringTableTrait;
use encoding::codec::{utf_16, utf_8};
use model::owned::{StringTableBuf, Encoding as EncodingType};

pub struct StringTableDecoder;

impl StringTableDecoder {
    pub fn decode<'a>(cursor: &mut Cursor<&'a [u8]>, header: &ChunkHeader)  -> Result<Chunk<'a>> {
         let stw = StringTableWrapper::new(cursor.get_ref(), *header);

         Ok(Chunk::StringTable(stw))
     }
}

pub struct StringTableWrapper<'a> {
    raw_data: &'a [u8],
    header: ChunkHeader,
}

impl<'a> StringTableWrapper<'a> {
    pub fn new(raw_data: &'a [u8], header: ChunkHeader) -> Self {
        StringTableWrapper {
            raw_data: raw_data,
            header: header,
        }
    }

    pub fn get_strings_len(&self) -> u32 {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(8));

        cursor.read_u32::<LittleEndian>().unwrap_or(0)
    }

    pub fn get_styles_len(&self) -> u32 {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(12));

        cursor.read_u32::<LittleEndian>().unwrap_or(0)
    }

    pub fn get_flags(&self) -> u32 {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(16));

        cursor.read_u32::<LittleEndian>().unwrap_or(0)
    }

    pub fn get_string(&self, idx: u32) -> Result<String> {
        let amount = self.get_strings_len();
        if idx > amount {
            return Err("Trying to get index outside StringTable".into());
        }

        self.get_string_position(idx).and_then(|position| self.parse_string(position as u32))
    }

    pub fn to_owned(self) -> Result<StringTableBuf> {
        let mut owned = StringTableBuf::new();

        if !self.is_utf8() {
            owned.set_encoding(EncodingType::Utf16);
        }

        println!("Strings: {}", self.get_strings_len());

        for i in 0..self.get_strings_len() {
            owned.add_string(self.get_string(i)?);
        }

        Ok(owned)
    }

    fn get_string_position(&self, idx: u32) -> Result<u64> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(20));
        let str_offset = self.header.get_offset() as u32 + cursor.read_u32::<LittleEndian>()?;

        cursor.set_position(self.header.absolute(28));

        let mut position = str_offset;
        let mut max_offset = 0;

        for _ in 0..(idx + 1) {
            let current_offset = cursor.read_u32::<LittleEndian>()?;
            position = str_offset + current_offset;

            if current_offset > max_offset {
                max_offset = current_offset
            }
        }

        Ok(position as u64)
    }

    fn parse_string(&self, offset: u32) -> Result<String> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(offset as u64);

        if self.is_utf8() {
            let mut ini_offset = offset;
            let v = cursor.read_u8()? as u32;
            if v == 0x80 {
                ini_offset += 2;
                cursor.read_u8()?;
            } else {
                ini_offset += 1;
            }

            let v = cursor.read_u8()? as u32;
            if v == 0x80 {
                ini_offset += 2;
                cursor.read_u8()?;
            } else {
                ini_offset += 1;
            }

            let mut length = 0;

            loop {
                let v = cursor.read_u8()?;

                if v != 0 {
                    length += 1;
                } else {
                    break;
                }
            }

            let a = ini_offset;
            let b = ini_offset + length;

            if a > self.raw_data.len() as u32 || b > self.raw_data.len() as u32 || a > b {
                return Err("Sub-slice out of raw_data range".into());
            }

            let subslice: &[u8] = &self.raw_data[a as usize..b as usize];

            let mut decoder = utf_8::UTF8Decoder::new();
            let mut o = String::new();
            decoder.raw_feed(&subslice, &mut o);
            let decode_error = decoder.raw_finish(&mut o);

            match decode_error {
                None => Ok(o),
                Some(_) => Err("Error decoding UTF8 string".into()),
            }
        } else {
            let size1 = cursor.read_u8()? as u32;
            let size2 = cursor.read_u8()? as u32;

            let val = ((size2 & 0xFF) << 8) | size1 & 0xFF;

            let a = offset + 2;
            let b = offset + 2 + (val * 2);


            if a > self.raw_data.len() as u32 || b > self.raw_data.len() as u32 || a > b {
                return Err("Sub-slice out of raw_data range".into());
            }

            let subslice: &[u8] = &self.raw_data[a as usize..b as usize];

            let mut decoder = utf_16::UTF16Decoder::<utf_16::Little>::new();
            let mut o = String::new();
            decoder.raw_feed(&subslice, &mut o);
            let decode_error = decoder.raw_finish(&mut o);

            match decode_error {
                None => Ok(o),
                Some(_) => Err("Error decoding UTF16 string".into()),
            }
        }
    }

    fn is_utf8(&self) -> bool {
        (self.get_flags() & 0x00000100) == 0x00000100
    }
}

pub struct StringTable<'a> {
    wrapper: StringTableWrapper<'a>,
    cache: HashMap<u32, Rc<String>>,
}

impl<'a> Display for StringTable<'a> {
    fn fmt(&self, formatter: &mut Formatter) -> StdResult<(), FmtError> {
        let amount = self.get_strings_len();

        for i in 0..amount {
            write!(formatter, "{} - {}\n", i, self.get_uncached_string(i).unwrap_or(Rc::new("<UNKOWN>".to_string())))?;
        }

        Ok(())
    }
}

impl<'a> StringTableTrait for StringTable<'a> {
    fn get_strings_len(&self) -> u32 {
        self.wrapper.get_strings_len()
    }

    fn get_styles_len(&self) -> u32 {
        self.wrapper.get_styles_len()
    }

    fn get_string(&self, idx: u32) -> Result<Rc<String>> {
        if idx > self.get_strings_len() {
            return Err("Index out of bounds".into());
        }

        let string = self.wrapper.get_string(idx)?;
        Ok(Rc::new(string))
    }
}

impl<'a> StringTable <'a> {
    pub fn new(wrapper: StringTableWrapper<'a>) -> Self {
        StringTable {
            wrapper: wrapper,
            cache: HashMap::new(),
        }
    }

    pub fn get_string(&mut self, idx: u32) -> Result<Rc<String>> {
        // TODO: THinkf about how to be able to cache this. Check serde or serde_json to check how they did it
        if idx > self.get_strings_len() {
            return Err("Index out of bounds".into());
        }

        let rc_string = match self.cache.entry(idx) {
            Entry::Vacant(entry) => {
                let string = self.wrapper.get_string(idx)?;
                let rc_string = Rc::new(string);

                entry.insert(rc_string.clone());

                rc_string.clone()
            },
            Entry::Occupied(entry) => {
                entry.get().clone()
            },
        };

        Ok(rc_string)
    }

    pub fn get_uncached_string(&self, idx: u32) -> Result<Rc<String>> {
        let string = self.wrapper.get_string(idx)?;
        Ok(Rc::new(string))
    }
}
