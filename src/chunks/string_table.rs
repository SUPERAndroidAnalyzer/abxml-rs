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
use encode_unicode::Utf16Char;

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

    pub fn get_string(&self, idx: u32) -> Result<String> {
        let amount = self.get_strings_len();
        if idx > amount {
            return Err("Trying to get index outside StringTable".into());
        }

        self.get_string_position(idx).and_then(|position| self.parse_string(position as u32))
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

        let size1 = cursor.read_u8()? as u32;
        let size2 = cursor.read_u8()? as u32;

        if size1 == size2 {
            let str_len = size1;
            let position = offset + 2;
            let a = position;
            let b = position + str_len;

            if a > self.raw_data.len() as u32 || b > self.raw_data.len() as u32 || a > b {
                return Err("Sub-slice out of raw_data range".into());
            }

            let subslice: &[u8] = &self.raw_data[a as usize..b as usize];

            let raw_str: Vec<u8> = subslice.iter()
                .cloned()
                .collect();

           String::from_utf8(raw_str).chain_err(|| "Could not convert to UTF-8")
        } else {
            let val = ((size2 & 0xFF) << 8) | size1 & 0xFF;

            let (aa, bb) = if val == 0x8000 {
                let low = (cursor.read_u8()? as u32) & 0xFF;
                let high = ((cursor.read_u8()? & 0xFF) as u32) << 8;

                (4 as u32, ((low + high) * 2) as u32)
            } else {
                (2 as u32, (size1 * 2) as u32)
            };

            let a = offset + aa;
            let b = offset + bb;

            // println!("Range: {} <-> {} ;;; Offset: {}", a, b, offset);

            if a > self.raw_data.len() as u32 || b > self.raw_data.len() as u32 || a > b {
                return Err("Sub-slice out of raw_data range".into());
            }

            let subslice: &[u8] = &self.raw_data[a as usize..b as usize];
            let subslice_16: &[u16] = unsafe {::std::mem::transmute(subslice)};
            /*let mut i = 0;
            let raw_str: Vec<u8> = subslice.iter()
                .cloned()
                .filter(|_| {
                    let result = i % 2 == 0;
                    i += 1;

                    result
                })
                .collect();*/
            let (utf16char, _) = Utf16Char::from_slice(subslice_16)?;

            Ok(utf16char.to_char().to_string())

           // String::from_utf8(raw_str).chain_err(|| "Could not convert to UTF-8")
            // Err("UTF16 not implemented".into())
        }
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
