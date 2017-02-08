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
            return Ok(())
        }
    };

    let android_apk_path = match env::args().nth(2) {
        Some(path) => path,
        None => {
            println!("Usage: converter <path>");
            return Ok(())
        }
    };

    // Android lib
    let file = std::fs::File::open(&android_apk_path)?;
    let mut android_archive = zip::ZipArchive::new(file).unwrap();

    let mut android_resources_content = Vec::new();
    android_archive.by_name("resources.arsc").unwrap().read_to_end(&mut android_resources_content)?;

    // APK
    let file = std::fs::File::open(&apk_path)?;
    let mut archive = zip::ZipArchive::new(file).unwrap();

    let mut resources_content = Vec::new();
    archive.by_name("resources.arsc").unwrap().read_to_end(&mut resources_content)?;

    let mut manifest_content = Vec::new();
    archive.by_name("AndroidManifest.xml").unwrap().read_to_end(&mut manifest_content)?;

    let resources_cursor: Cursor<&[u8]> = Cursor::new(&resources_content);
    let android_resources_cursor: Cursor<&[u8]> = Cursor::new(&android_resources_content);
    let mut resources_visitor = ModelVisitor::default();
    Executor::arsc(resources_cursor, &mut resources_visitor)?;
    Executor::arsc(android_resources_cursor, &mut resources_visitor)?;


    let resources = resources_visitor.get_mut_resources();
    // let manifest = parse_xml(manifest_content, resources)?;
    // println!("{}", manifest);

    for i in 0..archive.len() {
        let mut current_file = archive.by_index(i).unwrap();

        if current_file.name().contains("res/layout/detail_item_attachment.xml") {
            let mut xml_content = Vec::new();
            current_file.read_to_end(&mut xml_content)?;

            let out = parse_xml(xml_content, resources)?;
            println!("{}", out);
        }
    }

    Ok(())
}

fn parse_xml(content: Vec<u8>, resources: &mut Resources) -> Result<String> {
    let cursor: Cursor<&[u8]> = Cursor::new(&content);
    let mut visitor = XmlVisitor::default();

    Executor::xml(cursor, &mut visitor, resources)?;

    match *visitor.get_root() {
        Some(ref root) => {
            match *visitor.get_string_table() {
                Some(_) => {
                    return Xml::encode(
                        visitor.get_namespaces(),
                        root,
                        resources,
                    ).chain_err(|| "Could note encode XML");
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

    Err("Could not decode XML".into())
}
