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
use std;
use decoder::Decoder;
use zip;

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