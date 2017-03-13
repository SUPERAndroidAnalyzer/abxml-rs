use visitor::ModelVisitor;
use std::io::Cursor;
use visitor::Executor;
use errors::*;
use visitor::*;
use STR_ARSC;
use std::io::Read;

pub struct BufferedDecoder {
    buffer: Box<[u8]>,
}

impl From<Vec<u8>> for BufferedDecoder {
    fn from(buffer: Vec<u8>) -> BufferedDecoder {
        BufferedDecoder { buffer: buffer.into_boxed_slice() }
    }
}

impl From<Box<[u8]>> for BufferedDecoder {
    fn from(buffer: Box<[u8]>) -> BufferedDecoder {
        BufferedDecoder { buffer: buffer }
    }
}

impl BufferedDecoder {
    pub fn from_read<R: Read>(mut read: R) -> Result<BufferedDecoder> {
        let mut buffer = Vec::new();
        read.read_to_end(&mut buffer).chain_err(|| "could not read buffer")?;
        Ok(BufferedDecoder { buffer: buffer.into_boxed_slice() })
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

        Executor::arsc(decoder.buffer_android, &mut decoder.visitor).chain_err(|| {
            "Could not read android lib resources"
        })?;
        Executor::arsc(decoder.buffer_apk, &mut decoder.visitor).chain_err(|| {
            "Could not read target APK resources"
        })?;

        Ok(decoder)
    }

    pub fn get_resources(&self) -> &'a Resources {
        self.visitor.get_resources()
    }

    pub fn xml_visitor<T: AsRef<[u8]>>(&self, content: &'a T) -> Result<XmlVisitor> {
        let cursor = Cursor::new(content.as_ref());
        let mut visitor = XmlVisitor::new(self.get_resources());

        Executor::xml(cursor, &mut visitor)?;

        Ok(visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn it_can_not_decode_an_empty_binary_xml() {
        // Empty resources.arsc file
        let buffer = vec![0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let owned = BufferedDecoder::from_vec(buffer);
        let decoder = owned.get_decoder().unwrap();

        // Empty binary XML file
        let another = vec![0, 0, 0, 0, 0, 0, 0, 0];
        let xml_result = decoder.xml_visitor(&another).unwrap().into_string();
        assert!(xml_result.is_err());
    }

    #[test]
    fn it_can_create_a_buffer_decoder_from_read() {
        let buffer = vec![0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let owned = BufferedDecoder::from_read(Cursor::new(buffer));
        let _ = owned.get_decoder().unwrap();
    }
}
