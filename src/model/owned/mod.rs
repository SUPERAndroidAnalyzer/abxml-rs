use errors::*;

mod resources;

pub trait OwnedBuf {
    fn to_vec(&self) -> Result<Vec<u8>>;
}

pub use self::resources::ResourcesBuf;