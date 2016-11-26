extern crate abxml;

use std::env;
use abxml::encoder::Xml;
use abxml::BinaryXmlDecoder;
use std::path::Path;
use std::fs::File;
use std::error::Error;
use std::io::prelude::*;


fn main() {
    let path = match env::args().nth(1) {
        Some(path) => path,
        None => {
            println!("Usage: converter <path>");
            return;
        }
    };

    let content = file_get_contents(&path);
    let parser = BinaryXmlDecoder::new(&content);
    let result = parser.decode().unwrap();
    let xml_content = Xml::encode(&result.resources, &result.root_element).unwrap();
    println!("{}", xml_content);
}

fn file_get_contents(path: &str) -> Vec<u8> {
    let path = Path::new(path);
    let display = path.display();

    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut v: Vec<u8> = Vec::new();
    match file.read_to_end(&mut v) {
        Err(why) => panic!("couldn't read {}: {}", display, why.description()),
        Ok(_) => (),
    };

    return v;
}
