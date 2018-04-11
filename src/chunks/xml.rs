use std::io::Cursor;
use std::rc::Rc;

use byteorder::{LittleEndian, ReadBytesExt};
use failure::{Error, ResultExt};

use model::owned::{AttributeBuf, XmlNamespaceEndBuf, XmlNamespaceStartBuf, XmlTagEndBuf,
                   XmlTagStartBuf};
use model::{AttributeTrait, NamespaceEnd, NamespaceStart, StringTable, TagEnd, TagStart};

pub struct XmlNamespaceStartWrapper<'a> {
    raw_data: &'a [u8],
}

impl<'a> XmlNamespaceStartWrapper<'a> {
    pub fn new(raw_data: &'a [u8]) -> Self {
        Self { raw_data }
    }

    pub fn get_prefix_index(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(16);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    pub fn get_namespace_index(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(20);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    pub fn to_buffer(&self) -> Result<XmlNamespaceStartBuf, Error> {
        let namespace_start = XmlNamespaceStartBuf::new(
            self.get_line()?,
            self.get_prefix_index()?,
            self.get_namespace_index()?,
        );

        Ok(namespace_start)
    }
}

impl<'a> NamespaceStart for XmlNamespaceStartWrapper<'a> {
    fn get_prefix<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>, Error> {
        let index = self.get_prefix_index()?;
        let string = string_table.get_string(index)?;

        Ok(string)
    }

    fn get_namespace<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>, Error> {
        let index = self.get_namespace_index()?;
        let string = string_table.get_string(index)?;

        Ok(string)
    }

    fn get_line(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(8);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }
}

#[allow(dead_code)]
pub struct XmlNamespaceEndWrapper<'a> {
    raw_data: &'a [u8],
}

impl<'a> XmlNamespaceEndWrapper<'a> {
    pub fn new(raw_data: &'a [u8]) -> Self {
        Self { raw_data }
    }

    pub fn get_prefix_index(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(16);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    pub fn get_namespace_index(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(20);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    pub fn to_buffer(&self) -> Result<XmlNamespaceEndBuf, Error> {
        let namespace_end = XmlNamespaceEndBuf::new(
            self.get_line()?,
            self.get_prefix_index()?,
            self.get_namespace_index()?,
        );

        Ok(namespace_end)
    }
}

impl<'a> NamespaceEnd for XmlNamespaceEndWrapper<'a> {
    fn get_line(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(8);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    fn get_prefix<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>, Error> {
        let index = self.get_prefix_index()?;
        let string = string_table.get_string(index)?;

        Ok(string)
    }

    fn get_namespace<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>, Error> {
        let index = self.get_namespace_index()?;
        let string = string_table.get_string(index)?;

        Ok(string)
    }
}

/// Contains a reference to the whole buffer and the chunk header of a `TagStart`
pub struct XmlTagStartWrapper<'a> {
    raw_data: &'a [u8],
}

impl<'a> TagStart for XmlTagStartWrapper<'a> {
    type Attribute = AttributeWrapper<'a>;

    fn get_line(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(8);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get line")?)
    }

    fn get_field1(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(12);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get data")?)
    }

    fn get_namespace_index(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(16);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get data")?)
    }

    fn get_element_name_index(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(20);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get data")?)
    }

    fn get_field2(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(24);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get data")?)
    }

    fn get_attributes_amount(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(28);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get data")?)
    }

    fn get_class(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(32);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get data")?)
    }

    fn get_attribute(&self, index: u32) -> Result<Self::Attribute, Error> {
        let offset = 36 + (index * (5 * 4)) as u64;
        let initial_position = offset as usize;
        let final_position = (offset + (5 * 4)) as usize;

        ensure!(
            self.raw_data.len() >= final_position,
            "requested attribute out of bounds"
        );

        let slice = &self.raw_data[initial_position..final_position];

        let out = AttributeWrapper::new(slice);

        Ok(out)
    }
}

impl<'a> XmlTagStartWrapper<'a> {
    /// Creates a new `XmlTagStartWrapper`
    pub fn new(raw_data: &'a [u8]) -> Self {
        Self { raw_data }
    }

    /// It converts the wrapper into a `XmlTagStartBuf` which can be later manipulated
    pub fn to_buffer(&self) -> Result<XmlTagStartBuf, Error> {
        let mut tag_start = XmlTagStartBuf::new(
            self.get_line()?,
            self.get_field1()?,
            self.get_namespace_index()?,
            self.get_element_name_index()?,
            self.get_field2()?,
            self.get_class()?,
        );

        for i in 0..self.get_attributes_amount()? {
            let attr = self.get_attribute(i).context("could not get attribute")?;
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
    fn get_namespace(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(0);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get namespace")?)
    }

    fn get_name(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(4);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get name")?)
    }

    fn get_class(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(8);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get class")?)
    }

    fn get_resource_value(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(12);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get resource value")?)
    }

    fn get_data(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(16);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get data")?)
    }
}

impl<'a> AttributeWrapper<'a> {
    /// Creates a new `AttributeWrapper`
    pub fn new(slice: &'a [u8]) -> Self {
        Self { slice }
    }

    /// It converts the wrapper into a `AttributeBuf` which can be later manipulated
    pub fn to_buffer(&self) -> Result<AttributeBuf, Error> {
        let attr = AttributeBuf::new(
            self.get_namespace()?,
            self.get_name()?,
            self.get_class()?,
            self.get_resource_value()?,
            self.get_data()?,
        );

        Ok(attr)
    }
}

#[allow(dead_code)]
pub struct XmlTagEndWrapper<'a> {
    raw_data: &'a [u8],
}

impl<'a> XmlTagEndWrapper<'a> {
    pub fn new(raw_data: &'a [u8]) -> Self {
        Self { raw_data }
    }

    pub fn to_buffer(&self) -> Result<XmlTagEndBuf, Error> {
        Ok(XmlTagEndBuf::new(self.get_id()?))
    }
}

impl<'a> TagEnd for XmlTagEndWrapper<'a> {
    fn get_id(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(5 * 4);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }
}

pub struct XmlTextWrapper<'a> {
    raw_data: &'a [u8],
}

impl<'a> XmlTextWrapper<'a> {
    pub fn new(raw_data: &'a [u8]) -> Self {
        Self { raw_data }
    }

    pub fn get_text_index(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(16);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get data")?)
    }
}
