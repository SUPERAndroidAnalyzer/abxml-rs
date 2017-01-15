use chunks::{Chunk, ChunkHeader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use document::{HeaderStringTable};
use errors::*;
use std::clone::Clone;
use std::collections::hash_map::{HashMap, Entry};

pub struct StringTableDecoder;

impl StringTableDecoder {
    pub fn decode<'a>(cursor: &mut Cursor<&'a [u8]>, header: &ChunkHeader)  -> Result<Chunk<'a>> {
         let stw = StringTableWrapper::new(cursor.get_ref(), (*header).clone());

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

        cursor.read_u32::<LittleEndian>().unwrap()
    }

    pub fn get_styles_len(&self) -> u32 {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(12));

        cursor.read_u32::<LittleEndian>().unwrap()
    }

    pub fn get_string(&self, idx: u32) -> Result<String> {
        let amount = self.get_strings_len();
        if idx > amount {
            return Err("Trying to get index outside StringTable".into());
        }
        
        let position = self.get_string_position(idx);

        self.parse_string(position as u32)
    }

    fn get_string_position(&self, idx: u32) -> u64 {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(20));
        let str_offset = self.header.get_offset() as u32 + cursor.read_u32::<LittleEndian>().unwrap();

        cursor.set_position(self.header.absolute(28));

        let mut position = str_offset;
        let mut max_offset = 0;

        for i in 0..(idx + 1) {
            let current_offset = cursor.read_u32::<LittleEndian>().unwrap();
            position = str_offset + current_offset;

            if current_offset > max_offset {
                max_offset = current_offset
            }
        }

        position as u64
    }

    fn parse_string(&self, offset: u32) -> Result<String> {
        let mut final_offset = offset;

        let size1: u32 = self.raw_data[offset as usize] as u32;
        let size2: u32 = self.raw_data[(offset + 1) as usize] as u32;

        if size1 == size2 {
            let str_len = size1;
            let position = offset + 2;
            let a = position;
            let b = position + str_len;

            let subslice: &[u8] = &self.raw_data[a as usize..b as usize];

            let raw_str: Vec<u8> = subslice.iter()
                .cloned()
                .collect();

           String::from_utf8(raw_str).chain_err(|| "Could not convert to UTF-8")
        } else {
            let str_len = ((size2 << 8) & 0xFF00) | size1 & 0xFF;
            let position = offset + 2;
            let mut i = 0;
            let a = position;
            let b = position + (str_len * 2);

            let subslice: &[u8] = &self.raw_data[a as usize..b as usize];

            let raw_str: Vec<u8> = subslice.iter()
                .cloned()
                .filter(|_| {
                    let result = i % 2 == 0;
                    i = i + 1;

                    result
                })
                .collect();

           String::from_utf8(raw_str).chain_err(|| "Could not convert to UTF-8")
        }
    }
}

pub struct StringTable<'a> {
    wrapper: StringTableWrapper<'a>,
    cache: HashMap<u32, Rc<String>>,
}

impl<'a> StringTable <'a> {
    pub fn new(wrapper: StringTableWrapper<'a>) -> Self {
        StringTable {
            wrapper: wrapper,
            cache: HashMap::new(),
        }
    }

    pub fn get_strings_len(&self) -> u32 {
        self.wrapper.get_strings_len()
    }

    pub fn get_styles_len(&self) -> u32 {
        self.wrapper.get_styles_len()
    }

    pub fn get_string(&mut self, idx: u32) -> Result<Rc<String>> {
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
