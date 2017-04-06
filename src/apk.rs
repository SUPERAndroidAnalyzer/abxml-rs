//! High level abstraction to easy the extraction to file system of APKs

use zip::read::ZipArchive;
use std::fs;
use std;
use std::path::Path;
use std::io::Write;
use std::io::Read;
use errors::*;
use decoder::*;

pub struct Apk {
    handler: ZipArchive<std::fs::File>,
    decoder: BufferedDecoder,
}

impl Apk {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut buffer = Vec::new();
        let file = std::fs::File::open(&path)?;
        let mut zip_handler = ZipArchive::new(file)?;
        zip_handler
            .by_name("resources.arsc")?
            .read_to_end(&mut buffer)?;

        let apk = Apk {
            handler: zip_handler,
            decoder: buffer.into(),
        };

        Ok(apk)
    }

    /// It exports to target output_path the contents of the APK, transcoding the binary XML files
    /// found on it.
    pub fn export<P: AsRef<Path>>(&mut self, output_path: P, force: bool) -> Result<()> {
        let decoder = self.decoder
            .get_decoder()
            .chain_err(|| "Could not get the decoder")?;

        if fs::create_dir(&output_path).is_err() && force {
            fs::remove_dir_all(&output_path)
                .chain_err(|| "Could not clean target directory")?;
            fs::create_dir(&output_path)
                .chain_err(|| "Error creating the output folder")?;
        }

        // Iterate over all the files on the ZIP and extract them
        for i in 0..self.handler.len() {
            let (file_name, contents) = {
                let mut current_file = self.handler
                    .by_index(i)
                    .chain_err(|| "Could not read ZIP entry")?;
                let mut contents = Vec::new();
                current_file
                    .read_to_end(&mut contents)
                    .chain_err(|| format!("Could not read: {}", current_file.name()))?;
                let is_xml = current_file.name().to_string();

                (is_xml, contents)
            };

            let contents = if (file_name.starts_with("res/") && file_name.ends_with(".xml")) ||
                              file_name == "AndroidManifest.xml" {

                decoder
                    .xml_visitor(&contents)
                    .and_then(|visitor| visitor.into_string())
                    .and_then(|string| Ok(string.into_bytes()))
                    .unwrap_or(contents)

            } else {
                contents
            };

            Self::write_file(&output_path, &file_name, &contents)
                .chain_err(|| "Could not write output file")?;

        }
        Ok(())
    }

    fn write_file<B: AsRef<Path>, R: AsRef<Path>>(base_path: B,
                                                  relative: R,
                                                  content: &[u8])
                                                  -> Result<()> {
        let full_path = base_path.as_ref().join(&relative);
        // println!("Full path: {}", full_path.display());
        fs::create_dir_all(full_path.parent().unwrap())
            .chain_err(|| "Could not create the output dir")?;

        let mut descriptor = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(full_path)
            .chain_err(|| "Could not open file to write")?;

        descriptor
            .write_all(content)
            .chain_err(|| "Could not write to target file")?;

        Ok(())
    }
}
