use chunks::*;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use errors::*;
use std::clone::Clone;
use model::{Identifier, Namespaces, Value, Attribute};
use model::StringTable;
use model::owned::{XmlTagEndBuf, XmlNamespaceStartBuf, XmlNamespaceEndBuf};
use model::TagEnd;
use model::NamespaceStart;
use model::NamespaceEnd;

pub struct XmlDecoder;

const TOKEN_VOID: u32 = 0xFFFFFFFF;

impl XmlDecoder {
    pub fn decode_xml_namespace_start<'a>(cursor: &mut Cursor<&'a [u8]>,
                                          header: &ChunkHeader)
                                          -> Result<Chunk<'a>> {
        let xnsw = XmlNamespaceStartWrapper::new(cursor.get_ref(), *header);
        Ok(Chunk::XmlNamespaceStart(xnsw))
    }

    pub fn decode_xml_namespace_end<'a>(cursor: &mut Cursor<&'a [u8]>,
                                        header: &ChunkHeader)
                                        -> Result<Chunk<'a>> {
        let xnsw = XmlNamespaceEndWrapper::new(cursor.get_ref(), *header);
        Ok(Chunk::XmlNamespaceEnd(xnsw))
    }

    pub fn decode_xml_tag_start<'a>(cursor: &mut Cursor<&'a [u8]>,
                                    header: &ChunkHeader)
                                    -> Result<Chunk<'a>> {
        let xnsw = XmlTagStartWrapper::new(cursor.get_ref(), *header);
        Ok(Chunk::XmlTagStart(xnsw))
    }

    pub fn decode_xml_tag_end<'a>(cursor: &mut Cursor<&'a [u8]>,
                                  header: &ChunkHeader)
                                  -> Result<Chunk<'a>> {
        let xnsw = XmlTagEndWrapper::new(cursor.get_ref(), *header);
        Ok(Chunk::XmlTagEnd(xnsw))
    }

    pub fn decode_xml_text<'a>(cursor: &mut Cursor<&'a [u8]>,
                               header: &ChunkHeader)
                               -> Result<Chunk<'a>> {
        let xnsw = XmlTextWrapper::new(cursor.get_ref(), *header);
        Ok(Chunk::XmlText(xnsw))
    }
}

pub struct XmlNamespaceStartWrapper<'a> {
    raw_data: &'a [u8],
    header: ChunkHeader,
}

impl<'a> XmlNamespaceStartWrapper<'a> {
    pub fn new(raw_data: &'a [u8], header: ChunkHeader) -> Self {
        XmlNamespaceStartWrapper {
            raw_data: raw_data,
            header: header,
        }
    }

    pub fn get_prefix_index(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(16));

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    pub fn get_namespace_index(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(20));

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    pub fn to_owned(self) -> Result<XmlNamespaceStartBuf> {
        let namespace_start = XmlNamespaceStartBuf::new(self.get_line()?,
                                                        self.get_prefix_index()?,
                                                        self.get_namespace_index()?);

        Ok(namespace_start)
    }
}

impl<'a> NamespaceStart for XmlNamespaceStartWrapper<'a> {
    fn get_prefix<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>> {
        let index = self.get_prefix_index()?;
        let string = string_table.get_string(index)?;

        Ok(string)
    }

    fn get_namespace<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>> {
        let index = self.get_namespace_index()?;
        let string = string_table.get_string(index)?;

        Ok(string)
    }

    fn get_line(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(8));

        Ok(cursor.read_u32::<LittleEndian>()?)
    }
}

#[allow(dead_code)]
pub struct XmlNamespaceEndWrapper<'a> {
    raw_data: &'a [u8],
    header: ChunkHeader,
}

impl<'a> XmlNamespaceEndWrapper<'a> {
    pub fn new(raw_data: &'a [u8], header: ChunkHeader) -> Self {
        XmlNamespaceEndWrapper {
            raw_data: raw_data,
            header: header,
        }
    }

    pub fn get_prefix_index(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(16));

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    pub fn get_namespace_index(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(20));

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    pub fn to_owned(self) -> Result<XmlNamespaceEndBuf> {
        let namespace_end = XmlNamespaceEndBuf::new(self.get_line()?,
                                                    self.get_prefix_index()?,
                                                    self.get_namespace_index()?);

        Ok(namespace_end)
    }
}

impl<'a> NamespaceEnd for XmlNamespaceEndWrapper<'a> {
    fn get_line(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(8));

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    fn get_prefix<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>> {
        let index = self.get_prefix_index()?;
        let string = string_table.get_string(index)?;

        Ok(string)
    }

    fn get_namespace<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>> {
        let index = self.get_namespace_index()?;
        let string = string_table.get_string(index)?;

        Ok(string)
    }
}

pub struct XmlTagStartWrapper<'a> {
    raw_data: &'a [u8],
    header: ChunkHeader,
}

impl<'a> XmlTagStartWrapper<'a> {
    pub fn new(raw_data: &'a [u8], header: ChunkHeader) -> Self {
        XmlTagStartWrapper {
            raw_data: raw_data,
            header: header,
        }
    }

    pub fn get_line(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(8));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get line")
    }

    pub fn get_field1(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(12));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }

    pub fn get_ns_uri(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(16));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }

    pub fn get_element_name_index(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(20));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }

    pub fn get_field2(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(24));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }

    pub fn get_attributes_amount(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(28));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }

    pub fn get_class(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(32));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }

    pub fn get_tag_start(&self,
                         namespaces: &Namespaces,
                         string_table: &StringTableWrapper)
                         -> Result<(Vec<Attribute>, Rc<String>)> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(36));

        let element_name = string_table.get_string(self.get_element_name_index()?)?;

        let mut attributes = Vec::new();

        for _ in 0..self.get_attributes_amount()? {
            let attribute = self.decode_attribute(&mut cursor, namespaces, string_table)?;
            attributes.push(attribute);
        }

        if attributes.len() != self.get_attributes_amount()? as usize {
            return Err("Excptected a distinct amount of elements".into());
        }

        Ok((attributes, element_name))
    }

    pub fn get_attribute_values(&self, index: usize) -> Result<Attributes> {
        let mut values = Vec::new();

        let mut cursor = Cursor::new(self.raw_data);
        let initial_position = 36 + (index * (5 * 4));
        cursor.set_position(self.header.absolute(initial_position as u64));

        for i in 0..5 {
            if i == 3 {
                values.push(cursor.read_u32::<LittleEndian>()?.get_package() as u32);
            } else {
                values.push(cursor.read_u32::<LittleEndian>()?);
            }
        }

        let out = Attributes::new(values);

        Ok(out)
    }

    fn decode_attribute(&self,
                        cursor: &mut Cursor<&[u8]>,
                        namespaces: &Namespaces,
                        string_table: &StringTableWrapper)
                        -> Result<Attribute> {
        let attr_ns_idx = cursor.read_u32::<LittleEndian>()?;
        let attr_name_idx = cursor.read_u32::<LittleEndian>()?;
        let attr_value_idx = cursor.read_u32::<LittleEndian>()?;

        let _size = cursor.read_u16::<LittleEndian>()?;
        cursor.read_u8()?;
        let attr_type_idx = cursor.read_u8()?;
        let attr_data = cursor.read_u32::<LittleEndian>()?;

        let mut namespace = None;
        let mut prefix = None;

        if attr_ns_idx != TOKEN_VOID {
            let uri = string_table.get_string(attr_ns_idx)?.clone();

            if let Some(uri_prefix) = namespaces.get(&uri) {
                namespace = Some(uri);
                prefix = Some(uri_prefix.clone());
            }
        }

        let value = if attr_value_idx == TOKEN_VOID {
            Value::new(attr_type_idx as u8, attr_data, string_table)?
        } else {
            Value::String(string_table.get_string(attr_value_idx)?.clone())
        };

        let element_name = string_table.get_string(attr_name_idx)?.clone();

        Ok(Attribute::new(element_name, value, namespace, prefix, attr_name_idx))
    }
}

pub struct Attributes {
    values: Vec<u32>,
}

impl Attributes {
    pub fn new(values: Vec<u32>) -> Self {
        Attributes { values: values }
    }

    pub fn get_namespace(&self) -> Result<u32> {
        self.values
            .get(0)
            .cloned()
            .ok_or_else(|| "Error reading namespace".into())
    }

    pub fn get_name(&self) -> Result<u32> {
        self.values
            .get(1)
            .cloned()
            .ok_or_else(|| "Error reading name".into())
    }

    pub fn get_class(&self) -> Result<()> {
        Err("unimplemented".into())
    }

    pub fn get_attr_id(&self) -> Result<()> {
        Err("unimplemented".into())
    }

    pub fn get_resource_value(&self) -> Result<u8> {
        self.values
            .get(3)
            .map(|v| *v as u8)
            .ok_or_else(|| "Error reading value".into())
    }

    pub fn get_data(&self) -> Result<u32> {
        self.values
            .get(4)
            .cloned()
            .ok_or_else(|| "Error reading data".into())
    }
}

pub struct XmlTagStart<'a> {
    wrapper: XmlTagStartWrapper<'a>,
}

impl<'a> XmlTagStart<'a> {
    pub fn new(wrapper: XmlTagStartWrapper<'a>) -> Self {
        XmlTagStart { wrapper: wrapper }
    }

    pub fn get_namespace(&self) -> Result<u32> {
        self.wrapper.get_ns_uri()
    }

    pub fn get_name(&self) -> Result<u32> {
        self.wrapper.get_element_name_index()
    }

    pub fn get_attribute_id(&self) -> Result<u16> {
        let count = self.wrapper.get_attributes_amount()?;

        let mut high = count >> 16;

        if high > 0 {
            high -= 1;
        }
        Ok(high as u16)
    }

    pub fn get_attributes_amount(&self) -> Result<u32> {
        self.wrapper.get_attributes_amount()
    }

    pub fn get_class(&self) -> Result<(u32, u32)> {
        let class = self.wrapper.get_class()?;

        let high = Self::convert_id(class >> 16);
        let low = Self::convert_id(class & 0xFFFF);

        Ok((high, low))
    }

    pub fn get_tag(&self,
                   namespaces: &Namespaces,
                   string_table: &StringTableWrapper)
                   -> Result<(Vec<Attribute>, Rc<String>)> {
        self.wrapper.get_tag_start(namespaces, string_table)
    }

    pub fn get_attribute(&self, index: usize) -> Result<Attributes> {
        let amount = self.wrapper.get_attributes_amount()?;

        if index > amount as usize {
            return Err("Attribute out of range".into());
        }

        self.wrapper.get_attribute_values(index)
    }

    fn convert_id(input: u32) -> u32 {
        let mut id = input;

        if id > 0 {
            id -= 1;
        }

        id
    }
}

#[allow(dead_code)]
pub struct XmlTagEndWrapper<'a> {
    raw_data: &'a [u8],
    header: ChunkHeader,
}

impl<'a> XmlTagEndWrapper<'a> {
    pub fn new(raw_data: &'a [u8], header: ChunkHeader) -> Self {
        XmlTagEndWrapper {
            raw_data: raw_data,
            header: header,
        }
    }

    pub fn to_owned(self) -> Result<XmlTagEndBuf> {
        Ok(XmlTagEndBuf::new(self.get_id()?))
    }
}

impl<'a> TagEnd for XmlTagEndWrapper<'a> {
    fn get_id(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(5 * 4));

        Ok(cursor.read_u32::<LittleEndian>()?)
    }
}

#[allow(dead_code)]
pub struct XmlTextWrapper<'a> {
    raw_data: &'a [u8],
    header: ChunkHeader,
}

impl<'a> XmlTextWrapper<'a> {
    pub fn new(raw_data: &'a [u8], header: ChunkHeader) -> Self {
        XmlTextWrapper {
            raw_data: raw_data,
            header: header,
        }
    }

    pub fn get_text_index(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(16));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }
}

#[allow(dead_code)]
pub struct XmlText<'a> {
    wrapper: XmlTextWrapper<'a>,
}

impl<'a> XmlText<'a> {
    pub fn new(wrapper: XmlTextWrapper<'a>) -> Self {
        XmlText { wrapper: wrapper }
    }

    pub fn get_text(&self, string_table: &StringTableWrapper) -> Result<Rc<String>> {
        let index = self.wrapper.get_text_index()?;
        string_table.get_string(index)
    }
}
