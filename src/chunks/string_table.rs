use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use errors::*;
use model::StringTable;
use encoding::codec::{utf_16, utf_8};
use model::owned::{StringTableBuf, Encoding as EncodingType};
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

pub struct StringTableWrapper<'a> {
    raw_data: &'a [u8],
}

impl<'a> StringTableWrapper<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        StringTableWrapper { raw_data: slice }
    }

    pub fn get_flags(&self) -> u32 {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(16);

        cursor.read_u32::<LittleEndian>().unwrap_or(0)
    }

    pub fn to_buffer(&self) -> Result<StringTableBuf> {
        let mut owned = StringTableBuf::default();

        if !self.is_utf8() {
            owned.set_encoding(EncodingType::Utf16);
        }

        for i in 0..self.get_strings_len() {
            let string = &*self.get_string(i)?;
            owned.add_string(string.clone());
        }

        Ok(owned)
    }

    fn get_string_position(&self, idx: u32) -> Result<u64> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(20);
        let str_offset = cursor.read_u32::<LittleEndian>()?;

        cursor.set_position(28);

        let mut position = str_offset;
        let mut max_offset = 0;

        for _ in 0..(idx + 1) {
            let current_offset = cursor.read_u32::<LittleEndian>()?;
            position = str_offset.wrapping_add(current_offset);

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
            decoder.raw_feed(subslice, &mut o);
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
            decoder.raw_feed(subslice, &mut o);
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

impl<'a> StringTable for StringTableWrapper<'a> {
    fn get_strings_len(&self) -> u32 {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(8);

        cursor.read_u32::<LittleEndian>().unwrap_or(0)
    }

    fn get_styles_len(&self) -> u32 {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(12);

        cursor.read_u32::<LittleEndian>().unwrap_or(0)
    }

    fn get_string(&self, idx: u32) -> Result<Rc<String>> {
        if idx > self.get_strings_len() {
            return Err("Index out of bounds".into());
        }

        let string = self.get_string_position(idx).and_then(|position| {
            self.parse_string(position as u32)
        })?;

        Ok(Rc::new(string))
    }
}

pub struct StringTableCache<S: StringTable> {
    inner: S,
    cache: RefCell<HashMap<u32, Rc<String>>>,
}

impl<S: StringTable> StringTableCache<S> {
    pub fn new(inner: S) -> Self {
        StringTableCache {
            inner: inner,
            cache: RefCell::new(HashMap::new()),
        }
    }
}

impl<S: StringTable> StringTable for StringTableCache<S> {
    fn get_strings_len(&self) -> u32 {
        self.inner.get_strings_len()
    }

    fn get_styles_len(&self) -> u32 {
        self.inner.get_styles_len()
    }

    fn get_string(&self, idx: u32) -> Result<Rc<String>> {
        let mut cache = self.cache.borrow_mut();
        let entry = cache.entry(idx);

        match entry {
            Vacant(entry) => {
                let string_ref = self.inner.get_string(idx)?;
                entry.insert(string_ref.clone());

                Ok(string_ref)
            }
            Occupied(entry) => Ok(entry.get().clone()),
        }
    }
}
