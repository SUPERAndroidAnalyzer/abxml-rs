use chunks::{Chunk, ChunkLoaderStream, ChunkHeader};
use chunks::table_type::Entry;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use document::{HeaderStringTable, Value};
use errors::*;

pub struct PackageDecoder;

impl PackageDecoder {
    pub fn decode<'a>(cursor: &mut Cursor<&'a [u8]>, header: &ChunkHeader)  -> Result<Chunk<'a>> {
        let pw = PackageWrapper::new(cursor.get_ref(), (*header).clone());

        Ok(Chunk::Package(pw))
    }

    /*fn get_string_table(chunk: Chunk) -> Option<Rc<StringTable>> {
        match chunk {
            Chunk::StringTable(st) => Some(st.clone()),
            _ => None,
        }
    }*/

    fn package_name(cursor: &mut Cursor<&[u8]>) -> Result<String> {
        let initial_position = cursor.position();
        let raw_str = cursor.get_ref()[initial_position as usize..(initial_position+256) as usize].to_vec();
        String::from_utf8(raw_str).chain_err(|| "Could not convert to UTF-8")
    }
}

pub struct PackageWrapper<'a> {
    raw_data: &'a [u8],
    header: ChunkHeader,
}

impl<'a> PackageWrapper<'a> {
    pub fn new(raw_data: &'a [u8], header: ChunkHeader) -> Self {
        PackageWrapper {
            raw_data: raw_data,
            header: header,
        }
    }

    pub fn get_id(&self) -> u32 {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(8));

        cursor.read_u32::<LittleEndian>().unwrap()
    }

    pub fn get_name(&self) -> Result<String> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(12));

        let initial_position = cursor.position();
        let raw_str = cursor.get_ref()[initial_position as usize..(initial_position+256) as usize].to_vec();

        String::from_utf8(raw_str).chain_err(|| "Could not convert to UTF-8")
    }
}

pub struct Package<'a> {
    wrapper: PackageWrapper<'a>,
}

impl<'a> Package<'a> {
    pub fn new(wrapper: PackageWrapper<'a>) -> Self {
        Package {
            wrapper: wrapper,
        }
    }

    pub fn get_id(&self) -> u32 {
        self.wrapper.get_id()
    }

    pub fn get_name(&self) -> Result<String>{
        self.wrapper.get_name()
    }
}
