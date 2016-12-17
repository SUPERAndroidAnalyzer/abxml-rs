extern crate abxml;
#[macro_use]
extern crate error_chain;

use std::env;
use abxml::encoder::Xml;
use abxml::BinaryXmlDecoder;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use abxml::parser::ArscDecoder;
use abxml::chunks::Chunk;
use abxml::errors::*;

fn main() {
    if let Err(ref e) = run() {
        println!("error: {}", e);

        for e in e.iter().skip(1) {
            println!("caused by: {}", e);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }

        ::std::process::exit(1);
    }
}

// Most functions will return the `Result` type, imported from the
// `errors` module. It is a typedef of the standard `Result` type
// for which the error type is always our own `Error`.
fn run() -> Result<()> {
    let path = match env::args().nth(1) {
        Some(path) => path,
        None => {
            println!("Usage: converter <path>");
            return Ok(())
        }
    };

    let content = file_get_contents(&path);
    let decoder = ArscDecoder;
    let chunks = decoder.decode(&content)?;

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
            Chunk::TableType(id, rc, entries) => {
                // println!("Resource config chunk");
                // println!("Resc config {:?}", rc);
            },
            _ => {
                println!("Unknouwn Chunk!");
            }
        }
    }
    Ok(())
}

fn file_get_contents(path: &str) -> Vec<u8> {
    let path = Path::new(path);
    let display = path.display();

    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("Could ont open file"),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut v: Vec<u8> = Vec::new();
    match file.read_to_end(&mut v) {
        Err(why) => panic!("Could not read"),
        Ok(_) => (),
    };

    return v;
}
