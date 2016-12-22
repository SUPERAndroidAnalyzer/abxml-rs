#![recursion_limit = "1024"]

#![feature(repeat_str, test)]
extern crate byteorder;
extern crate test;
extern crate quick_xml;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

mod document;
pub mod encoder;
pub mod parser;
pub mod chunks;

pub mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {
        foreign_links {
            Io(::std::io::Error) #[cfg(unix)];
        }
    }
}

use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use document::*;
use std::rc::Rc;
use errors::*;
use log::{LogRecord, LogLevelFilter, LogLevel};

#[cfg(test)]
mod tests {
/*    use std::error::Error;
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::Path;
    use super::*;

    use test::Bencher;

    #[test]
    fn it_works() {
        let original_file = file_get_contents("tests/binary_manifests/AndroidManifest-ce.xml");
        let parser = BinaryXmlDecoder::new(&original_file);
        let result = parser.decode();
        println!("{:?}", result);
    }

    #[bench]
    fn bench_manifest_parsing(b: &mut Bencher) {
        let original_file = file_get_contents("tests/binary_manifests/AndroidManifest-ce.xml");

        b.iter(move || {
            let parser = BinaryXmlDecoder::new(&original_file);
            parser.decode().unwrap();
        });
    }*/
}
