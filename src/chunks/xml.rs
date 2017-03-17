use chunks::*;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use errors::*;
use std::clone::Clone;
use model::{Identifier, Namespaces, Value, Attribute};
use model::StringTable;
use model::owned::{XmlTagStartBuf, XmlTagEndBuf, XmlNamespaceStartBuf, XmlNamespaceEndBuf,
                   AttributeBuf};
use model::{TagStart, TagEnd, NamespaceStart, NamespaceEnd, AttributeTrait};

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

impl<'a> TagStart for XmlTagStartWrapper<'a> {
    type Attribute = AttributeWrapper<'a>;

    fn get_line(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(8));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get line")
    }

    fn get_field1(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(12));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }

    fn get_namespace_index(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(16));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }

    fn get_element_name_index(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(20));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }

    fn get_field2(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(24));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }

    fn get_attributes_amount(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(28));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }

    fn get_class(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(32));

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }

    fn get_attribute(&self, index: u32) -> Result<Self::Attribute> {
        let offset = 36 + (index * (5 * 4)) as u64;
        let initial_position: usize = self.header.absolute(offset) as usize;
        let final_position: usize = self.header.absolute(offset + (5 * 4)) as usize;
        let slice = &self.raw_data[initial_position..final_position];

        let out = AttributeWrapper::new(slice);

        Ok(out)
    }
}

impl<'a> XmlTagStartWrapper<'a> {
    pub fn new(raw_data: &'a [u8], header: ChunkHeader) -> Self {
        XmlTagStartWrapper {
            raw_data: raw_data,
            header: header,
        }
    }

    pub fn to_owned(self) -> Result<XmlTagStartBuf> {
        let mut tag_start = XmlTagStartBuf::new(self.get_line()?,
                                                self.get_field1()?,
                                                self.get_namespace_index()?,
                                                self.get_element_name_index()?,
                                                self.get_field2()?,
                                                self.get_class()?);

        for i in 0..self.get_attributes_amount()? {
            let attr = self.get_attribute(i).chain_err(|| "Could not get attribute")?;
            tag_start.add_attribute(attr.to_owned()?);
        }

        Ok(tag_start)
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

pub struct AttributeWrapper<'a> {
    slice: &'a [u8],
}

impl<'a> AttributeTrait for AttributeWrapper<'a> {
    fn get_namespace(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(0);

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get namespace")
    }

    fn get_name(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(4);

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get name")
    }

    fn get_class(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(8);

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get class")
    }

    fn get_resource_value(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(12);

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get resource value")
    }

    fn get_data(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(16);

        cursor.read_u32::<LittleEndian>().chain_err(|| "Could not get data")
    }
}

impl<'a> AttributeWrapper<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        AttributeWrapper { slice: slice }
    }

    pub fn to_owned(self) -> Result<AttributeBuf> {
        let attr = AttributeBuf::new(self.get_namespace()?,
                                     self.get_name()?,
                                     self.get_class()?,
                                     self.get_resource_value()?,
                                     self.get_data()?);

        Ok(attr)
    }
}

/*
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
*/

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
