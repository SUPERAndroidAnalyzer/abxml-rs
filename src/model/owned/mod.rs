use errors::*;
use byteorder::{LittleEndian, WriteBytesExt};

mod resources;
mod string_table;

pub trait OwnedBuf {
    fn get_token(&self) -> u16;
    fn get_body_data(&self) -> Result<Vec<u8>>;
    fn get_header_size(&self) -> u16;

    fn to_vec(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        let body = self.get_body_data()?;

        self.write_header(&mut out, &body)?;

        out.extend(body.iter());

        Ok(out)
    }

    fn write_header(&self, buffer: &mut Vec<u8>, body: &Vec<u8>) -> Result<()> {
        let header_size = self.get_header_size();
        buffer.write_u16::<LittleEndian>(self.get_token())?;
        buffer.write_u16::<LittleEndian>(header_size)?;
        buffer.write_u32::<LittleEndian>(body.len() as u32 + 8);

        Ok(())
    }
}

pub use self::resources::ResourcesBuf as ResourceBuf;
pub use self::string_table::StringTableBuf as StringTableBuf;
pub use self::string_table::Encoding as Encoding;