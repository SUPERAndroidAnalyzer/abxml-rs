use std::io::Cursor;
use chunks::*;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;

pub trait ChunkVisitor {
    fn visit_string_table(&mut self, mut string_table: StringTable) {}
    fn visit_package(&mut self, mut package: Package) {}
    fn visit_table_type(&mut self, mut table_type: TableType) {}
    fn visit_type_spec(&mut self, mut type_spec: TypeSpec) {}
}

pub struct Executor<V: ChunkVisitor> {
    visitor: V,
}

impl<V: ChunkVisitor> Executor<V> {
    pub fn arsc(mut cursor: Cursor<&[u8]>, mut visitor: V) -> Result<()> {
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
}

pub struct DummyVisitor;

impl ChunkVisitor for DummyVisitor {}

pub struct PrintVisitor;

impl ChunkVisitor for PrintVisitor {
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
