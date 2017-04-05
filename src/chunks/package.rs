use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;
use encoding::codec::utf_16;
use encoding::codec::utf_16::Little;

pub struct PackageWrapper<'a> {
    raw_data: &'a [u8],
}

impl<'a> PackageWrapper<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        PackageWrapper { raw_data: slice }
    }

    pub fn get_id(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(8);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    pub fn get_name(&self) -> Result<String> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(12);
        let initial_position = cursor.position();
        let final_position = self.find_end_position(initial_position as usize);

        let raw_str = &cursor.get_ref()[initial_position as usize..final_position];
        let mut decoder = utf_16::UTF16Decoder::<Little>::new();
        let mut o = String::new();
        decoder.raw_feed(raw_str, &mut o);
        let decode_error = decoder.raw_finish(&mut o);

        match decode_error {
            None => Ok(o),
            Some(_) => Err("Error decoding UTF8 string".into()),
        }
    }

    fn find_end_position(&self, initial_position: usize) -> usize {
        let buffer = &self.raw_data[initial_position..initial_position + 256];

        let mut zeros = 0;
        let mut i = 0;

        for c in buffer {
            if *c == 0 {
                zeros += 1;
            } else {
                zeros = 0;
            }

            if zeros > 1 {
                break;
            }

            i += 1;
        }

        initial_position + i
    }
}
