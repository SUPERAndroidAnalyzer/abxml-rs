use std::io::Cursor;
use chunks::*;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;
use std::marker::PhantomData;
use std::collections::HashMap;
use chunks::table_type::Entry;

pub trait ChunkVisitor<'a> {
    fn visit_string_table(&mut self, mut string_table: StringTable<'a>) {}
    fn visit_package(&mut self, mut package: Package<'a>) {}
    fn visit_table_type(&mut self, mut table_type: TableType<'a>) {}
    fn visit_type_spec(&mut self, mut type_spec: TypeSpec<'a>) {}
    fn visit_xml_namespace_start(&mut self, mut namespace_start: XmlNamespaceStart<'a>) {}
    fn visit_xml_namespace_end(&mut self, mut namespace_end: XmlNamespaceEnd<'a>) {}
    fn visit_xml_tag_start(&mut self, mut namespace_start: XmlTagStart<'a>) {}
    fn visit_xml_tag_end(&mut self, mut namespace_end: XmlTagEnd<'a>) {}
}

pub struct Executor;

impl Executor {
    pub fn arsc<'a, V: ChunkVisitor<'a>>(mut cursor: Cursor<&'a [u8]>, mut visitor: &mut V) -> Result<()> {
        let token = cursor.read_u16::<LittleEndian>()?;
        let header_size = cursor.read_u16::<LittleEndian>()?;
        let chunk_size = cursor.read_u32::<LittleEndian>()?;
        let package_amount = cursor.read_u32::<LittleEndian>()?;

        let stream = ChunkLoaderStream::new(cursor);

        for c in stream {
            match c {
                Chunk::StringTable(stw) => {
                    let mut st = StringTable::new(stw);
                    visitor.visit_string_table(st);
                },
                Chunk::Package(pw) => {
                    let mut package = Package::new(pw);
                    visitor.visit_package(package);
                },
                Chunk::TableType(ttw) => {
                    let mut tt = TableType::new(ttw);
                    visitor.visit_table_type(tt);
                },
                Chunk::TableTypeSpec(tsw) => {
                    let mut ts = TypeSpec::new(tsw);
                    visitor.visit_type_spec(ts);
                },
                _ => (),
            }
        }

        Ok(())
    }

    pub fn xml<'a, V: ChunkVisitor<'a>>(mut cursor: Cursor<&'a [u8]>, mut visitor: &mut V) -> Result<()> {
        let token = cursor.read_u16::<LittleEndian>()?;
        let header_size = cursor.read_u16::<LittleEndian>()?;
        let chunk_size = cursor.read_u32::<LittleEndian>()?;

        let stream = ChunkLoaderStream::new(cursor);

        for c in stream {
            match c {
                Chunk::StringTable(stw) => {
                    let mut st = StringTable::new(stw);
                    visitor.visit_string_table(st);
                },
                Chunk::Package(pw) => {
                    let mut package = Package::new(pw);
                    visitor.visit_package(package);
                },
                Chunk::TableType(ttw) => {
                    let mut tt = TableType::new(ttw);
                    visitor.visit_table_type(tt);
                },
                Chunk::TableTypeSpec(tsw) => {
                    let mut ts = TypeSpec::new(tsw);
                    visitor.visit_type_spec(ts);
                },
                Chunk::XmlNamespaceStart(xnsw) => {
                    let mut ts = XmlNamespaceStart::new(xnsw);
                    visitor.visit_xml_namespace_start(ts);
                },
                Chunk::XmlNamespaceEnd(xnsw) => {
                    let mut ts = XmlNamespaceEnd::new(xnsw);
                    visitor.visit_xml_namespace_end(ts);
                },
                Chunk::XmlTagStart(xnsw) => {
                    let mut ts = XmlTagStart::new(xnsw);
                    visitor.visit_xml_tag_start(ts);
                },
                Chunk::XmlTagEnd(xnsw) => {
                    let mut ts = XmlTagEnd::new(xnsw);
                    visitor.visit_xml_tag_end(ts);
                },
                _ => (),
            }
        }

        Ok(())
    }
}

pub struct DummyVisitor;

impl<'a> ChunkVisitor<'a> for DummyVisitor {}

pub struct PrintVisitor;

impl<'a> ChunkVisitor<'a> for PrintVisitor {
    fn visit_string_table(&mut self, mut string_table: StringTable) {
        println!("String Table!");
        println!("\tLength: {}", string_table.get_strings_len());
    }

    fn visit_package(&mut self, mut package: Package) {
        println!("Package!");
        println!("\tId: {}", package.get_id());
        println!("\tName: {}", package.get_name().unwrap());
    }

    fn visit_table_type(&mut self, mut table_type: TableType) {
        println!("Table type!");
        println!("\tId: {}", table_type.get_id());
    }

    fn visit_type_spec(&mut self, mut type_spec: TypeSpec) {
        println!("Type spec!");
        println!("\tId: {}", type_spec.get_id());
    }
}

pub struct ModelVisitor<'a> {
    main_string_table: Option<StringTable<'a>>,
    package: Option<Package<'a>>,
    current_spec: Option<TypeSpec<'a>>,
    package_mask: u32,
    entries: HashMap<u32, Entry>,
}

impl<'a> ModelVisitor<'a> {
    pub fn new() -> ModelVisitor<'a> {
        ModelVisitor {
            main_string_table: None,
            package: None,
            current_spec: None,
            package_mask: 0,
            entries: HashMap::new(),
        }
    }

    pub fn get_entries(&self) -> &HashMap<u32, Entry> {
        &self.entries
    }
}

impl<'a> ChunkVisitor<'a> for ModelVisitor<'a> {
    fn visit_string_table(&mut self, mut string_table: StringTable<'a>) {
        match self.main_string_table {
            Some(_) => {
                println!("Secondary table!");
            },
            None => {
                self.main_string_table = Some(string_table);
            },
        }
    }

    fn visit_package(&mut self, mut package: Package<'a>) {
        self.package_mask = package.get_id() << 24;
        println!("{:X} - {}", self.package_mask, self.package_mask);
        self.package = Some(package);
    }

    fn visit_table_type(&mut self, mut table_type: TableType<'a>) {
        // println!("Table type!");
        // println!("\tId: {}", table_type.get_id());
        // println!("ResourceConfig: {:?}", table_type.get_configuration());
        match self.current_spec {
            Some(ref ts) => {
                let mask = self.package_mask |
                    ((ts.get_id() as u32) << 16);
                // println!("Mask: {:X} - {}", mask, mask);
                let entries = table_type.get_entries(ts, mask).unwrap();
                self.entries.extend(entries);
                // println!("Entries: {:?}", entries);
            },
            None => (),
        }
    }

    fn visit_type_spec(&mut self, mut type_spec: TypeSpec<'a>) {
        match self.current_spec {
            Some(ref ts) => {
                println!("Previous type spec: {}", ts.get_id());
            },
            None => (),
        }

        self.current_spec = Some(type_spec);
    }
}
