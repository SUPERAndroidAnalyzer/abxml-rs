use std::io::{Error, ErrorKind};
use chunks::{Chunk, ChunkHeader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use document::{HeaderStringTable, StringTable};

pub struct StringTableDecoder;

impl StringTableDecoder {
    pub fn decode(raw_data:&[u8], cursor: &mut Cursor<&[u8]>, header: &ChunkHeader)  -> Result<Chunk, Error> {
         let mut header_string_table = HeaderStringTable::default();

         header_string_table.string_amount = cursor.read_u32::<LittleEndian>()?;
         header_string_table.style_amount = cursor.read_u32::<LittleEndian>()?;
         header_string_table.flags = cursor.read_u32::<LittleEndian>()?;
         header_string_table.string_offset = cursor.read_u32::<LittleEndian>()?;
         header_string_table.style_offset = cursor.read_u32::<LittleEndian>()?;

         println!("Header: {:?}", header_string_table);

         let mut string_table = StringTable::default();
         let str_offset = header.get_offset() as u32 + header_string_table.string_offset;

         let mut max_offset = 0;

         for i in 0..header_string_table.string_amount {
             // println!("Position: {}", cursor.position());
             let current_offset = cursor.read_u32::<LittleEndian>()?;
             // println!("Offset: {}", current_offset);
             let position = str_offset + max_offset + current_offset;
             // println!("Position: {}", position);

             let s = Self::parse_string(raw_data, position, true).unwrap_or(String::new());

             // println!("String: {}", s);
             // println!("i: {} => {}", i, s);
             string_table.strings.push(Rc::new(s));

             if current_offset > max_offset {
                 max_offset = current_offset
             }

         }

         println!("Amount of strings");
         Ok(Chunk::StringTable(string_table))
     }

     fn parse_string(raw_data: &[u8], offset: u32, utf8: bool) -> Result<String, Error> {
         let mut final_offset = offset;
         //let val = raw_data[offset as usize] as u8;

         let size1: u32 = raw_data[offset as usize] as u32;
         let size2: u32 = raw_data[(offset + 1) as usize] as u32;

         if size1 == size2 || true {
             let str_len = size1;
             let position = offset + 2;
             let a = position;
             let b = position + str_len;

             let subslice: &[u8] = &raw_data[a as usize..b as usize];

             let raw_str: Vec<u8> = subslice.iter()
                 .cloned()
                 .collect();

             match String::from_utf8(raw_str) {
                 Ok(s) => Ok(s),
                 Err(e) => Err(Error::new(ErrorKind::Other, e)),
             }
         } else {
             let str_len = ((size2 << 8) & 0xFF00) | size1 & 0xFF;
             let position = offset + 2;
             let mut i = 0;
             let a = position;
             let b = position + (str_len * 2);

             let subslice: &[u8] = &raw_data[a as usize..b as usize];

             let raw_str: Vec<u8> = subslice.iter()
                 .cloned()
                 .filter(|_| {
                     let result = i % 2 == 0;
                     i = i + 1;

                     result
                 })
                 .collect();

             match String::from_utf8(raw_str) {
                 Ok(s) => Ok(s),
                 Err(e) => Err(Error::new(ErrorKind::Other, e)),
             }
         }
     }
}
