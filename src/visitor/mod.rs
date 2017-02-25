use std::io::Cursor;
use chunks::*;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;

mod xml;
pub mod model;
mod print;

pub use self::model::Resources as Resources;
pub use self::model::ModelVisitor as ModelVisitor;
pub use self::xml::XmlVisitor as XmlVisitor;
pub use self::model::RefPackage as RefPackage;

pub trait ChunkVisitor<'a> {
    fn visit_string_table(&mut self, _string_table: StringTable<'a>, _origin: Origin) {}
    fn visit_package(&mut self, _package: PackageRef<'a>) {}
    fn visit_table_type(&mut self, _table_type: TableType<'a>) {}
    fn visit_type_spec(&mut self, _type_spec: TypeSpec<'a>) {}
    fn visit_xml_namespace_start(&mut self, _namespace_start: XmlNamespaceStart<'a>) {}
    fn visit_xml_namespace_end(&mut self, _namespace_end: XmlNamespaceEnd<'a>) {}
    fn visit_xml_tag_start(&mut self, _tag_start: XmlTagStart<'a>) {}
    fn visit_xml_tag_end(&mut self, _tag_end: XmlTagEnd<'a>) {}
    fn visit_xml_text(&mut self, _text: XmlText<'a>) {}
    fn visit_resource(&mut self, _resource: Resource<'a>) {}
}

pub struct Executor;

impl Executor {
    pub fn arsc<'a, V: ChunkVisitor<'a>>(mut cursor: Cursor<&'a [u8]>, mut visitor: &mut V) -> Result<()> {
        let _token = cursor.read_u16::<LittleEndian>().chain_err(|| "Error reading first token")?;
        let _header_size = cursor.read_u16::<LittleEndian>().chain_err(|| "Error reading header size")?;
        let _chunk_size = cursor.read_u32::<LittleEndian>().chain_err(|| "Error reading chunk size")?;
        let _package_amount = cursor.read_u32::<LittleEndian>().chain_err(|| "Error reading package amount")?;
        cursor.set_position(_header_size as u64);

        let stream = ChunkLoaderStream::new(cursor);
        let mut origin = Origin::Global;

        for c in stream {
            match c.chain_err(|| "Error reading next chunk")? {
                Chunk::StringTable(stw) => {
                    let st = StringTable::new(stw);
                    visitor.visit_string_table(st, origin);
                    origin = Origin::next(origin);
                },
                Chunk::Package(pw) => {
                    let package = PackageRef::new(pw);
                    visitor.visit_package(package);
                },
                Chunk::TableType(ttw) => {
                    let tt = TableType::new(ttw);
                    visitor.visit_table_type(tt);
                },
                Chunk::TableTypeSpec(tsw) => {
                    let ts = TypeSpec::new(tsw);
                    visitor.visit_type_spec(ts);
                },
                _ => (),
            }
        }

        Ok(())
    }

    pub fn xml<'a, V: ChunkVisitor<'a>>(mut cursor: Cursor<&'a [u8]>, mut visitor: &mut V) -> Result<()> {
        let _token = cursor.read_u16::<LittleEndian>().chain_err(|| "Error reading first token")?;
        let _header_size = cursor.read_u16::<LittleEndian>().chain_err(|| "Error reading header size")?;
        let _chunk_size = cursor.read_u32::<LittleEndian>().chain_err(|| "Error reading chunk size")?;

        let stream = ChunkLoaderStream::new(cursor);
        let mut origin = Origin::Global;

        for c in stream {
            match c.chain_err(|| "Error reading next chunk")? {
                Chunk::StringTable(stw) => {
                    let st = StringTable::new(stw);
                    visitor.visit_string_table(st, origin);
                },
                Chunk::Package(pw) => {
                    let package = PackageRef::new(pw);
                    visitor.visit_package(package);
                },
                Chunk::TableType(ttw) => {
                    origin = Origin::Entries;
                    let tt = TableType::new(ttw);
                    visitor.visit_table_type(tt);
                },
                Chunk::TableTypeSpec(tsw) => {
                    origin = Origin::Spec;
                    let ts = TypeSpec::new(tsw);
                    visitor.visit_type_spec(ts);
                },
                Chunk::XmlNamespaceStart(xnsw) => {
                    let ts = XmlNamespaceStart::new(xnsw);
                    visitor.visit_xml_namespace_start(ts);
                },
                Chunk::XmlNamespaceEnd(xnsw) => {
                    let ts = XmlNamespaceEnd::new(xnsw);
                    visitor.visit_xml_namespace_end(ts);
                },
                Chunk::XmlTagStart(xnsw) => {
                    let ts = XmlTagStart::new(xnsw);
                    visitor.visit_xml_tag_start(ts);
                },
                Chunk::XmlTagEnd(xnsw) => {
                    let ts = XmlTagEnd::new(xnsw);
                    visitor.visit_xml_tag_end(ts);
                },
                Chunk::XmlText(xsnw) => {
                    let ts = XmlText::new(xsnw);
                    visitor.visit_xml_text(ts);
                }
                Chunk::Resource(rw) => {
                    let ts = Resource::new(rw);
                    visitor.visit_resource(ts);
                }
                _ => (),
            }
        }

        Ok(())
    }
}

pub struct DummyVisitor;

impl<'a> ChunkVisitor<'a> for DummyVisitor {}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Origin {
    Global,
    Spec,
    Entries,
}

impl Origin {
    pub fn next(origin: Origin) -> Origin {
        match origin {
            Origin::Global => Origin::Spec,
            Origin::Spec => Origin::Entries,
            Origin::Entries => Origin::Global,
        }
    }
}