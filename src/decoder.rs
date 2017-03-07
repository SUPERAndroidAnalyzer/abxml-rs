use visitor::ModelVisitor;
use std::path::Path;
use std::io::Cursor;
use visitor::Executor;
use errors::*;
use std::io::Read;
use std::fs;
use zip::ZipArchive;
use visitor::*;
use encoder::Xml;
use std::io::Write;
use STR_ARSC;
use std;
use zip;
use std::borrow::Cow;
use std::borrow::Borrow;

pub struct Decoder<'a> {
    visitor: ModelVisitor<'a>,
    buffer_android: &'a [u8],
    buffer_apk: Cow<'a, [u8]>,
}

impl<'a> Decoder<'a> {
    pub fn new(data: Cow<'a, [u8]>) -> Result<Decoder<'a>> {
        let visitor = ModelVisitor::default();

        let mut decoder = Decoder {
            visitor: visitor,
            buffer_android: STR_ARSC,
            buffer_apk: data,
        };
/*
        let android_resources_cursor = Cursor::new(decoder.buffer_android);
        Executor::arsc(android_resources_cursor, &mut decoder.visitor).chain_err(|| "Could not read android lib resources")?;

        let cursor = Cursor::new(decoder.get_buffer());
        Executor::arsc(cursor, &mut decoder.visitor).chain_err(|| "Could not read target APK resources")?;
*/
        Ok(decoder)
    }

    pub fn get_resources(&self) -> &'a Resources {
        self.visitor.get_resources()
    }

    pub fn as_xml(&self, content: &[u8]) -> Result<String> {
        let cursor = Cursor::new(content);
        let mut visitor = XmlVisitor::default();

        Executor::xml(cursor, &mut visitor)?;

        match *visitor.get_root() {
            Some(ref root) => {
                match *visitor.get_string_table() {
                    Some(_) => {
                        return Xml::encode(visitor.get_namespaces(),
                                           root,
                                           visitor.get_resources(),
                                           self.get_resources())
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
}

pub struct Apk<'a> {
    handler: ZipArchive<std::fs::File>,
    decoder: Decoder<'a>,
    cow: Cow<'a, [u8]>
}

impl<'a> Apk<'a> {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut buffer = Vec::new();
        let file = std::fs::File::open(&path)?;
        let mut zip_handler = zip::ZipArchive::new(file)?;
        zip_handler.by_name("resources.arsc")?.read_to_end(&mut buffer)?;
        let cow = Cow::from(buffer);
        let decoder = Decoder::new(cow.clone())?;

        let apk = Apk {
            handler: zip_handler,
            decoder: decoder,
            cow: cow,
        };

        Ok(apk)
    }

    /// It exports to target output_path the contents of the APK, transcoding the binary XML files
    /// found on it.
    pub fn export<P: AsRef<Path>>(&mut self, output_path: P, force: bool) -> Result<()> {
        if fs::create_dir(&output_path).is_err() && force {
            fs::remove_dir_all(&output_path).chain_err(|| "Could not clean target directory")?;
            fs::create_dir(&output_path).chain_err(|| "Error creating the output folder")?;
        }

        // Iterate over all the files on the ZIP and extract them
        for i in 0..self.handler.len() {
            let (file_name, contents) = {
                let mut current_file =
                    self.handler.by_index(i).chain_err(|| "Could not read ZIP entry")?;
                let mut contents = Vec::new();
                current_file.read_to_end(&mut contents)
                    .chain_err(|| format!("Could not read: {}", current_file.name()))?;
                let is_xml = current_file.name().to_string();

                (is_xml, contents)
            };

            let contents = if (file_name.starts_with("res/") && file_name.ends_with(".xml")) ||
                              file_name == "AndroidManifest.xml" {
                let new_content = contents.clone();
                let out = self.decoder
                    .as_xml(&new_content)
                    .chain_err(|| format!("Could not decode: {}", file_name))?;

                out.into_bytes()
            } else {
                contents
            };

            Self::write_file(&output_path, &file_name, &contents).chain_err(|| "Could not write output file")?;

        }
        Ok(())
    }

    fn write_file<B: AsRef<Path>, R: AsRef<Path>>(base_path: B,
                                                  relative: R,
                                                  content: &[u8])
                                                  -> Result<()> {
        let full_path = base_path.as_ref().join(&relative);
        // println!("Full path: {}", full_path.display());
        fs::create_dir_all(full_path.parent().unwrap()).chain_err(|| "Could not create the output dir")?;

        let mut descriptor = fs::OpenOptions::new().write(true)
            .create_new(true)
            .open(full_path)
            .chain_err(|| "Could not open file to write")?;

        descriptor.write_all(content).chain_err(|| "Could not write to target file")?;
        descriptor.sync_all().chain_err(|| "Could not flush")?;

        Ok(())
    }
}
