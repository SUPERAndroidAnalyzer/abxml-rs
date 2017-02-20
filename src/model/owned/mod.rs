use errors::*;

mod resources;
mod string_table;

pub trait OwnedBuf {
    fn to_vec(&self) -> Result<Vec<u8>>;
}

pub use self::resources::ResourcesBuf as ResourceBuf;
pub use self::string_table::StringTableBuf as StringTableBuf;
pub use self::string_table::StringMode as StringMode;