#![recursion_limit = "1024"]

extern crate byteorder;
extern crate quick_xml;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate encoding;
#[cfg(feature = "zip_decode")]
extern crate zip;

pub mod encoder;
pub mod chunks;
pub mod visitor;
pub mod model;
pub mod decoder;
#[cfg(test)]
pub mod test;
#[cfg(test)]
pub mod raw_chunks;
#[cfg(feature = "zip_decode")]
pub mod apk;

pub const STR_ARSC: &'static [u8] = include_bytes!("../resources/resources.arsc");

pub mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            Xml(::quick_xml::error::Error);
            Utf8(::std::string::FromUtf8Error);
            Zip(::zip::result::ZipError) #[cfg(feature = "zip_decode")];
        }
    }
}
