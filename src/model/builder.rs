use model::owned::OwnedBuf;
use byteorder::{LittleEndian, WriteBytesExt};
use chunks::*;
use errors::*;

pub struct Arsc {
    chunks: Vec<Box<OwnedBuf>>,
}

impl Arsc {
    pub fn new() -> Self {
        Arsc {
            chunks: Vec::new(),
        }
    }

    pub fn push_owned(&mut self, chunk: Box<OwnedBuf>) {
        self.chunks.push(chunk);
    }

    pub fn to_vec(self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        let mut inner = Vec::new();
        let mut file_size = 0;

        for c in self.chunks {
            let encoded_chunk = c.to_vec().chain_err(|| "Could not encode a chunk")?;
            file_size += encoded_chunk.len();

            inner.extend(encoded_chunk);
        }

        // TODO: Check initial token
        // Token
        out.write_u16::<LittleEndian>(0)?;

        // Header_size
        out.write_u16::<LittleEndian>(3*4)?;

        // Chunk size
        out.write_u32::<LittleEndian>(file_size as u32)?;

        // TODO: Review this value
        // Package amount
        out.write_u32::<LittleEndian>(0)?;

        out.extend(inner);

        Ok(out)
    }
}

pub struct Xml {
    chunks: Vec<Box<OwnedBuf>>,
}

impl Xml {
    pub fn new() -> Self {
        Xml {
            chunks: Vec::new(),
        }
    }

    pub fn push_owned(&mut self, chunk: Box<OwnedBuf>) {
        self.chunks.push(chunk);
    }

    pub fn to_vec(self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        let mut inner = Vec::new();
        let mut file_size = 0;

        for c in self.chunks {
            let encoded_chunk = c.to_vec().chain_err(|| "Could not encode a chunk")?;
            file_size += encoded_chunk.len();

            inner.extend(encoded_chunk);
        }

        // TODO: Check initial token
        // Token
        out.write_u16::<LittleEndian>(0)?;

        // Header_size
        out.write_u16::<LittleEndian>(3*4)?;

        // Chunk size
        out.write_u32::<LittleEndian>(file_size as u32)?;

        out.extend(inner);

        Ok(out)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use chunks::*;
    use visitor::Executor;
    use visitor::ChunkVisitor;
    use visitor::Origin;
    use std::io::Cursor;
    use model::owned::*;

    pub struct CounterChunkVisitor {
        count: u32,
    }

    impl CounterChunkVisitor {
        pub fn new() -> CounterChunkVisitor {
            CounterChunkVisitor {
                count: 0,
            }
        }

        pub fn get_count(&self) -> u32 {
            self.count
        }
    }

    impl<'a> ChunkVisitor<'a> for CounterChunkVisitor {
        fn visit_string_table(&mut self, _string_table: StringTable<'a>, _origin: Origin) {
            self.count += 1
        }
        fn visit_package(&mut self, _package: PackageRef<'a>) {
            self.count += 1
        }
        fn visit_table_type(&mut self, _table_type: TableType<'a>) {
            self.count += 1
        }
        fn visit_type_spec(&mut self, _type_spec: TypeSpec<'a>) {
            self.count += 1
        }
        fn visit_xml_namespace_start(&mut self, _namespace_start: XmlNamespaceStart<'a>) {
            self.count += 1
        }
        fn visit_xml_namespace_end(&mut self, _namespace_end: XmlNamespaceEnd<'a>) {
            self.count += 1
        }
        fn visit_xml_tag_start(&mut self, _tag_start: XmlTagStart<'a>) {
            self.count += 1
        }
        fn visit_xml_tag_end(&mut self, _tag_end: XmlTagEnd<'a>) {
            self.count += 1
        }
        fn visit_xml_text(&mut self, _text: XmlText<'a>) {
            self.count += 1
        }
        fn visit_resource(&mut self, _resource: Resource<'a>) {
            self.count += 1
        }
    }

    #[test]
    fn it_can_generate_a_resources_arsc_file_content() {
        let arsc = Arsc::new();
        let content = arsc.to_vec().unwrap();
        let mut visitor = CounterChunkVisitor::new();

        assert_eq!(vec![0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0], content);

        Executor::arsc(Cursor::new(&content), &mut visitor);

        assert_eq!(0, visitor.get_count());
    }

    #[test]
    fn it_can_generate_a_resources_arsc_file_content_with_some_chunks() {
        let mut arsc = Arsc::new();

        arsc.push_owned(Box::new(StringTableBuf::new()));
        arsc.push_owned(Box::new(StringTableBuf::new()));
        arsc.push_owned(Box::new(ResourceBuf::new()));

        let content = arsc.to_vec().unwrap();
        let mut visitor = CounterChunkVisitor::new();

        Executor::arsc(Cursor::new(&content), &mut visitor);

        // Resource should be ignored as it is not a chunk that appears on an ARSC
        assert_eq!(2, visitor.get_count());
    }

    #[test]
    fn it_can_generate_a_resources_xml_file_content() {
        let xml = Xml::new();
        let content = xml.to_vec().unwrap();
        let mut visitor = CounterChunkVisitor::new();

        assert_eq!(vec![0, 0, 12, 0, 0, 0, 0, 0], content);

        Executor::arsc(Cursor::new(&content), &mut visitor);

        assert_eq!(0, visitor.get_count());
    }

    #[test]
    fn it_can_generate_a_resources_xml_file_content_with_some_chunks() {
        /*
        let mut xml = Xml::new();

        xml.push_owned(Box::new(StringTableBuf::new()));
        xml.push_owned(Box::new(StringTableBuf::new()));
        xml.push_owned(Box::new(ResourceBuf::new()));

        let content = xml.to_vec().unwrap();
        let mut visitor = CounterChunkVisitor::new();

        Executor::arsc(Cursor::new(&content), &mut visitor);

        // Resource should be ignored as it is not a chunk that appears on an ARSC
        assert_eq!(1, visitor.get_count());
        */
    }
}