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
            println!("Usage: converter <path>");
            return Ok(());
        }
    };

    let target_file = match env::args().nth(2) {
        Some(path) => path,
        None => {
            println!("Usage: converter <path>");
            return Ok(());
        }
    };

    // Android lib
    let android_resources_content = abxml::STR_ARSC.to_owned();

    // APK
    let file = std::fs::File::open(&apk_path)?;
    let mut archive = zip::ZipArchive::new(file).unwrap();

    let mut resources_content = Vec::new();
    archive.by_name("resources.arsc").unwrap().read_to_end(&mut resources_content)?;

    let mut resources_visitor = ModelVisitor::default();
    Executor::arsc(&resources_content, &mut resources_visitor)?;
    Executor::arsc(&android_resources_content, &mut resources_visitor)?;

    for i in 0..archive.len() {
        let mut current_file = archive.by_index(i).unwrap();

        if current_file.name().contains(&target_file) {
            {
                // println!("Current file: {}", current_file.name());
                let mut xml_content = Vec::new();
                current_file.read_to_end(&mut xml_content)?;
                let new_content = xml_content.clone();

                let resources = resources_visitor.get_resources();
                let out = parse_xml(&new_content, resources).unwrap();
                println!("{}", out);
            }
        }
    }

    Ok(())
}

fn parse_xml<'a>(content: &[u8], resources: &'a Resources<'a>) -> Result<String> {
    let cursor = Cursor::new(content);
    let mut visitor = XmlVisitor::new(resources);

    Executor::xml(cursor, &mut visitor)?;

    match *visitor.get_root() {
        Some(ref root) => {
            match *visitor.get_string_table() {
                Some(_) => {
                    return Xml::encode(visitor.get_namespaces(),
                                       root,
                                       visitor.get_resources(),
                                       resources)
                        .chain_err(|| "Could note encode XML");
                }
                None => {
                    println!("No string table found");
                }
            }
        }
        None => {
            println!("No root on target XML");
        }
    }

    Err("Could not decode XML".into())
}
