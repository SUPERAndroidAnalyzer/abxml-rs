use errors::*;
use byteorder::{LittleEndian, WriteBytesExt};

pub use self::resources::ResourcesBuf;
pub use self::string_table::StringTableBuf;
pub use self::string_table::Encoding;
pub use self::xml::XmlTagStartBuf;
pub use self::xml::XmlTagEndBuf;
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
    fn get_header_size(&self) -> u16;

    fn to_vec(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        let body = self.get_body_data().chain_err(|| "Could not read chunk body")?;

        self.write_header(&mut out, &body).chain_err(|| "Could not write header")?;

        out.extend(body.iter());

        Ok(out)
    }

    fn write_header(&self, buffer: &mut Vec<u8>, body: &[u8]) -> Result<()> {
        let header_size = self.get_header_size();
        buffer.write_u16::<LittleEndian>(self.get_token())?;
        buffer.write_u16::<LittleEndian>(header_size)?;
        buffer.write_u32::<LittleEndian>(body.len() as u32 + 8)?;

        Ok(())
    }
}
