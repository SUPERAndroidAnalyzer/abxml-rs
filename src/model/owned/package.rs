use byteorder::{LittleEndian, WriteBytesExt};
use failure::Error;

use chunks::*;
use encoding::Encoding;
use encoding::codec::utf_16;
use model::owned::OwnedBuf;

#[derive(Default, Debug)]
pub struct PackageBuf {
    id: u32,
    package_name: String,
    inner_chunks: Vec<Box<OwnedBuf>>,
}

#[allow(dead_code)]
impl PackageBuf {
    pub fn new(id: u32, package_name: String) -> Result<Self, Error> {
        ensure!(
            package_name.as_bytes().len() <= 256,
            "can not create a package with a length greater than 256"
        );

        Ok(Self {
            id,
            package_name,
            inner_chunks: Vec::new(),
        })
    }

    pub fn add_chunk(&mut self, chunk: Box<OwnedBuf>) {
        self.inner_chunks.push(chunk);
    }
}

impl OwnedBuf for PackageBuf {
    fn get_token(&self) -> u16 {
        TOKEN_PACKAGE
    }

    fn get_body_data(&self) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();

        for c in &self.inner_chunks {
            let current_chunk = c.to_vec()?;
            out.extend(current_chunk);
        }

        Ok(out)
    }

    fn get_header(&self) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        let mut encoder = utf_16::UTF_16LE_ENCODING.raw_encoder();
        let mut encoded_string = Vec::new();
        let (size, error) = encoder.raw_feed(&self.package_name, &mut encoded_string);

        ensure!(error.is_none(), "error encoding package name as UTF-16");

        buffer.write_u32::<LittleEndian>(self.id)?;
        buffer.extend(encoded_string);

        // Padding package name up to 256 characters
        for _ in 0..(256 - size) {
            buffer.push(0);
        }

        // Padding (non-used data)
        buffer.write_u32::<LittleEndian>(0)?;
        buffer.write_u32::<LittleEndian>(0)?;

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chunks::{Chunk, ChunkLoaderStream, PackageWrapper};
    use model::StringTable;
    use model::owned::StringTableBuf;
    use std::io::Cursor;
    use std::iter;

    #[test]
    fn it_can_generate_a_chunk_with_the_given_data() {
        let some_other_chunk = PackageBuf::new(4, "com.test.test".to_string()).unwrap();
        let mut package = PackageBuf::new(3, "com.test.test".to_string()).unwrap();
        package.add_chunk(Box::new(some_other_chunk));
        let out = package.to_vec().unwrap();

        let wrapper = PackageWrapper::new(&out);

        assert_eq!(3, wrapper.get_id().unwrap());
        assert_eq!("com.test.test", wrapper.get_name().unwrap());
    }

    #[test]
    fn body_can_be_iterated_with_chunk_stream_loader() {
        let some_other_chunk = StringTableBuf::default();
        let mut inner_chunk_2 = StringTableBuf::default();
        inner_chunk_2.add_string("some string".to_string());
        inner_chunk_2.add_string("another string".to_string());

        let mut package = PackageBuf::new(3, "com.test.test".to_string()).unwrap();
        package.add_chunk(Box::new(some_other_chunk));
        package.add_chunk(Box::new(inner_chunk_2));

        let out = package.to_vec().unwrap();
        let cursor = Cursor::new(out.as_slice());
        let mut stream = ChunkLoaderStream::new(cursor);

        let first_chunk = stream.next().unwrap().unwrap();
        let second_chunk = stream.next().unwrap().unwrap();
        let third_chunk = stream.next().unwrap().unwrap();

        match first_chunk {
            Chunk::Package(_) => (),
            _ => panic!("First chunk should be a Package"),
        }

        match second_chunk {
            Chunk::StringTable(st) => {
                assert_eq!(st.get_strings_len(), 0);
            }
            _ => panic!("Second chunk should be a string table"),
        }

        match third_chunk {
            Chunk::StringTable(st) => {
                assert_eq!(st.get_strings_len(), 2);
            }
            _ => panic!("Second chunk should be string table"),
        }
    }

    #[test]
    fn it_can_not_create_a_package_with_a_too_large_package_name() {
        let target = iter::repeat('\u{1F624}')
            .take((256 / 4) + 1)
            .collect::<String>();
        let package = PackageBuf::new(1, target);

        assert!(package.is_err());
    }

    #[test]
    fn it_can_create_a_package_with_the_maximum_length() {
        let target = iter::repeat('\u{1F624}').take(256 / 4).collect::<String>();
        let package = PackageBuf::new(1, target);

        assert!(package.is_ok());
    }
}
