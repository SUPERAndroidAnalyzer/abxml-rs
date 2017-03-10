use visitor::ModelVisitor;
use std::io::Cursor;
use visitor::Executor;
use errors::*;
use visitor::*;
use encoder::Xml;
use STR_ARSC;
use std::io::Read;
use zip::read::ZipArchive;
use std::fs;
use std;
use std::path::Path;
use std::io::Write;

pub struct BufferedDecoder {
    buffer: Vec<u8>,
}

// TODO: Implement this methods with Into/From
impl BufferedDecoder {
    pub fn from_vec(buffer: Vec<u8>) -> BufferedDecoder {
        BufferedDecoder {
            buffer: buffer,
        }
    }

    pub fn from_read<R: Read>(mut read: R) -> BufferedDecoder {
        let mut buffer = Vec::new();
        let _ = read.read_to_end(&mut buffer);

        BufferedDecoder {
            buffer: buffer,
        }
    }

    pub fn get_decoder(&self) -> Result<Decoder> {
        Decoder::new(&self.buffer)
    }
}

pub struct Decoder<'a> {
    visitor: ModelVisitor<'a>,
    buffer_android: &'a [u8],
    buffer_apk: &'a [u8],
}

impl<'a> Decoder<'a> {
    pub fn new(data: &'a [u8]) -> Result<Decoder<'a>> {
        let visitor = ModelVisitor::default();

        let mut decoder = Decoder {
            visitor: visitor,
            buffer_android: STR_ARSC,
            buffer_apk: data,
        };

        Executor::arsc(decoder.buffer_android, &mut decoder.visitor).chain_err(|| "Could not read android lib resources")?;
        Executor::arsc(decoder.buffer_apk, &mut decoder.visitor).chain_err(|| "Could not read target APK resources")?;

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
                        warn!("No string table found");
                    }
                }
            }
            None => {
                warn!("No root on target XML");
            }
        }

        Err("Could not decode XML".into())
    }
}

pub struct Apk {
    handler: ZipArchive<std::fs::File>,
    decoder: BufferedDecoder,
}

impl Apk {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut buffer = Vec::new();
        let file = std::fs::File::open(&path)?;
        let mut zip_handler = ZipArchive::new(file)?;
        zip_handler.by_name("resources.arsc")?.read_to_end(&mut buffer)?;

        let apk = Apk {
            handler: zip_handler,
            decoder: BufferedDecoder::from_vec(buffer),
        };

        Ok(apk)
    }

    /// It exports to target output_path the contents of the APK, transcoding the binary XML files
    /// found on it.
    pub fn export<P: AsRef<Path>>(&mut self, output_path: P, force: bool) -> Result<()> {
        let decoder = self.decoder.get_decoder().chain_err(|| "Could not get the decoder")?;

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
                let out = decoder
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn it_can_not_decode_an_empty_binary_xml() {
        // Empty resources.arsc file
        let buffer = vec![0,0, 12,0, 0,0,0,0, 0,0,0,0];

        let owned = BufferedDecoder::from_vec(buffer);
        let decoder = owned.get_decoder().unwrap();

        // Empty binary XML file
        let another = vec![0,0, 0, 0, 0, 0, 0, 0];
        assert!(decoder.as_xml(&another).is_err());
    }

    #[test]
    fn it_can_create_a_buffer_decoder_from_read() {
        let buffer = vec![0,0, 12,0, 0,0,0,0, 0,0,0,0];

        let owned = BufferedDecoder::from_read(Cursor::new(buffer));
        let _ = owned.get_decoder().unwrap();
    }
}