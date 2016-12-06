extern crate abxml;

use std::env;
use abxml::encoder::Xml;
use abxml::BinaryXmlDecoder;
use std::path::Path;
use std::fs::File;
use std::error::Error;
use std::io::prelude::*;
use abxml::parser::ArscDecoder;
use abxml::chunks::Chunk;

fn main() {
    let path = match env::args().nth(1) {
        Some(path) => path,
        None => {
            println!("Usage: converter <path>");
            return;
        }
    };

    let content = file_get_contents(&path);
    let decoder = ArscDecoder;
    let chunks = decoder.decode(&content).unwrap();

    for c in chunks {
        match c {
            Chunk::Unknown => {
                println!("Unknown chunk!");
            },
            Chunk::StringTable(st) => {
                println!("Strint table chunk");
                println!("Strings size {}", st.strings.len());
                println!("Styles size {}", st.styles.len());
            },
            Chunk::Package => {
                println!("Package chunk");
            },
            Chunk::TableType(rc) => {
                // println!("Resource config chunk");
                // println!("Resc config {:?}", rc);
            }
        }
    }
    // println!("{:?}", result);
    // Add the new decoder
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
