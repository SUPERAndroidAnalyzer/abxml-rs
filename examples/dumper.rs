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

    let visitor = PrintVisitor;
    let executor = Executor::arsc(cursor, visitor);


    // resources.arsc head. Move somewhere else
    /*let token = cursor.read_u16::<LittleEndian>()?;
    let header_size = cursor.read_u16::<LittleEndian>()?;
    let chunk_size = cursor.read_u32::<LittleEndian>()?;
    let package_amount = cursor.read_u32::<LittleEndian>()?;

    let stream = ChunkLoaderStream::new(cursor);

    for c in stream {
        match c {
            Chunk::Unknown => {
                println!("Unknown chunk!");
            },
            Chunk::StringTable(stw) => {
                let mut st = StringTable::new(stw);

                println!("Strint table chunk");
                println!("Strings size {}", st.get_strings_len());
                println!("Styles size {}", st.get_styles_len());
                println!("First string: {}", st.get_string(0).unwrap());
            },
            Chunk::Package(pw) => {
                let package = Package::new(pw);

                println!("Package chunk");
                println!("\tId: {}", package.get_id());
                println!("\tName: {}", package.get_name().unwrap());
            },
            Chunk::TableTypeSpec(tsw) => {
                let type_spec = TypeSpec::new(tsw);

                println!("TableTypeSpec chunk");
                println!("\tId: {}", type_spec.get_id());
            },
            Chunk::TableType(ttw) => {
                let table_type = TableType::new(ttw);

                println!("TableType chunk");
                println!("\tId: {}", table_type.get_id());
            },
            //Chunk::TableType(id, rc, entries) => {
                // println!("Resource config chunk");
                // println!("Resc config {:?}", rc);
            //},
            _ => {
                println!("Unknouwn Chunk!");
            }
        }
    }*/
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
