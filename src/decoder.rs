use visitor::ModelVisitor;
use std::io::Cursor;
use visitor::Executor;
use errors::*;
use visitor::*;
use encoder::Xml;
use STR_ARSC;
use std::io::Read;

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