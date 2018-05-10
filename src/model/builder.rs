use byteorder::{LittleEndian, WriteBytesExt};
use failure::{Error, ResultExt};

use model::owned::OwnedBuf;

#[derive(Default, Debug)]
pub struct Arsc {
    chunks: Vec<Box<OwnedBuf>>,
}

impl Arsc {
    pub fn push_owned(&mut self, chunk: Box<OwnedBuf>) {
        self.chunks.push(chunk);
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();
        let mut inner = Vec::new();
        let mut file_size = 0;

        for c in &self.chunks {
            let encoded_chunk = c.to_vec().context("could not encode a chunk")?;
            file_size += encoded_chunk.len();

            inner.extend(encoded_chunk);
        }

        // Token
        out.write_u16::<LittleEndian>(2)?;

        // Header_size
        out.write_u16::<LittleEndian>(3 * 4)?;

        // Chunk size
        out.write_u32::<LittleEndian>(file_size as u32)?;

        // TODO: Review this value
        // Package amount
        out.write_u32::<LittleEndian>(0)?;

        out.extend(inner);

        Ok(out)
    }
}

#[derive(Default, Debug)]
pub struct Xml {
    chunks: Vec<Box<OwnedBuf>>,
}

impl Xml {
    pub fn push_owned(&mut self, chunk: Box<OwnedBuf>) {
        self.chunks.push(chunk);
    }

    pub fn into_vec(self) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();
        let mut inner = Vec::new();
        let mut file_size = 0;

        for c in self.chunks {
            let encoded_chunk = c.to_vec().context("could not encode a chunk")?;
            file_size += encoded_chunk.len();

            inner.extend(encoded_chunk);
        }

        // Token
        out.write_u16::<LittleEndian>(3)?;

        // Header_size
        out.write_u16::<LittleEndian>(2 * 4)?;

        // Chunk size
        out.write_u32::<LittleEndian>(file_size as u32)?;

        out.extend(inner);

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::owned::*;
    use std::io::Cursor;
    use test::*;
    use visitor::Executor;

    #[test]
    fn it_can_generate_a_resources_arsc_file_content() {
        let arsc = Arsc::default();
        let content = arsc.to_vec().unwrap();
        let mut visitor = CounterChunkVisitor::default();

        assert_eq!(vec![2, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0], content);

        Executor::arsc(&content, &mut visitor).unwrap();

        assert_eq!(0, visitor.get_count());
    }

    #[test]
    fn it_can_generate_a_resources_arsc_file_content_with_some_chunks() {
        let mut arsc = Arsc::default();

        arsc.push_owned(Box::new(StringTableBuf::default()));
        arsc.push_owned(Box::new(StringTableBuf::default()));
        arsc.push_owned(Box::new(ResourcesBuf::default()));

        let content = arsc.to_vec().unwrap();
        let mut visitor = CounterChunkVisitor::default();

        Executor::arsc(&content, &mut visitor).unwrap();

        // Resource should be ignored as it is not a chunk that appears on an ARSC
        assert_eq!(2, visitor.get_count());
    }

    #[test]
    fn it_can_generate_a_resources_xml_file_content() {
        let xml = Xml::default();
        let content = xml.into_vec().unwrap();
        let mut visitor = CounterChunkVisitor::default();

        assert_eq!(vec![3, 0, 8, 0, 0, 0, 0, 0], content);

        Executor::xml(Cursor::new(&content), &mut visitor).unwrap();

        assert_eq!(0, visitor.get_count());
    }

    #[test]
    fn it_can_generate_a_resources_xml_file_content_with_some_chunks() {
        let mut xml = Xml::default();

        xml.push_owned(Box::new(StringTableBuf::default()));
        xml.push_owned(Box::new(StringTableBuf::default()));
        xml.push_owned(Box::new(ResourcesBuf::default()));

        let content = xml.into_vec().unwrap();
        let mut visitor = CounterChunkVisitor::default();

        let _ = Executor::xml(Cursor::new(&content), &mut visitor);

        assert_eq!(3, visitor.get_count());
    }
}
