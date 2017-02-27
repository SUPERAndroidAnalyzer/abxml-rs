extern crate abxml;
extern crate error_chain;
extern crate byteorder;
extern crate zip;
extern crate log;
extern crate env_logger;

use std::env;
use abxml::encoder::Xml;
use std::io::prelude::*;
use abxml::errors::*;
use abxml::visitor::*;
use std::io::Cursor;
use abxml::Apk;
use std::path::Path;

fn main() {
    env_logger::init().unwrap();

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
    let apk_path = match env::args().nth(1) {
        Some(path) => path,
        None => {
            println!("Usage: exporter <apk> <path>");
            return Ok(())
        }
    };

    let output = match env::args().nth(2) {
        Some(path) => path,
        None => {
            println!("Usage: exporter <apk> <path>");
            return Ok(())
        }
    };

    let path = Path::new(&apk_path);
    let mut buffer = Vec::new();

    let mut apk = Apk::new(path, &mut buffer).chain_err(|| "Error loading APK")?;
    apk.export(Path::new(&output), true).chain_err(|| "APK could not be exported")?;

    Ok(())
}