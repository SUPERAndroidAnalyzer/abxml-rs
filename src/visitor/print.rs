use super::{ChunkVisitor, Origin};
use crate::{
    chunks::{PackageWrapper, StringTableWrapper, TableTypeWrapper, TypeSpecWrapper},
    model::{StringTable, TableType, TypeSpec},
};

#[allow(dead_code)]
#[derive(Debug)]
pub struct PrintVisitor;

impl<'a> ChunkVisitor<'a> for PrintVisitor {
    fn visit_string_table(&mut self, string_table: StringTableWrapper, origin: Origin) {
        println!("String Table!");
        println!("\tLength ({:?}): {} ", origin, string_table.strings_len());
    }

    fn visit_package(&mut self, package: PackageWrapper) {
        println!("Package!");
        println!("\tId: {}", package.id().unwrap());
        println!("\tName: {}", package.name().unwrap());
    }

    fn visit_table_type(&mut self, table_type: TableTypeWrapper) {
        println!("Table type!");
        println!("\tId: {}", table_type.id().unwrap());
    }

    fn visit_type_spec(&mut self, type_spec: TypeSpecWrapper) {
        println!("Type spec!");
        println!("\tId: {}", type_spec.id().unwrap());
    }
}
