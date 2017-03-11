use chunks::*;
use visitor::ChunkVisitor;
use visitor::Origin;

pub struct CounterChunkVisitor {
    count: u32,
}

impl CounterChunkVisitor {
    pub fn new() -> CounterChunkVisitor {
        CounterChunkVisitor { count: 0 }
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

pub fn compare_chunks(expected: &[u8], data: &[u8]) {
    if expected.len() != data.len() {
        println!("Expected len: {}; Data len: {}", expected.len(), data.len());
    }

    let mut is_equal = true;

    let len = if expected.len() < data.len() {
        expected.len()
    } else {
        data.len()
    };

    for i in 0..len {
        if expected[i] != data[i] {
            println!("Difference @{}: {} <-> {}", i, expected[i], data[i]);
            is_equal = false;
        }
    }

    assert!(is_equal);
}