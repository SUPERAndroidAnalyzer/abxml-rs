use chunks::{Chunk, ChunkHeader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use document::{HeaderStringTable, StringTable};
use errors::*;
use parser::Decoder;
use std::clone::Clone;

pub struct StringTableDecoder;

impl StringTableDecoder {
    pub fn decode(decoder: &mut Decoder, cursor: &mut Cursor<&[u8]>, header: &ChunkHeader)  -> Result<Chunk> {
         let mut header_string_table = HeaderStringTable::default();

         header_string_table.string_amount = cursor.read_u32::<LittleEndian>()?;
         header_string_table.style_amount = cursor.read_u32::<LittleEndian>()?;
         header_string_table.flags = cursor.read_u32::<LittleEndian>()?;
         header_string_table.string_offset = cursor.read_u32::<LittleEndian>()?;
         header_string_table.style_offset = cursor.read_u32::<LittleEndian>()?;

         let mut string_table = StringTable::default();
         let str_offset = header.get_offset() as u32 + header_string_table.string_offset;

         let mut max_offset = 0;

         for i in 0..header_string_table.string_amount {
             let current_offset = cursor.read_u32::<LittleEndian>()?;
             let position = str_offset + current_offset;
             let s = Self::parse_string(cursor.get_ref(), position, true).unwrap_or(String::new());
             string_table.strings.push(Rc::new(s));

             if current_offset > max_offset {
                 max_offset = current_offset
             }
         }
         
         let ref_count_st = Rc::new(string_table);
         println!("String table with {} elements", header_string_table.string_amount);

         Ok(Chunk::StringTable(ref_count_st.clone()))
     }

     fn parse_string(raw_data: &[u8], offset: u32, utf8: bool) -> Result<String> {
         let mut final_offset = offset;

         let size1: u32 = raw_data[offset as usize] as u32;
         let size2: u32 = raw_data[(offset + 1) as usize] as u32;

         if size1 == size2 {
             let str_len = size1;
             let position = offset + 2;
             let a = position;
             let b = position + str_len;

             let subslice: &[u8] = &raw_data[a as usize..b as usize];

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

             let subslice: &[u8] = &raw_data[a as usize..b as usize];

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
