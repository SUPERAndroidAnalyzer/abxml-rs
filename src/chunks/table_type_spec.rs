use chunks::{Chunk, ChunkHeader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;
use model::TypeSpec as TypeSpecTrait;

pub struct TableTypeSpecDecoder;

impl TableTypeSpecDecoder {
    pub fn decode<'a>(cursor: &mut Cursor<&'a [u8]>, header: &ChunkHeader) -> Result<Chunk<'a>> {
        let tsw = TypeSpecWrapper::new(cursor.get_ref(), *header);
        Ok(Chunk::TableTypeSpec(tsw))
    }
}

#[derive(Clone)]
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

    pub fn get_id(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(8));

        Ok(cursor.read_u32::<LittleEndian>()?)
    }
}

#[derive(Clone)]
pub struct TypeSpec<'a> {
    wrapper: TypeSpecWrapper<'a>,
}

impl<'a> TypeSpec<'a> {
    pub fn new(wrapper: TypeSpecWrapper<'a>) -> Self {
        TypeSpec { wrapper: wrapper }
    }
}

impl<'a> TypeSpecTrait for TypeSpec<'a> {
    fn get_id(&self) -> Result<u16> {
        Ok((self.wrapper.get_id()? & 0xFF) as u16)
    }
}
