pub use self::{
    package::PackageBuf,
    resources::ResourcesBuf,
    string_table::{Encoding, StringTableBuf},
    table_type::{ComplexEntry, ConfigurationBuf, Entry, EntryHeader, SimpleEntry, TableTypeBuf},
    table_type_spec::TableTypeSpecBuf,
    xml::{AttributeBuf, XmlNamespaceEndBuf, XmlNamespaceStartBuf, XmlTagEndBuf, XmlTagStartBuf},
};
use anyhow::{Context, Result};
use byteorder::{LittleEndian, WriteBytesExt};
use std::fmt::Debug;

mod package;
mod resources;
mod string_table;
mod table_type;
mod table_type_spec;
mod xml;

/// Implementors are able to be converted to well formed chunks as expected on `ChunkLoaderStream`
pub trait OwnedBuf: Debug {
    /// Token that identifies the current chunk
    fn token(&self) -> u16;
    /// Return the bytes corresponding to chunk's body
    fn body_data(&self) -> Result<Vec<u8>>;

    /// Return the bytes corresponding to chunk's header
    fn header(&self) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    /// Convert the given `OwnedBuf` to a well formed chunk in form of vector of bytes
    fn to_vec(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        let body = self.body_data().context("could not read chunk body")?;

        self.write_header(&mut out, &body)
            .context("could not write header")?;

        out.extend(body.iter());

        Ok(out)
    }

    /// Writes the header to the output buffer. It writes token, header size and chunk_size and
    /// then the data returned by `get_header`.
    fn write_header(&self, buffer: &mut Vec<u8>, body: &[u8]) -> Result<()> {
        let header = self.header()?;
        let header_size = header.len() as u16 + 8;

        buffer.write_u16::<LittleEndian>(self.token())?;
        buffer.write_u16::<LittleEndian>(header_size)?;
        buffer.write_u32::<LittleEndian>(body.len() as u32 + u32::from(header_size))?;
        buffer.extend(&header);

        Ok(())
    }
}
