use std::rc::Rc;

use failure::Error;

use chunks::*;
use model;
use visitor::{ChunkVisitor, Origin};

#[derive(Default, Debug, Copy, Clone)]
pub struct CounterChunkVisitor {
    count: u32,
}

impl CounterChunkVisitor {
    pub fn get_count(&self) -> u32 {
        self.count
    }
}

impl<'a> ChunkVisitor<'a> for CounterChunkVisitor {
    fn visit_string_table(&mut self, _string_table: StringTableWrapper<'a>, _origin: Origin) {
        self.count += 1
    }
    fn visit_package(&mut self, _package: PackageWrapper<'a>) {
        self.count += 1
    }
    fn visit_table_type(&mut self, _table_type: TableTypeWrapper<'a>) {
        self.count += 1
    }
    fn visit_type_spec(&mut self, _type_spec: TypeSpecWrapper<'a>) {
        self.count += 1
    }
    fn visit_xml_namespace_start(&mut self, _namespace_start: XmlNamespaceStartWrapper<'a>) {
        self.count += 1
    }
    fn visit_xml_namespace_end(&mut self, _namespace_end: XmlNamespaceEndWrapper<'a>) {
        self.count += 1
    }
    fn visit_xml_tag_start(&mut self, _tag_start: XmlTagStartWrapper<'a>) {
        self.count += 1
    }
    fn visit_xml_tag_end(&mut self, _tag_end: XmlTagEndWrapper<'a>) {
        self.count += 1
    }
    fn visit_xml_text(&mut self, _text: XmlTextWrapper<'a>) {
        self.count += 1
    }
    fn visit_resource(&mut self, _resource: ResourceWrapper<'a>) {
        self.count += 1
    }
}

pub fn compare_chunks(expected: &[u8], data: &[u8]) {
    if expected.len() != data.len() {
        eprintln!("Expected len: {}; Data len: {}", expected.len(), data.len());
    }

    let mut is_equal = true;

    let len = if expected.len() < data.len() {
        expected.len()
    } else {
        data.len()
    };

    for i in 0..len {
        if expected[i] != data[i] {
            eprintln!("Difference @{}: {} <-> {}", i, expected[i], data[i]);
            is_equal = false;
        }
    }

    assert!(is_equal);
}

#[derive(Debug, Copy, Clone)]
pub struct FakeStringTable;
impl model::StringTable for FakeStringTable {
    fn get_strings_len(&self) -> u32 {
        8
    }

    fn get_styles_len(&self) -> u32 {
        0
    }

    fn get_string(&self, idx: u32) -> Result<Rc<String>, Error> {
        match idx {
            0 => Ok(Rc::new("Zero".to_string())),
            11 => Ok(Rc::new("Ones".to_string())),
            22 => Ok(Rc::new("Twos".to_string())),
            33 => Ok(Rc::new("Threes".to_string())),
            44 => Ok(Rc::new("Fours".to_string())),
            123 => Ok(Rc::new("center".to_string())),
            456 => Ok(Rc::new("left".to_string())),
            789 => Ok(Rc::new("right".to_string())),
            _ => bail!("index out of bounds"),
        }
    }
}
