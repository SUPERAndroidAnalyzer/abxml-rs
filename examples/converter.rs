extern crate abxml;
#[macro_use]
extern crate error_chain;
extern crate byteorder;

use std::env;
use abxml::encoder::Xml;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use abxml::errors::*;
use abxml::chunks::*;
use abxml::visitor::*;

use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};

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
    let mut cursor: Cursor<&[u8]> = Cursor::new(&content);

    let mut visitor = ModelVisitor::new();
    let executor = Executor::xml(cursor, &mut visitor);

    match visitor.get_root() {
        &Some(ref root) => {
            let xml_content = Xml::encode(&visitor.get_namespaces(), &root).chain_err(|| "Could note encode XML")?;
            println!("{}", xml_content);
        },
        &None => {
            println!("No root on target XML");
        }
    }


/*
        match parser.get_element_container().get_root() {
-        &Some(ref root) => {
-            let xml_content = Xml::encode(&parser.get_namespaces(), &root).chain_err(|| "Could not decode XML")?;
-            println!("{}", xml_content);
-        },
-        _ => (),
-    }
*/

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
