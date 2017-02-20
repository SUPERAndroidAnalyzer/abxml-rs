#![recursion_limit = "1024"]

#![feature(repeat_str, test)]
extern crate byteorder;
extern crate test;
extern crate quick_xml;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate encode_unicode;
extern crate encoding;

pub mod encoder;
pub mod chunks;
pub mod visitor;
pub mod model;

pub mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {
        foreign_links {
            Io(::std::io::Error) #[cfg(unix)];
            Xml(::quick_xml::error::Error);
            Utf8(::std::string::FromUtf8Error);
            Utf16(::encode_unicode::error::InvalidUtf16Slice);
        }
    }
}

#[cfg(test)]
mod tests {

}
