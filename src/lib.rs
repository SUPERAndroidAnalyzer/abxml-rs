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
    use decoder::{Apk, Decoder};
    use model::builder::Xml;
    use model::owned::{XmlTagStartBuf, XmlTagEndBuf, StringTableBuf};

    #[test]
    #[should_panic]
    fn it_can_generate_a_decoder_from_an_apk() {
        let path = Path::new("some.apk");
        let mut buffer = Vec::new();

        Apk::new(path, &mut buffer).unwrap();
    }

    #[test]
    fn it_can_generate_a_decoder_from_a_buffer() {
        let arsc = vec![0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut xml = Xml::new();
        let mut st = StringTableBuf::new();
        st.add_string("Some string".to_string());
        st.add_string("Another srtring".to_string());
        st.add_string("start_tag".to_string());

        xml.push_owned(Box::new(st));
        xml.push_owned(Box::new(XmlTagStartBuf::new(2, None)));
        xml.push_owned(Box::new(XmlTagEndBuf::new()));

        let xml_content = xml.to_vec().unwrap();

        let decoder = Decoder::new(&arsc).unwrap();
        let out = decoder.as_xml(&xml_content).unwrap();

        let inner_expected = "<start_tag></start_tag>";
        let expected = format!("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"no\"?>\n{}", inner_expected);

        assert_eq!(expected, out);
    }
}
