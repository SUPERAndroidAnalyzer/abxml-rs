extern crate abxml;
extern crate byteorder;
extern crate env_logger;
extern crate failure;
extern crate log;
extern crate zip;

use std::env;
use std::path::Path;

use failure::{Error, ResultExt};

use abxml::apk::Apk;

fn main() {
    env_logger::try_init().unwrap();

    if let Err(ref e) = run() {
        println!("error: {}", e);

        for e in e.iter_causes() {
            println!("caused by: {}", e);
        }

        ::std::process::exit(1);
    }
}

// Most functions will return the `Result` type, imported from the
// `errors` module. It is a typedef of the standard `Result` type
// for which the error type is always our own `Error`.
fn run() -> Result<(), Error> {
    let apk_path = match env::args().nth(1) {
        Some(path) => path,
        None => {
            println!("Usage: exporter <apk> <path>");
            return Ok(());
        }
    };

    let output = match env::args().nth(2) {
        Some(path) => path,
        None => {
            println!("Usage: exporter <apk> <path>");
            return Ok(());
        }
    };

    let path = Path::new(&apk_path);
    let mut apk = Apk::new(path).context("error loading APK")?;
    apk.export(Path::new(&output), true)
        .context("APK could not be exported")?;

    Ok(())
}
