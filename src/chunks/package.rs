use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};
use encoding::codec::utf_16::{self, Little};
use failure::{ensure, format_err, Error};

#[derive(Debug)]
pub struct PackageWrapper<'a> {
    raw_data: &'a [u8],
}

impl<'a> PackageWrapper<'a> {
    pub fn new(raw_data: &'a [u8]) -> Self {
        Self { raw_data }
    }

    pub fn get_id(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(8);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    pub fn get_name(&self) -> Result<String, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(12);
        let initial_position = cursor.position();
        ensure!(
            ((initial_position + 256) as usize) < self.raw_data.len(),
            "cursor position out of bounds"
        );

        let final_position = self.find_end_position(initial_position as usize);

        ensure!(
            self.raw_data.len() >= (initial_position + 256) as usize,
            "not enough bytes to retrieve package name"
        );

        let raw_str = &cursor.get_ref()[initial_position as usize..final_position];
        let mut decoder = utf_16::UTF16Decoder::<Little>::new();
        let mut o = String::new();
        decoder.raw_feed(raw_str, &mut o);
        let decode_error = decoder.raw_finish(&mut o);

        match decode_error {
            None => Ok(o),
            Some(_) => Err(format_err!("error decoding UTF8 string")),
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
