extern crate abxml;
extern crate error_chain;
extern crate byteorder;

use std::env;
use abxml::encoder::Xml;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use abxml::errors::*;
use abxml::visitor::*;
use std::io::Cursor;

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

    let xml_path = match env::args().nth(2) {
        Some(path) => path,
        None => {
            println!("Usage: converter <path>");
            return Ok(())
        }
    };

    let resource_content = file_get_contents(&path);
    let resources_cursor: Cursor<&[u8]> = Cursor::new(&resource_content);
    let mut resources_visitor = ModelVisitor::new();
    Executor::arsc(resources_cursor, &mut resources_visitor)?;

    let content = file_get_contents(&xml_path);
    let cursor: Cursor<&[u8]> = Cursor::new(&content);

    let mut visitor = XmlVisitor::new();
    let resources = resources_visitor.get_mut_resources();

    Executor::xml(cursor, &mut visitor, resources)?;

    match *visitor.get_root() {
        Some(ref root) => {
            match *visitor.get_string_table() {
                Some(_) => {
                    let xml_content = Xml::encode(
                        visitor.get_namespaces(),
                        root,
                        resources,
                    ).chain_err(|| "Could note encode XML")?;
                    println!("{}", xml_content);
                },
                None => {
                    println!("No string table found");
                }
            }
        },
        None => {
            println!("No root on target XML");
        }
    }

    Ok(())
}

fn file_get_contents(path: &str) -> Vec<u8> {
    let path = Path::new(path);

    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(_) => panic!("Could ont open file"),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut v: Vec<u8> = Vec::new();
    if file.read_to_end(&mut v).is_err() {
        panic!("Could not read");
    }

    v
}
