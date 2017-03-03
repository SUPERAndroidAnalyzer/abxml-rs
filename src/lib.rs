#![recursion_limit = "1024"]

#![feature(repeat_str, test)]
extern crate byteorder;
extern crate test;
extern crate quick_xml;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate encoding;
extern crate zip;

pub mod encoder;
pub mod chunks;
pub mod visitor;
pub mod model;
pub mod apk;
pub mod decoder;

use visitor::ModelVisitor;
use std::path::Path;
use std::io::Cursor;
use visitor::Executor;
use errors::*;
use std::io::Read;
use std::io;
use std::fs;
use zip::ZipArchive;
use visitor::*;
use encoder::Xml;
use std::io::Write;

pub const STR_ARSC: &'static [u8] = include_bytes!("../resources/resources.arsc");

pub mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            Xml(::quick_xml::error::Error);
            Utf8(::std::string::FromUtf8Error);
            Zip(::zip::result::ZipError);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use apk::Apk;

    #[test]
    #[should_panic]
    fn it_can_generate_a_decoder_from_an_apk() {
        let path = Path::new("some.apk");
        let mut buffer = Vec::new();

        Apk::new(path, &mut buffer).unwrap();
    }
}
