extern crate abxml;
extern crate byteorder;
extern crate env_logger;
extern crate failure;
extern crate log;
extern crate zip;

use std::{
    env,
    io::{prelude::*, Cursor},
};

use failure::{bail, Error, ResultExt};

use abxml::{encoder::Xml, visitor::*};

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

fn run() -> Result<(), Error> {
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
    archive
        .by_name("resources.arsc")
        .unwrap()
        .read_to_end(&mut resources_content)?;

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
                let out =
                    parse_xml(&new_content, resources).context("could not decode target file")?;
                println!("{}", out);
            }
        }
    }

    Ok(())
}

fn parse_xml<'a>(content: &[u8], resources: &'a Resources<'a>) -> Result<String, Error> {
    let cursor = Cursor::new(content);
    let mut visitor = XmlVisitor::new(resources);

    Executor::xml(cursor, &mut visitor)?;

    match *visitor.get_root() {
        Some(ref root) => match *visitor.get_string_table() {
            Some(_) => {
                let res =
                    Xml::encode(visitor.get_namespaces(), root).context("could note encode XML")?;
                return Ok(res);
            }
            None => {
                println!("No string table found");
            }
        },
        None => {
            println!("No root on target XML");
        }
    }

    bail!("could not decode XML")
}
