extern crate byteorder;

mod document;

use std::io::Cursor;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::io::{Error, ErrorKind};
use document::*;

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
    states_stack: Vec<u64>,
    cursor: Cursor<&'a [u8]>,
    raw_data: &'a [u8],
    document: Document,
    current_element: Option<Element>,
}

impl<'a> BinaryXmlDecoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        BinaryXmlDecoder {
            states_stack: Vec::new(),
            cursor: Cursor::new(data),
            raw_data: data,
            document: Document::default(),
            current_element: None,
        }
    }

    pub fn uncompress_xml(&mut self) -> Result<Vec<u8>, Error> {
        let v = Vec::new();

        println!("Buffer with {} bytes", self.cursor.get_ref().len());

        // Read document header
        self.parse_document_header()?;
        // Read strings table
        self.parse_string_table()?;
        // Read resource table
        self.parse_resource_table();

        loop {
            let code = self.cursor.read_u32::<LittleEndian>();
            match code {
                Ok(TOKEN_NAMESPACE_START) => {println!("NS Start"); self.parse_namespace_start()?},
                Ok(TOKEN_NAMESPACE_END) => {println!("NS End"); self.parse_namespace_end()?},
                Ok(TOKEN_START_TAG) => {println!("Tag Start"); self.parse_start_tag()?},
                Ok(TOKEN_END_TAG) => {println!("Tag end"); self.parse_end_tag()?},
                Ok(t) => {println!("Unkown {}", t); panic!("")},
                Err(_) => break,
                // _ => unreachable!(""),
            }
        }

        /*let next_section = self.cursor.read_u32::<LittleEndian>()?;
        println!("Next section: {:X} {}", next_section, next_section);*/

        println!("{:?}", self.document);
        Ok(v)
    }

    fn parse_document_header(&mut self) -> Result<(), Error> {
        let first_token = self.cursor.read_u32::<LittleEndian>()?;
        let chunk_size = self.cursor.read_u32::<LittleEndian>()?;
        if first_token != TOKEN_START_DOCUMENT {
            return Err(Error::new(ErrorKind::Other, format!("Document not starting with the START_DOCUMENT number: {:X}", first_token)));
        }

        let header = Header {
            size: chunk_size,
        };
        self.document.header = header;

        Ok(())
    }

    fn parse_string_table(&mut self) -> Result<(), Error> {
        let mut header_string_table = HeaderStringTable::default();
        let initial_position = self.cursor.position() as u32;
        let token = self.cursor.read_u32::<LittleEndian>()?;
        println!("Initial position: {}", initial_position);

        if token != TOKEN_STRING_TABLE {
            return Err(Error::new(ErrorKind::Other, format!("Expected TOKEN_STRING_TABLE ({:X}), but found: {:X}", TOKEN_STRING_TABLE, token)));
        }

        println!("TOKEN_STRING_TABLE: {:X}, {}", TOKEN_STRING_TABLE, TOKEN_STRING_TABLE);

        header_string_table.chunk = self.cursor.read_u32::<LittleEndian>()?;
        header_string_table.string_amount = self.cursor.read_u32::<LittleEndian>()?;
        header_string_table.style_amount = self.cursor.read_u32::<LittleEndian>()?;
        header_string_table.unknown = self.cursor.read_u32::<LittleEndian>()?;
        header_string_table.string_offset = self.cursor.read_u32::<LittleEndian>()?;
        header_string_table.style_offset = self.cursor.read_u32::<LittleEndian>()?;

        let mut string_table = StringTable::default();
        let str_offset = initial_position + header_string_table.string_offset;

        for i in 0..header_string_table.string_amount {
            let current_offset = self.cursor.read_u32::<LittleEndian>()?;
            let position = str_offset + current_offset;
            let s = self.parse_string(position)?;
            string_table.strings.push(s);
        }

        self.cursor.set_position(
            (initial_position + header_string_table.chunk) as u64
        );
        self.document.header_string_table = header_string_table;
        self.document.string_table = string_table;

        Ok(())
    }

    fn parse_string(&mut self, offset: u32) -> Result<String, Error> {
        self.push_state_forward(offset as u64);
        let size1 = self.cursor.read_u8()? as u64;
        let size2 = self.cursor.read_u8()? as u64;

        if size1 == size2 {
            // Collect iterative
            println!("Size 1 === size2");
        } else {
            let str_len = ((size2 << 8) & 0xFF00) |
                            size1 & 0xFF;
            let position = self.cursor.position();
            let mut i = 0;
            let a = position;
            let b = position + (str_len * 2);

            let subslice: &[u8] = &self.raw_data[a as usize..b as usize];

            let raw_str: Vec<u8> = subslice
                .iter()
                .cloned()
                .filter(|_| {
                    let result = i % 2 == 0;
                    i = i + 1;

                    result
                })
                .collect();

            self.pop_state();
            return match String::from_utf8(raw_str) {
                Ok(s) => Ok(s),
                Err(e) => Err(Error::new(ErrorKind::Other, e)),
            }
        }


        println!("Sizes: {}, {}", size1, size2);
        let s = String::new();

        self.pop_state();
        Ok(s)
    }

    fn parse_resource_table(&mut self) -> Result<(), Error> {
        let initial_position = self.cursor.position();
        let token = self.cursor.read_u32::<LittleEndian>()?;
        let chunk = self.cursor.read_u32::<LittleEndian>()?;
        self.document.header_resource_table = HeaderResourceTable {
            chunk: chunk,
        };

        if token != TOKEN_RESOURCE_TABLE {
            return Err(Error::new(ErrorKind::Other, format!("Expected TOKEN_RESOURCE_TABLE ({:X}), but found: {:X}", TOKEN_RESOURCE_TABLE, token)));
        }

        let amount = (chunk / 4) - 2;
        let resource_table = (1..amount).into_iter().map(|_| {
            self.cursor.read_u32::<LittleEndian>().unwrap()
        }).collect::<Vec<u32>>();

        self.document.resource_table = ResourceTable {
                resources: resource_table,
        };
        self.cursor.set_position(initial_position + chunk as u64);

        Ok(())
    }

    fn parse_namespace_start(&mut self) -> Result<(), Error> {
        let chunk = self.cursor.read_u32::<LittleEndian>()?;
        let line = self.cursor.read_u32::<LittleEndian>()?;
        let unknown = self.cursor.read_u32::<LittleEndian>()?;
        let prefix_idx = self.cursor.read_u32::<LittleEndian>()?;
        let uri_idx = self.cursor.read_u32::<LittleEndian>()?;

        let prefix = self.document.string_table.strings.get(prefix_idx as usize).unwrap().clone();
        let uri = self.document.string_table.strings.get(uri_idx as usize).unwrap().clone();

        self.document.resources.insert(uri, prefix);

        Ok(())
    }

    fn parse_namespace_end(&mut self) -> Result<(), Error> {
        let chunk = self.cursor.read_u32::<LittleEndian>()?;
        let line = self.cursor.read_u32::<LittleEndian>()?;
        let unknown = self.cursor.read_u32::<LittleEndian>()?;
        let prefix_idx = self.cursor.read_u32::<LittleEndian>()?;
        let uri_idx = self.cursor.read_u32::<LittleEndian>()?;

        let uri = self.document.string_table.strings.get(uri_idx as usize).unwrap().clone();
        self.document.resources.remove(&uri);

        Ok(())
    }

    fn parse_start_tag(&mut self) -> Result<(), Error> {
        let initial_position = self.cursor.position();
        let chunk = self.cursor.read_u32::<LittleEndian>()?;
        let line = self.cursor.read_u32::<LittleEndian>()?;
        let unknown = self.cursor.read_u32::<LittleEndian>()?;
        let ns_uri = self.cursor.read_u32::<LittleEndian>()?;
        let element_name_idx = self.cursor.read_u32::<LittleEndian>()?;
        let unknwon2 = self.cursor.read_u32::<LittleEndian>()?;
        let attributes_amount = self.cursor.read_u32::<LittleEndian>()? as usize;
        let unknwon3 = self.cursor.read_u32::<LittleEndian>()?;

        let element_name = self.document.string_table.strings.get(element_name_idx as usize).unwrap().clone();
        let mut attributes = Vec::new();
        for i in 0..attributes_amount {
            let attribute = self.parse_attribute()?;
            attributes.push(attribute);
        }

        if attributes.len() != attributes_amount {
            return Err(Error::new(ErrorKind::Other, format!("Expected a different amount of elements {} {}", attributes.len(), attributes_amount)));
        }

        self.current_element = Some(Element::new(element_name, attributes));
        println!("Start element: {:?}", self.current_element);
        println!("Current position: {}; Following: {}", self.cursor.position(), initial_position + chunk as u64);
        self.cursor.set_position(initial_position + chunk as u64 - 4);

        Ok(())
    }

    fn parse_attribute(&mut self) -> Result <Attribute, Error> {
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
                },
                None => ()
            };
        }

        let value = if attr_value_idx == TOKEN_VOID {
            self.parse_value(attr_type_idx, attr_data)?
        } else {
            Value::String(self.document.string_table.strings.get(attr_value_idx as usize).unwrap().clone())
        };

        let element_name = self.document.string_table.strings.get(attr_name_idx as usize).unwrap().clone();

        Ok(Attribute::new(element_name, value, namespace, prefix))
    }

    fn parse_end_tag(&mut self) -> Result<(), Error> {
        let initial_position = self.cursor.position();
        let chunk = self.cursor.read_u32::<LittleEndian>()?;
        let line = self.cursor.read_u32::<LittleEndian>()?;
        let unknown = self.cursor.read_u32::<LittleEndian>()?;
        let uri_idx = self.cursor.read_u32::<LittleEndian>()?;
        let name_idx = self.cursor.read_u32::<LittleEndian>()?;

        let element_name = self.document.string_table.strings.get(name_idx as usize).unwrap().clone();
        let mut maybe_uri = None;
        if uri_idx != TOKEN_VOID {
            maybe_uri = Some(self.document.string_table.strings.get(uri_idx as usize).unwrap().clone());
        }

        println!("{:?}", self.current_element);
        println!("Element name: {}", element_name);
        println!("Uri: {:?}", maybe_uri);

        self.cursor.set_position(initial_position + chunk as u64 - 4);

        Ok(())
    }

    fn parse_value(&mut self, value_type: u32, data: u32) -> Result<Value, Error> {
        let value = match value_type {
            TOKEN_TYPE_REFERENCE_ID => {
                Value::String(format!("@id/0x{:#8}", data))
            },
            TOKEN_TYPE_ATTRIBUTE_REFERENCE_ID => {
                Value::String(format!("?id/0x{:#8}", data))
            },
            TOKEN_TYPE_STRING => {
                Value::String(self.document.string_table.strings.get(data as usize).unwrap().clone())
            },
            TOKEN_TYPE_DIMENSION => {
                let units: [&str; 6] = ["px", "dp", "sp", "pt", "in", "mm"];
                let mut size = (data >> 8).to_string();
                let unit_idx = data & 0xFF;

                match units.get(unit_idx as usize) {
                    Some(unit) => size.push_str(unit),
                    None => {
                        return Err(Error::new(ErrorKind::Other, format!("Expected a valid unit index. Got: {}", unit_idx)));
                    }
                }

                Value::Dimension(size)
            },
            TOKEN_TYPE_FRACTION => {
                let value = (data as f64) / (0x7FFFFFFF as f64);
                let formatted_fraction = format!("{:.*}", 2, value);

                Value::String(formatted_fraction)
            },
            TOKEN_TYPE_INTEGER | TOKEN_TYPE_FLAGS => {
                let formatted = format!("{}", data);

                Value::String(formatted)
            },
            TOKEN_TYPE_FLOAT => {
                Value::Float(data as f64)
            },
            TOKEN_TYPE_BOOLEAN => {
                if data > 0 {
                    Value::String("true".to_string())
                } else {
                    Value::String("false".to_string())
                }
            },
            TOKEN_TYPE_COLOR | TOKEN_TYPE_COLOR2 => {
                let formatted_color = format!("#{:#8}", data);
                Value::String(formatted_color)
            },
            _ => Value::Unknown,

        };

        Ok(value)
    }

    fn push_state_forward(&mut self, new_offset: u64) {
        self.states_stack.push(self.cursor.position());
        self.cursor.set_position(new_offset);
    }

    fn pop_state(&mut self) {
        if self.states_stack.len() > 0 {
            let new_offset = self.states_stack.pop().unwrap();
            self.cursor.set_position(new_offset);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::Path;
    use super::*;
    use std::io::Cursor;
    use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

    #[test]
    fn it_works() {
        let original_file = file_get_contents("AndroidManifest.xml");
        // let mut rdr = Cursor::new(original_file);
        let mut parser = BinaryXmlDecoder::new(&original_file);
        let result = parser.uncompress_xml();
        println!("{:?}", result);
        // uncompress_xml(original_file).unwrap();

        // println!("{:X}", rdr.read_u32::<LittleEndian>().unwrap());

        // println!("{:?}", original_file);
        panic!("");
    }

    fn file_get_contents(path: &str) -> Vec<u8>{
        let path = Path::new(path);
        let display = path.display();

        let mut file = match File::open(&path) {
            // The `description` method of `io::Error` returns a string that
            // describes the error
            Err(why) => panic!("couldn't open {}: {}", display,
                                                       why.description()),
            Ok(file) => file,
        };

        // Read the file contents into a string, returns `io::Result<usize>`
        let mut v: Vec<u8> = Vec::new();
        match file.read_to_end(&mut v) {
            Err(why) => panic!("couldn't read {}: {}", display,
                                                       why.description()),
            Ok(_) => (),
        };

        return v;
    }
}
