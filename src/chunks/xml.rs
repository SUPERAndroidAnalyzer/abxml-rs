use crate::model::{
    owned::{AttributeBuf, XmlNamespaceEndBuf, XmlNamespaceStartBuf, XmlTagEndBuf, XmlTagStartBuf},
    AttributeTrait, NamespaceEnd, NamespaceStart, StringTable, TagEnd, TagStart,
};
use anyhow::{ensure, Context, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::{io::Cursor, rc::Rc};

#[derive(Debug)]
pub struct XmlNamespaceStartWrapper<'a> {
    raw_data: &'a [u8],
}

impl<'a> XmlNamespaceStartWrapper<'a> {
    pub fn new(raw_data: &'a [u8]) -> Self {
        Self { raw_data }
    }

    pub fn prefix_index(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(16);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    pub fn namespace_index(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(20);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    pub fn to_buffer(&self) -> Result<XmlNamespaceStartBuf> {
        let namespace_start =
            XmlNamespaceStartBuf::new(self.line()?, self.prefix_index()?, self.namespace_index()?);

        Ok(namespace_start)
    }
}

impl<'a> NamespaceStart for XmlNamespaceStartWrapper<'a> {
    fn prefix<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>> {
        let index = self.prefix_index()?;
        let string = string_table.get_string(index)?;

        Ok(string)
    }

    fn namespace<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>> {
        let index = self.namespace_index()?;
        let string = string_table.get_string(index)?;

        Ok(string)
    }

    fn line(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(8);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct XmlNamespaceEndWrapper<'a> {
    raw_data: &'a [u8],
}

impl<'a> XmlNamespaceEndWrapper<'a> {
    pub fn new(raw_data: &'a [u8]) -> Self {
        Self { raw_data }
    }

    pub fn prefix_index(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(16);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    pub fn namespace_index(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(20);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    pub fn to_buffer(&self) -> Result<XmlNamespaceEndBuf> {
        let namespace_end =
            XmlNamespaceEndBuf::new(self.line()?, self.prefix_index()?, self.namespace_index()?);

        Ok(namespace_end)
    }
}

impl<'a> NamespaceEnd for XmlNamespaceEndWrapper<'a> {
    fn line(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(8);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    fn prefix<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>> {
        let index = self.prefix_index()?;
        let string = string_table.get_string(index)?;

        Ok(string)
    }

    fn namespace<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>> {
        let index = self.namespace_index()?;
        let string = string_table.get_string(index)?;

        Ok(string)
    }
}

/// Contains a reference to the whole buffer and the chunk header of a `TagStart`
#[derive(Debug)]
pub struct XmlTagStartWrapper<'a> {
    raw_data: &'a [u8],
}

impl<'a> TagStart for XmlTagStartWrapper<'a> {
    type Attribute = AttributeWrapper<'a>;

    fn line(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(8);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get line")?)
    }

    fn field1(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(12);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get data")?)
    }

    fn namespace_index(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(16);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get data")?)
    }

    fn element_name_index(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(20);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get data")?)
    }

    fn field2(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(24);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get data")?)
    }

    fn attributes_amount(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(28);

        Ok(u32::from(
            cursor
                .read_u16::<LittleEndian>()
                .context("could not get data")?,
        ))
    }

    fn class(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(32);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get data")?)
    }

    fn attribute(&self, index: u32) -> Result<Self::Attribute> {
        let offset = 36 + u64::from(index) * 5 * 4;
        let initial_position = offset as usize;
        let final_position = (offset + 5 * 4) as usize;

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
    pub fn to_buffer(&self) -> Result<XmlTagStartBuf> {
        let mut tag_start = XmlTagStartBuf::new(
            self.line()?,
            self.field1()?,
            self.namespace_index()?,
            self.element_name_index()?,
            self.field2()?,
            self.class()?,
        );

        for i in 0..self.attributes_amount()? {
            let attr = self.attribute(i).context("could not get attribute")?;
            tag_start.add_attribute(attr.to_buffer()?);
        }

        Ok(tag_start)
    }
}

/// Contains a slice that represents an attribute
#[derive(Debug)]
pub struct AttributeWrapper<'a> {
    slice: &'a [u8],
}

impl<'a> AttributeTrait for AttributeWrapper<'a> {
    fn namespace(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(0);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get namespace")?)
    }

    fn name(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(4);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get name")?)
    }

    fn class(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(8);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get class")?)
    }

    fn resource_value(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(12);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get resource value")?)
    }

    fn data(&self) -> Result<u32> {
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
    pub fn to_buffer(&self) -> Result<AttributeBuf> {
        let attr = AttributeBuf::new(
            self.namespace()?,
            self.name()?,
            self.class()?,
            self.resource_value()?,
            self.data()?,
        );

        Ok(attr)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct XmlTagEndWrapper<'a> {
    raw_data: &'a [u8],
}

impl<'a> XmlTagEndWrapper<'a> {
    pub fn new(raw_data: &'a [u8]) -> Self {
        Self { raw_data }
    }

    pub fn to_buffer(&self) -> Result<XmlTagEndBuf> {
        Ok(XmlTagEndBuf::new(self.id()?))
    }
}

impl<'a> TagEnd for XmlTagEndWrapper<'a> {
    fn id(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(5 * 4);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }
}

#[derive(Debug)]
pub struct XmlTextWrapper<'a> {
    raw_data: &'a [u8],
}

impl<'a> XmlTextWrapper<'a> {
    pub fn new(raw_data: &'a [u8]) -> Self {
        Self { raw_data }
    }

    pub fn text_index(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(16);

        Ok(cursor
            .read_u32::<LittleEndian>()
            .context("could not get data")?)
    }
}
