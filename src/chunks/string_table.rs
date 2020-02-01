use crate::model::{
    owned::{Encoding as EncodingType, StringTableBuf},
    StringTable,
};
use anyhow::{bail, ensure, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use encoding::codec::{utf_16, utf_8};
use std::{
    cell::RefCell,
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap,
    },
    io::Cursor,
    rc::Rc,
};

#[derive(Debug)]
pub struct StringTableWrapper<'a> {
    raw_data: &'a [u8],
}

impl<'a> StringTableWrapper<'a> {
    pub fn new(raw_data: &'a [u8]) -> Self {
        Self { raw_data }
    }

    pub fn flags(&self) -> u32 {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(16);

        cursor.read_u32::<LittleEndian>().unwrap_or(0)
    }

    pub fn to_buffer(&self) -> Result<StringTableBuf> {
        let mut owned = StringTableBuf::default();

        if !self.is_utf8() {
            owned.set_encoding(EncodingType::Utf16);
        }

        for i in 0..self.strings_len() {
            let string = &*self.get_string(i)?;
            owned.add_string(string.clone());
        }

        Ok(owned)
    }

    fn string_position(&self, idx: u32) -> Result<u64> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(20);
        let str_offset = cursor.read_u32::<LittleEndian>()?;

        cursor.set_position(28);

        let mut position = str_offset;
        let mut max_offset = 0;

        for _ in 0..=idx {
            let current_offset = cursor.read_u32::<LittleEndian>()?;
            position = str_offset.wrapping_add(current_offset);

            if current_offset > max_offset {
                max_offset = current_offset
            }
        }

        Ok(u64::from(position))
    }

    fn parse_string(&self, offset: u32) -> Result<String> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(u64::from(offset));

        if self.is_utf8() {
            let mut ini_offset = offset;
            let v = u32::from(cursor.read_u8()?);
            if v == 0x80 {
                ini_offset += 2;
                cursor.read_u8()?;
            } else {
                ini_offset += 1;
            }

            let v = u32::from(cursor.read_u8()?);
            if v == 0x80 {
                ini_offset += 2;
                cursor.read_u8()?;
            } else {
                ini_offset += 1;
            }

            let mut length = 0;

            loop {
                let v = cursor.read_u8()?;

                if v == 0 {
                    break;
                } else {
                    length += 1;
                }
            }

            let a = ini_offset;
            let b = ini_offset + length;

            ensure!(
                a <= self.raw_data.len() as u32 && b <= self.raw_data.len() as u32 && a <= b,
                "sub-slice out of raw_data range"
            );

            let subslice: &[u8] = &self.raw_data[a as usize..b as usize];

            let mut decoder = utf_8::UTF8Decoder::new();
            let mut o = String::new();
            decoder.raw_feed(subslice, &mut o);
            let decode_error = decoder.raw_finish(&mut o);

            if decode_error.is_none() {
                Ok(o)
            } else {
                bail!("error decoding UTF8 string")
            }
        } else {
            let size1 = u32::from(cursor.read_u8()?);
            let size2 = u32::from(cursor.read_u8()?);

            let val = ((size2 & 0xFF) << 8) | size1 & 0xFF;

            let a = offset + 2;
            let b = a + val * 2;

            ensure!(
                a <= self.raw_data.len() as u32 && b <= self.raw_data.len() as u32 && a <= b,
                "sub-slice out of raw_data range"
            );

            let subslice: &[u8] = &self.raw_data[a as usize..b as usize];

            let mut decoder = utf_16::UTF16Decoder::<utf_16::Little>::new();
            let mut o = String::new();
            decoder.raw_feed(subslice, &mut o);
            let decode_error = decoder.raw_finish(&mut o);

            if decode_error.is_some() {
                bail!("error decoding UTF16 string");
            }
            Ok(o)
        }
    }

    fn is_utf8(&self) -> bool {
        (self.flags() & 0x0000_0100) == 0x0000_0100
    }
}

impl<'a> StringTable for StringTableWrapper<'a> {
    fn strings_len(&self) -> u32 {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(8);

        cursor.read_u32::<LittleEndian>().unwrap_or(0)
    }

    fn styles_len(&self) -> u32 {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(12);

        cursor.read_u32::<LittleEndian>().unwrap_or(0)
    }

    fn get_string(&self, idx: u32) -> Result<Rc<String>> {
        ensure!(idx <= self.strings_len(), "index out of bounds");

        let string = self
            .string_position(idx)
            .and_then(|position| self.parse_string(position as u32))?;

        Ok(Rc::new(string))
    }
}

#[derive(Debug)]
pub struct StringTableCache<S: StringTable> {
    inner: S,
    cache: RefCell<HashMap<u32, Rc<String>>>,
}

impl<S: StringTable> StringTableCache<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            cache: RefCell::new(HashMap::new()),
        }
    }
}

impl<S: StringTable> StringTable for StringTableCache<S> {
    fn strings_len(&self) -> u32 {
        self.inner.strings_len()
    }

    fn styles_len(&self) -> u32 {
        self.inner.styles_len()
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
