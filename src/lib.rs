#![feature(repeat_str, test)]
extern crate byteorder;
extern crate test;
extern crate quick_xml;

mod document;
pub mod encoder;

use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Error, ErrorKind};
use document::*;
use std::rc::Rc;

const TOKEN_START_DOCUMENT: u32 = 0x00080003;
const TOKEN_STRING_TABLE: u32 = 0x001C0001;
const TOKEN_RESOURCE_TABLE: u32 = 0x00080180;
const TOKEN_NAMESPACE_START: u32 = 0x00100100;
const TOKEN_NAMESPACE_END: u32 = 0x00100101;
const TOKEN_START_TAG: u32 = 0x00100102;
const TOKEN_END_TAG: u32 = 0x00100103;

const TOKEN_VOID: u32 = 0xFFFFFFFF;

const TOKEN_TYPE_REFERENCE_ID: u32 = 0x01000008;
const TOKEN_TYPE_ATTRIBUTE_REFERENCE_ID: u32 = 0x02000008;
const TOKEN_TYPE_STRING: u32 = 0x03000008;
const TOKEN_TYPE_DIMENSION: u32 = 0x05000008;
const TOKEN_TYPE_FRACTION: u32 = 0x06000008;
const TOKEN_TYPE_INTEGER: u32 = 0x10000008;
const TOKEN_TYPE_FLOAT: u32 = 0x04000008;
const TOKEN_TYPE_FLAGS: u32 = 0x11000008;
const TOKEN_TYPE_BOOLEAN: u32 = 0x12000008;
const TOKEN_TYPE_COLOR: u32 = 0x1C000008;
const TOKEN_TYPE_COLOR2: u32 = 0x1D000008;

pub struct BinaryXmlDecoder<'a> {
    cursor: Cursor<&'a [u8]>,
    raw_data: &'a [u8],
    document: Document,
    element_container: ElementContainer,
}

impl<'a> BinaryXmlDecoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        BinaryXmlDecoder {
            cursor: Cursor::new(data),
            raw_data: data,
            document: Document::default(),
            element_container: ElementContainer::new(),
        }
    }

    pub fn decode(mut self) -> Result<Document, Error> {
        // Read document header
        self.parse_document_header()?;

        // Loop trough all of the frames
        loop {
            let initial_position = self.cursor.position();
            if initial_position == self.document.header.size as u64 {
                // We are at the end of the document. We are done!
                break;
            }
            let code = self.cursor.read_u32::<LittleEndian>();
            let chunk_size = self.cursor.read_u32::<LittleEndian>()?;

            match code {
                Ok(TOKEN_STRING_TABLE) => self.parse_string_table(initial_position as u32)?,
                Ok(TOKEN_RESOURCE_TABLE) => self.parse_resource_table(chunk_size)?,
                Ok(TOKEN_NAMESPACE_START) => self.parse_namespace_start()?,
                Ok(TOKEN_NAMESPACE_END) => self.parse_namespace_end()?,
                Ok(TOKEN_START_TAG) => self.parse_start_tag()?,
                Ok(TOKEN_END_TAG) => self.parse_end_tag()?,
                Ok(_) => {
                    () /* Add some warning on a logger */
                }
                Err(_) => break,
            }

            self.cursor.set_position(initial_position + chunk_size as u64);
        }

        self.document.root_element = self.element_container.get_root().unwrap();

        Ok(self.document)
    }

    fn parse_document_header(&mut self) -> Result<(), Error> {
        let first_token = self.cursor.read_u32::<LittleEndian>()?;
        let chunk_size = self.cursor.read_u32::<LittleEndian>()?;
        if first_token != TOKEN_START_DOCUMENT {
            return Err(Error::new(ErrorKind::Other,
                                  format!("Document not starting with the START_DOCUMENT \
                                           number: {:X}",
                                          first_token)));
        }

        let header = Header { size: chunk_size };
        self.document.header = header;

        Ok(())
    }

    fn parse_string_table(&mut self, initial_position: u32) -> Result<(), Error> {
        let mut header_string_table = HeaderStringTable::default();

        header_string_table.string_amount = self.cursor.read_u32::<LittleEndian>()?;
        header_string_table.style_amount = self.cursor.read_u32::<LittleEndian>()?;
        header_string_table.unknown = self.cursor.read_u32::<LittleEndian>()?;
        header_string_table.string_offset = self.cursor.read_u32::<LittleEndian>()?;
        header_string_table.style_offset = self.cursor.read_u32::<LittleEndian>()?;

        let mut string_table = StringTable::default();
        let str_offset = initial_position + header_string_table.string_offset;

        for _ in 0..header_string_table.string_amount {
            let current_offset = self.cursor.read_u32::<LittleEndian>()?;
            let position = str_offset + current_offset;
            let s = self.parse_string(position)?;
            string_table.strings.push(Rc::new(s));
        }

        self.document.header_string_table = header_string_table;
        self.document.string_table = string_table;

        Ok(())
    }

    fn parse_string(&mut self, offset: u32) -> Result<String, Error> {
        let size1: u32 = self.raw_data[offset as usize] as u32;
        let size2: u32 = self.raw_data[(offset + 1) as usize] as u32;

        if size1 == size2 {
            let str_len = size1;
            let position = offset + 2;
            let a = position;
            let b = position + str_len;

            let subslice: &[u8] = &self.raw_data[a as usize..b as usize];

            let raw_str: Vec<u8> = subslice.iter()
                .cloned()
                .collect();

            match String::from_utf8(raw_str) {
                Ok(s) => Ok(s),
                Err(e) => Err(Error::new(ErrorKind::Other, e)),
            }
        } else {
            let str_len = ((size2 << 8) & 0xFF00) | size1 & 0xFF;
            let position = offset + 2;
            let mut i = 0;
            let a = position;
            let b = position + (str_len * 2);

            let subslice: &[u8] = &self.raw_data[a as usize..b as usize];

            let raw_str: Vec<u8> = subslice.iter()
                .cloned()
                .filter(|_| {
                    let result = i % 2 == 0;
                    i = i + 1;

                    result
                })
                .collect();

            match String::from_utf8(raw_str) {
                Ok(s) => Ok(s),
                Err(e) => Err(Error::new(ErrorKind::Other, e)),
            }
        }
    }

    fn parse_resource_table(&mut self, chunk: u32) -> Result<(), Error> {
        let amount = (chunk / 4) - 2;
        let resource_table = (1..amount)
            .into_iter()
            .map(|_| self.cursor.read_u32::<LittleEndian>().unwrap())
            .collect::<Vec<u32>>();

        self.document.resource_table = ResourceTable { resources: resource_table };

        Ok(())
    }

    fn parse_namespace_start(&mut self) -> Result<(), Error> {
        let _line = self.cursor.read_u32::<LittleEndian>()?;
        let _unknown = self.cursor.read_u32::<LittleEndian>()?;
        let prefix_idx = self.cursor.read_u32::<LittleEndian>()?;
        let uri_idx = self.cursor.read_u32::<LittleEndian>()?;

        let prefix = self.document.string_table.strings.get(prefix_idx as usize).unwrap().clone();
        let uri = self.document.string_table.strings.get(uri_idx as usize).unwrap().clone();

        self.document.resources.insert(uri, prefix);

        Ok(())
    }

    fn parse_namespace_end(&mut self) -> Result<(), Error> {
        let _line = self.cursor.read_u32::<LittleEndian>()?;
        let _unknown = self.cursor.read_u32::<LittleEndian>()?;
        let _prefix_idx = self.cursor.read_u32::<LittleEndian>()?;
        let _uri_idx = self.cursor.read_u32::<LittleEndian>()?;

        // TODO: What should we do on NS end

        Ok(())
    }

    fn parse_start_tag(&mut self) -> Result<(), Error> {
        let _line = self.cursor.read_u32::<LittleEndian>()?;
        let _unknown = self.cursor.read_u32::<LittleEndian>()?;
        let _ns_uri = self.cursor.read_u32::<LittleEndian>()?;
        let element_name_idx = self.cursor.read_u32::<LittleEndian>()?;
        let _unknwon2 = self.cursor.read_u32::<LittleEndian>()?;
        let attributes_amount = self.cursor.read_u32::<LittleEndian>()? as usize;
        let _unknwon3 = self.cursor.read_u32::<LittleEndian>()?;

        let element_name =
            self.document.string_table.strings.get(element_name_idx as usize).unwrap().clone();
        let mut attributes = Vec::new();
        for _ in 0..attributes_amount {
            let attribute = self.parse_attribute()?;
            attributes.push(attribute);
        }

        if attributes.len() != attributes_amount {
            return Err(Error::new(ErrorKind::Other,
                                  format!("Expected a different amount of elements {} {}",
                                          attributes.len(),
                                          attributes_amount)));
        }

        self.element_container.start_element(Element::new(element_name.clone(), attributes));

        Ok(())
    }

    fn parse_attribute(&mut self) -> Result<Attribute, Error> {
        let attr_ns_idx = self.cursor.read_u32::<LittleEndian>()?;
        let attr_name_idx = self.cursor.read_u32::<LittleEndian>()?;
        let attr_value_idx = self.cursor.read_u32::<LittleEndian>()?;
        let attr_type_idx = self.cursor.read_u32::<LittleEndian>()?;
        let attr_data = self.cursor.read_u32::<LittleEndian>()?;

        let mut namespace = None;
        let mut prefix = None;

        if attr_ns_idx != TOKEN_VOID {
            let uri = self.document.string_table.strings.get(attr_ns_idx as usize).unwrap().clone();

            match self.document.resources.get(&uri) {
                Some(uri_prefix) => {
                    namespace = Some(uri);
                    prefix = Some(uri_prefix.clone());
                }
                None =>(),
            };
        }

        let value = if attr_value_idx == TOKEN_VOID {
            self.parse_value(attr_type_idx, attr_data)?
        } else {
            Value::String(self.document
                .string_table
                .strings
                .get(attr_value_idx as usize)
                .unwrap()
                .clone())
        };

        let element_name =
            self.document.string_table.strings.get(attr_name_idx as usize).unwrap().clone();

        Ok(Attribute::new(element_name, value, namespace, prefix))
    }

    fn parse_end_tag(&mut self) -> Result<(), Error> {
        let _line = self.cursor.read_u32::<LittleEndian>()?;
        let _unknown = self.cursor.read_u32::<LittleEndian>()?;
        let _uri_idx = self.cursor.read_u32::<LittleEndian>()?;
        let name_idx = self.cursor.read_u32::<LittleEndian>()?;

        let _ = self.document.string_table.strings.get(name_idx as usize).unwrap().clone();

        // TODO: Is this needed?
        // let mut maybe_uri = None;
        // if uri_idx != TOKEN_VOID {
        // maybe_uri =
        // Some(element_name);
        // }


        self.element_container.end_element();

        Ok(())
    }

    fn parse_value(&mut self, value_type: u32, data: u32) -> Result<Value, Error> {
        let value = match value_type {
            TOKEN_TYPE_REFERENCE_ID => Value::ReferenceId(format!("@id/0x{:#8}", data)),
            TOKEN_TYPE_ATTRIBUTE_REFERENCE_ID => {
                Value::AttributeReferenceId(format!("?id/0x{:#8}", data))
            }
            TOKEN_TYPE_STRING => {
                Value::String(self.document
                    .string_table
                    .strings
                    .get(data as usize)
                    .unwrap()
                    .clone())
            }
            TOKEN_TYPE_DIMENSION => {
                let units: [&str; 6] = ["px", "dp", "sp", "pt", "in", "mm"];
                let mut size = (data >> 8).to_string();
                let unit_idx = data & 0xFF;

                match units.get(unit_idx as usize) {
                    Some(unit) => size.push_str(unit),
                    None => {
                        return Err(Error::new(ErrorKind::Other,
                                              format!("Expected a valid unit index. Got: {}",
                                                      unit_idx)));
                    }
                }

                Value::Dimension(size)
            }
            TOKEN_TYPE_FRACTION => {
                let value = (data as f64) / (0x7FFFFFFF as f64);
                let formatted_fraction = format!("{:.*}", 2, value);

                Value::Fraction(formatted_fraction)
            }
            TOKEN_TYPE_INTEGER => Value::Integer(data as u64),
            TOKEN_TYPE_FLAGS => Value::Flags(data as u64),
            TOKEN_TYPE_FLOAT => Value::Float(data as f64),
            TOKEN_TYPE_BOOLEAN => {
                if data > 0 {
                    Value::Boolean(true)
                } else {
                    Value::Boolean(false)
                }
            }
            TOKEN_TYPE_COLOR => {
                let formatted_color = format!("#{:#8}", data);
                Value::Color(formatted_color)
            }
            TOKEN_TYPE_COLOR2 => {
                let formatted_color = format!("#{:#8}", data);
                Value::Color2(formatted_color)
            }
            _ => Value::Unknown,

        };

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::Path;
    use super::*;

    use test::Bencher;

    #[test]
    fn it_works() {
        let original_file = file_get_contents("tests/binary_manifests/AndroidManifest-ce.xml");
        let parser = BinaryXmlDecoder::new(&original_file);
        let result = parser.decode();
        println!("{:?}", result);
    }

    #[bench]
    fn bench_manifest_parsing(b: &mut Bencher) {
        let original_file = file_get_contents("tests/binary_manifests/AndroidManifest-ce.xml");

        b.iter(move || {
            let parser = BinaryXmlDecoder::new(&original_file);
            parser.decode().unwrap();
        });
    }

    fn file_get_contents(path: &str) -> Vec<u8> {
        let path = Path::new(path);
        let display = path.display();

        let mut file = match File::open(&path) {
            // The `description` method of `io::Error` returns a string that
            // describes the error
            Err(why) => panic!("couldn't open {}: {}", display, why.description()),
            Ok(file) => file,
        };

        // Read the file contents into a string, returns `io::Result<usize>`
        let mut v: Vec<u8> = Vec::new();
        match file.read_to_end(&mut v) {
            Err(why) => panic!("couldn't read {}: {}", display, why.description()),
            Ok(_) => (),
        };

        return v;
    }
}
