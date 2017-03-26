use chunks::*;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use errors::*;
use model::StringTable;
use model::owned::{XmlTagStartBuf, XmlTagEndBuf, XmlNamespaceStartBuf, XmlNamespaceEndBuf,
                   AttributeBuf};
use model::{TagStart, TagEnd, NamespaceStart, NamespaceEnd, AttributeTrait};

pub struct XmlDecoder;

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

    pub fn to_buffer(&self) -> Result<XmlNamespaceStartBuf> {
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

    pub fn to_buffer(&self) -> Result<XmlNamespaceEndBuf> {
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

/// Contains a reference to the whole buffer and the chunk header of a `TagStart`
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
    /// Creates a new `XmlTagStartWrapper`
    pub fn new(raw_data: &'a [u8], header: ChunkHeader) -> Self {
        XmlTagStartWrapper {
            raw_data: raw_data,
            header: header,
        }
    }

    /// It converts the wrapper into a `XmlTagStartBuf` which can be later manipulated
    pub fn to_buffer(&self) -> Result<XmlTagStartBuf> {
        let mut tag_start = XmlTagStartBuf::new(self.get_line()?,
                                                self.get_field1()?,
                                                self.get_namespace_index()?,
                                                self.get_element_name_index()?,
                                                self.get_field2()?,
                                                self.get_class()?);

        for i in 0..self.get_attributes_amount()? {
            let attr = self.get_attribute(i).chain_err(|| "Could not get attribute")?;
            tag_start.add_attribute(attr.to_buffer()?);
        }

        Ok(tag_start)
    }
}

/// Contains a slice that represents an attribute
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
    /// Creates a new `AttributeWrapper`
    pub fn new(slice: &'a [u8]) -> Self {
        AttributeWrapper { slice: slice }
    }

    /// It converts the wrapper into a `AttributeBuf` which can be later manipulated
    pub fn to_buffer(&self) -> Result<AttributeBuf> {
        let attr = AttributeBuf::new(self.get_namespace()?,
                                     self.get_name()?,
                                     self.get_class()?,
                                     self.get_resource_value()?,
                                     self.get_data()?);

        Ok(attr)
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

    pub fn to_buffer(&self) -> Result<XmlTagEndBuf> {
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
