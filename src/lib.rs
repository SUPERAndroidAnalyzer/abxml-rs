#![recursion_limit = "1024"]

#![feature(repeat_str, test)]
extern crate byteorder;
extern crate test;
extern crate quick_xml;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate encoding;
extern crate zip;

pub mod encoder;
pub mod chunks;
pub mod visitor;
pub mod model;

use visitor::ModelVisitor;
use std::path::Path;
use std::io::Cursor;
use visitor::Executor;
use errors::*;
use std::io::Read;
use std::io;
use std::fs;
use zip::ZipArchive;
use visitor::*;
use encoder::Xml;
use std::io::Write;

#[cfg(feature = "embed_default_arsc")]
pub const STR_ARSC: &'static [u8] = include_bytes!("../resources/resources.arsc");

pub mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            Xml(::quick_xml::error::Error);
            Utf8(::std::string::FromUtf8Error);
            Zip(::zip::result::ZipError);
        }
    }
}

pub struct Decoder<'a> {
    visitor: ModelVisitor<'a>,
    buffer_android: &'a [u8],
    buffer_apk: &'a [u8],
}

impl<'a> Decoder<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self> {
        let visitor = ModelVisitor::default();

        let mut decoder = Decoder {
            visitor: visitor,
            buffer_android: STR_ARSC,
            buffer_apk: data,
        };

        let android_resources_cursor: Cursor<&[u8]> = Cursor::new(decoder.buffer_android);
        Executor::arsc(android_resources_cursor, &mut decoder.visitor).chain_err(|| "Could not read android lib resources")?;

        let cursor: Cursor<&[u8]> = Cursor::new(decoder.buffer_apk);
        Executor::arsc(cursor, &mut decoder.visitor).chain_err(|| "Could not read target APK resources")?;

        Ok(decoder)
    }

    pub fn get_resources(&self) -> &'a Resources {
        &self.visitor.get_resources()
    }
}

pub struct Apk<'a> {
    handler: ZipArchive<std::fs::File>,
    decoder: Decoder<'a>,
}

impl<'a> Apk<'a> {
    pub fn new(path: &Path, mut buffer: &'a mut Vec<u8>) -> Result<Self> {
        let file = std::fs::File::open(&path)?;
        let mut zip_handler = zip::ZipArchive::new(file)?;

        zip_handler.by_name("resources.arsc")?
            .read_to_end(&mut buffer)?;

        let decoder = Decoder::new(buffer.as_slice())?;

        let apk = Apk {
            handler: zip_handler,
            decoder: decoder,
        };

        Ok(apk)
    }

    /// It exports to target output_path the contents of the APK, transcoding the binary XML files found on it.
    pub fn export(&mut self, output_path: &Path, force: bool) -> Result<()> {
        match fs::create_dir(output_path) {
            Err(_) => {
                if force {
                    fs::remove_dir_all(output_path).chain_err(|| "Could not clean target directory")?;
                    fs::create_dir(output_path).chain_err(|| "Error creating the output folder")?;
                }
            }
            _ => (),
        }

        // Iterate over all the files on the ZIP and extract them
        for i in 0..self.handler.len() {
            let (file_name, contents) = {
                let mut current_file = self.handler.by_index(i).chain_err(|| "Could not read ZIP entry")?;
                let mut contents = Vec::new();
                current_file.read_to_end(&mut contents).chain_err(|| format!("Could not read: {}", current_file.name()))?;
                let is_xml = current_file.name().to_string();

                (is_xml, contents)
            };

            let contents = if (file_name.starts_with("res/") && file_name.ends_with(".xml")) || file_name == "AndroidManifest.xml" {
                let new_content = contents.clone();
                let resources = self.decoder.get_resources();
                let out = self.parse_xml(&new_content).chain_err(|| format!("Could not decode: {}", file_name))?;

                out.into_bytes()
            } else {
                contents
            };

            Self::write_file(output_path, &file_name, &contents).chain_err(|| "Could not write output file")?;

        }
        Ok(())
    }

    fn write_file(base_path: &Path, relative: &String, content: &Vec<u8>) -> Result<()> {
        let full_path = base_path.join(Path::new(relative));
        //println!("Full path: {:?}", full_path);
        fs::create_dir_all(full_path.parent().unwrap()).chain_err(|| "Could not create the output dir")?;

        let mut descriptor = fs::OpenOptions::new().write(true)
                            .create_new(true)
                            .open(full_path)
                            .chain_err(|| "Could not open file to write")?;

        descriptor.write_all(&content).chain_err(|| "Could not write to target file")?;
        descriptor.sync_all().chain_err(|| "Could not flush")?;

        Ok(())
    }

    fn parse_xml(&self, content: &Vec<u8>) -> Result<String> {
        let cursor: Cursor<&[u8]> = Cursor::new(&content);
        let mut visitor = XmlVisitor::default();

        Executor::xml(cursor, &mut visitor)?;

        match *visitor.get_root() {
            Some(ref root) => {
                match *visitor.get_string_table() {
                    Some(_) => {
                        return Xml::encode(
                            visitor.get_namespaces(),
                            root,
                            visitor.get_resources(),
                            self.decoder.get_resources(),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn it_can_generate_a_decoder_from_an_apk() {
        let path = Path::new("some.apk");
        let mut buffer = Vec::new();

        Apk::new(path, &mut buffer).unwrap();
    }
}
