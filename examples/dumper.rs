extern crate abxml;
#[macro_use]
extern crate error_chain;
extern crate ansi_term;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate byteorder;

use std::env;
use abxml::encoder::Xml;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
/*use abxml::chunks::Chunk;
use abxml::chunks::ChunkLoaderStream;
use abxml::chunks::StringTable;*/
use abxml::chunks::*;
use abxml::errors::*;
use abxml::visitor::*;

use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};

use ansi_term::Colour::{Red, Green, Blue, Yellow};
use ansi_term::Style;
use env_logger::LogBuilder;
use log::{LogRecord, LogLevelFilter, LogLevel};

fn main() {
    env_logger::init().unwrap();

    if let Err(ref e) = run() {
        let err_str = Red.bold().paint("Error: ").to_string();
        println!("{}{}", err_str, Red.paint(e.description()));

        for e in e.iter().skip(1) {
            let err_str = Green.bold().paint("Caused by: ").to_string();
            println!("\t{}{}", err_str, Green.paint(e.description()));
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            let str_backtrace = format!("{:?}", backtrace);

            let err_str = Blue.bold().paint("Backtrace: ").to_string();
            println!("\t{}{}", err_str, str_backtrace);
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
    let executor = Executor::arsc(cursor, &mut visitor);
    let entries = visitor.get_entries();

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
