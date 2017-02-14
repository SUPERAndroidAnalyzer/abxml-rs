use chunks::*;
use model::StringTable as StringTableTrait;

use super::ChunkVisitor;
use super::Origin;

#[allow(dead_code)]
pub struct PrintVisitor;

impl<'a> ChunkVisitor<'a> for PrintVisitor {
    fn visit_string_table(&mut self, string_table: StringTable, origin: Origin) {
        println!("String Table!");
        println!("\tLength ({:?}): {} ", origin, string_table.get_strings_len());
    }

    fn visit_package(&mut self, package: Package) {
        println!("Package!");
        println!("\tId: {}", package.get_id());
        println!("\tName: {}", package.get_name().unwrap());
    }

    fn visit_table_type(&mut self, table_type: TableType) {
        println!("Table type!");
        println!("\tId: {}", table_type.get_id());
    }

    fn visit_type_spec(&mut self, type_spec: TypeSpec) {
        println!("Type spec!");
        println!("\tId: {}", type_spec.get_id());
    }
}
