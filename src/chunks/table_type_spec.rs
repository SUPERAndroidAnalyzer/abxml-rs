use chunks::{Chunk, ChunkHeader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use document::{HeaderStringTable};
use errors::*;

pub struct TableTypeSpecDecoder;

impl TableTypeSpecDecoder {
    pub fn decode<'a>(cursor: &mut Cursor<&'a [u8]>, header: &ChunkHeader)  -> Result<Chunk<'a>> {
        let tsw = TypeSpecWrapper::new(cursor.get_ref(), (*header).clone());
        Ok(Chunk::TableTypeSpec(tsw))
    }
}

pub struct TypeSpecWrapper<'a> {
    raw_data: &'a [u8],
    header: ChunkHeader,
}

impl<'a> TypeSpecWrapper<'a> {
    pub fn new(raw_data: &'a [u8], header: ChunkHeader) -> Self {
        TypeSpecWrapper {
            raw_data: raw_data,
            header: header,
        }
    }

    pub fn get_id(&self) -> u32 {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(8));

        cursor.read_u32::<LittleEndian>().unwrap()
    }
}

pub struct TypeSpec<'a> {
    wrapper: TypeSpecWrapper<'a>,
}

impl<'a> TypeSpec<'a> {
    pub fn new(wrapper: TypeSpecWrapper<'a>) -> Self {
        TypeSpec {
            wrapper: wrapper,
        }
    }

    pub fn get_id(&self) -> u16 {
        (self.wrapper.get_id() & 0xFF) as u16
    }
}
