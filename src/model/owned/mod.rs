use errors::*;
use byteorder::{LittleEndian, WriteBytesExt};

pub use self::resources::ResourcesBuf;
pub use self::string_table::StringTableBuf;
pub use self::string_table::Encoding;
pub use self::xml::XmlTagStartBuf;
pub use self::xml::XmlTagEndBuf;
pub use self::xml::XmlNamespaceStartBuf;
pub use self::xml::XmlNamespaceEndBuf;
pub use self::package::PackageBuf;
pub use self::table_type_spec::TableTypeSpecBuf;
pub use self::table_type::ConfigurationBuf;
pub use self::table_type::TableTypeBuf;
pub use self::table_type::{Entry, EntryHeader, ComplexEntry, SimpleEntry};

mod resources;
mod string_table;
mod xml;
mod package;
mod table_type_spec;
mod table_type;

pub trait OwnedBuf {
    fn get_token(&self) -> u16;
    fn get_body_data(&self) -> Result<Vec<u8>>;

    fn get_header(&self) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    fn to_vec(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        let body = self.get_body_data().chain_err(|| "Could not read chunk body")?;

        self.write_header(&mut out, &body).chain_err(|| "Could not write header")?;

        out.extend(body.iter());

        Ok(out)
    }

    fn write_header(&self, buffer: &mut Vec<u8>, body: &[u8]) -> Result<()> {
        let header = self.get_header()?;
        let header_size = header.len() as u16 + 8;

        buffer.write_u16::<LittleEndian>(self.get_token())?;
        buffer.write_u16::<LittleEndian>(header_size)?;
        buffer.write_u32::<LittleEndian>(body.len() as u32 + header_size as u32)?;
        buffer.extend(&header);

        Ok(())
    }
}
