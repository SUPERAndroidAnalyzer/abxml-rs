use chunks::{Chunk, ChunkHeader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;

pub struct PackageDecoder;

impl PackageDecoder {
    pub fn decode<'a>(cursor: &mut Cursor<&'a [u8]>, header: &ChunkHeader)  -> Result<Chunk<'a>> {
        let pw = PackageWrapper::new(cursor.get_ref(), *header);

        Ok(Chunk::Package(pw))
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
        let a: Vec<u8> = raw_str;
        let mut i = 0;
        let rw: Vec<u8> = a.iter()
            .cloned()
            .filter(|current| {
                let result = i % 2 == 0;
                i += 1;

                result && current != &0
            })
            .collect();

        String::from_utf8(rw).chain_err(|| "Could not convert to UTF-8")
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
