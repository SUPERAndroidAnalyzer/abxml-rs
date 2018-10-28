//! Collection of visitors that are fed from chunk iterator
use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};
use failure::{bail, Error, ResultExt};

use crate::chunks::*;

pub mod model;
mod print;
mod xml;

pub use self::{
    model::{ModelVisitor, RefPackage, Resources},
    xml::XmlVisitor,
};

pub trait ChunkVisitor<'a> {
    fn visit_string_table(&mut self, _string_table: StringTableWrapper<'a>, _origin: Origin) {}
    fn visit_package(&mut self, _package: PackageWrapper<'a>) {}
    fn visit_table_type(&mut self, _table_type: TableTypeWrapper<'a>) {}
    fn visit_type_spec(&mut self, _type_spec: TypeSpecWrapper<'a>) {}
    fn visit_xml_namespace_start(&mut self, _namespace_start: XmlNamespaceStartWrapper<'a>) {}
    fn visit_xml_namespace_end(&mut self, _namespace_end: XmlNamespaceEndWrapper<'a>) {}
    fn visit_xml_tag_start(&mut self, _tag_start: XmlTagStartWrapper<'a>) {}
    fn visit_xml_tag_end(&mut self, _tag_end: XmlTagEndWrapper<'a>) {}
    fn visit_xml_text(&mut self, _text: XmlTextWrapper<'a>) {}
    fn visit_resource(&mut self, _resource: ResourceWrapper<'a>) {}
}

/// Methods to decode a binary resource.arsc file or a binary xml file
#[derive(Debug, Copy, Clone)]
pub struct Executor;

impl Executor {
    /// Given a valid `resources.arsc` file contents, it will call to the proper methods on the
    /// given visitor.
    pub fn arsc<'a, V: ChunkVisitor<'a>>(buffer: &'a [u8], visitor: &mut V) -> Result<(), Error> {
        let mut cursor = Cursor::new(buffer);
        let token = cursor
            .read_u16::<LittleEndian>()
            .context("error reading first token")?;

        if token != 0x2 {
            bail!("file does not start with ARSC token: {:X}", token);
        }

        let header_size = cursor
            .read_u16::<LittleEndian>()
            .context("error reading header size")?;
        let _chunk_size = cursor
            .read_u32::<LittleEndian>()
            .context("error reading chunk size")?;
        let _package_amount = cursor
            .read_u32::<LittleEndian>()
            .context("error reading package amount")?;
        // TODO: Avoid infinite loop
        cursor.set_position(u64::from(header_size));

        let stream = ChunkLoaderStream::new(cursor);
        let mut origin = Origin::Global;

        for c in stream {
            match c.context("error reading next chunk")? {
                Chunk::StringTable(stw) => {
                    visitor.visit_string_table(stw, origin);
                    origin = Origin::next(origin);
                }
                Chunk::Package(pw) => {
                    visitor.visit_package(pw);
                }
                Chunk::TableType(ttw) => {
                    visitor.visit_table_type(ttw);
                }
                Chunk::TableTypeSpec(tsw) => {
                    visitor.visit_type_spec(tsw);
                }
                _ => {
                    warn!("Not expected chunk on ARSC");
                }
            }
        }

        Ok(())
    }

    /// Given a valid binary XML file contents, it will call to the proper methods on the
    /// given visitor.
    pub fn xml<'a, V: ChunkVisitor<'a>>(
        mut cursor: Cursor<&'a [u8]>,
        visitor: &mut V,
    ) -> Result<(), Error> {
        let token = cursor
            .read_u16::<LittleEndian>()
            .context("error reading first token")?;

        if token != 0x3 {
            bail!("document does not start with XML token: {:X}", token);
        }

        let header_size = cursor
            .read_u16::<LittleEndian>()
            .context("error reading header size")?;
        let _chunk_size = cursor
            .read_u32::<LittleEndian>()
            .context("error reading chunk size")?;
        cursor.set_position(u64::from(header_size));
        let stream = ChunkLoaderStream::new(cursor);

        for c in stream {
            match c.context("error reading next chunk")? {
                Chunk::StringTable(stw) => {
                    visitor.visit_string_table(stw, Origin::Global);
                }
                Chunk::XmlNamespaceStart(xnsw) => {
                    visitor.visit_xml_namespace_start(xnsw);
                }
                Chunk::XmlNamespaceEnd(xnsw) => {
                    visitor.visit_xml_namespace_end(xnsw);
                }
                Chunk::XmlTagStart(xnsw) => {
                    visitor.visit_xml_tag_start(xnsw);
                }
                Chunk::XmlTagEnd(xnsw) => {
                    visitor.visit_xml_tag_end(xnsw);
                }
                Chunk::XmlText(xsnw) => {
                    visitor.visit_xml_text(xsnw);
                }
                Chunk::Resource(rw) => {
                    visitor.visit_resource(rw);
                }
                _ => (),
            }
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Origin {
    Global,
    Spec,
    Entries,
}

impl Origin {
    pub fn next(origin: Self) -> Self {
        match origin {
            Origin::Global => Origin::Spec,
            Origin::Spec => Origin::Entries,
            Origin::Entries => Origin::Global,
        }
    }
}
