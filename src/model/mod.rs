//! Representations of logical structures found on android binary files

pub mod builder;
mod element;
pub mod owned;
mod value;

use self::owned::Entry;
pub use self::{
    element::{Element, ElementContainer, Tag},
    value::Value,
};
use crate::visitor::Origin;
use anyhow::Result;
use log::info;
use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

pub type Namespaces = BTreeMap<String, String>;
pub type Entries = HashMap<u32, Entry>;

pub trait Identifier {
    fn package(&self) -> u8;
    fn spec(&self) -> u8;
    fn id(&self) -> u16;
}

impl Identifier for u32 {
    fn package(&self) -> u8 {
        let mut package_id = (self >> 24) as u8;

        if package_id == 0 {
            package_id = 1;
            info!("Resource with package id 0 found. Recreate id with current package id");
        }

        package_id
    }

    fn spec(&self) -> u8 {
        ((self & 0x00FF_0000) >> 16) as u8
    }

    fn id(&self) -> u16 {
        (self & 0xFFFF) as u16
    }
}

// Traits
pub trait StringTable {
    fn strings_len(&self) -> u32;
    fn styles_len(&self) -> u32;
    fn get_string(&self, idx: u32) -> Result<Rc<String>>;
}

// TODO: Decide if the trait should return Results or not
pub trait Package {
    fn id(&self) -> Result<u32>;
    fn name(&self) -> Result<String>;
}

pub trait Library {
    fn name(&self) -> Option<String>;
    fn format_reference(
        &self,
        id: u32,
        key: u32,
        namespace: Option<String>,
        prefix: &str,
    ) -> Result<String>;
    // fn entries(&self) -> &Entries;
    fn entry(&self, id: u32) -> Result<&Entry>;
    fn entries_string(&self, str_id: u32) -> Result<Rc<String>>;
    fn spec_string(&self, str_id: u32) -> Result<Rc<String>>;
}

pub trait LibraryBuilder<'a> {
    type StringTable: StringTable;
    type TypeSpec: TypeSpec;

    fn set_string_table(&mut self, string_table: Self::StringTable, origin: Origin);
    fn add_entries(&mut self, entries: Entries);
    fn add_type_spec(&mut self, type_spec: Self::TypeSpec) -> Result<()>;
}

pub trait Resources<'a> {
    type Library: Library + LibraryBuilder<'a>;

    fn package(&self, package_id: u8) -> Option<&Self::Library>;
    fn mut_package(&mut self, package_id: u8) -> Option<&mut Self::Library>;
    fn main_package(&self) -> Option<&Self::Library>;
    fn is_main_package(&self, package_id: u8) -> bool;
}

/// Trait that represents a XML tag start
pub trait TagStart {
    /// Type of the attributes
    type Attribute: AttributeTrait;

    /// Return the ¿line in which the tag appear?
    fn line(&self) -> Result<u32>;
    /// Return the content of the unknown field1
    fn field1(&self) -> Result<u32>;
    /// Return the namespace index. If there is no namespace, it will return 0xFFFF_FFFF
    fn namespace_index(&self) -> Result<u32>;
    /// Returns the index of the tag name on the string table
    fn element_name_index(&self) -> Result<u32>;
    /// Return the content of the unknown field1
    fn field2(&self) -> Result<u32>;
    /// Return the amount of attributes this tag contains
    fn attributes_amount(&self) -> Result<u32>;
    /// Returns the ¿class?
    fn class(&self) -> Result<u32>;
    /// Returns the attribute on the `index` position or error if it is greater than
    /// `attributes_amount()`.
    fn attribute(&self, index: u32) -> Result<Self::Attribute>;
}

pub trait AttributeTrait {
    /// Return the namespace index. If there is no namespace, it will return 0xFFFF_FFFF
    fn namespace(&self) -> Result<u32>;
    /// Returns the index of the attribute on the string table
    fn name(&self) -> Result<u32>;
    /// Returns the ¿class?
    fn class(&self) -> Result<u32>;
    /// Returns the data type (see `Values`)
    fn resource_value(&self) -> Result<u32>;
    /// Returns the value (see `Values`)
    fn data(&self) -> Result<u32>;

    /// Creates a `Value` depending on the data type and data value
    fn value(&self) -> Result<Value> {
        let data_type = ((self.resource_value()? >> 24) & 0xFF) as u8;
        let data_value = self.data()?;
        let class = self.class()?;

        let value = if class == 0xFFFF_FFFF {
            Value::create(data_type, data_value)?
        } else {
            Value::StringReference(class)
        };

        Ok(value)
    }
}

pub trait TagEnd {
    fn id(&self) -> Result<u32>;
}

pub trait NamespaceStart {
    fn line(&self) -> Result<u32>;
    fn prefix<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>>;
    fn namespace<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>>;
}

pub trait NamespaceEnd {
    fn line(&self) -> Result<u32>;
    fn prefix<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>>;
    fn namespace<S: StringTable>(&self, string_table: &S) -> Result<Rc<String>>;
}

pub trait TypeSpec {
    fn id(&self) -> Result<u16>;
    fn amount(&self) -> Result<u32>;
    fn flag(&self, index: u32) -> Result<u32>;
}

pub trait TableType {
    type Configuration: Configuration;

    fn id(&self) -> Result<u8>;
    fn amount(&self) -> Result<u32>;
    fn configuration(&self) -> Result<Self::Configuration>;
    fn entry(&self, index: u32) -> Result<Entry>;
}

pub trait Configuration {
    fn size(&self) -> Result<u32>;
    fn mcc(&self) -> Result<u16>;
    fn mnc(&self) -> Result<u16>;
    fn language(&self) -> Result<String>;
    fn region(&self) -> Result<String>;
    fn orientation(&self) -> Result<u8>;
    fn touchscreen(&self) -> Result<u8>;
    fn density(&self) -> Result<u16>;
    fn keyboard(&self) -> Result<u8>;
    fn navigation(&self) -> Result<u8>;
    fn input_flags(&self) -> Result<u8>;
    fn width(&self) -> Result<u16>;
    fn height(&self) -> Result<u16>;
    fn sdk_version(&self) -> Result<u16>;
    fn min_sdk_version(&self) -> Result<u16>;
    fn screen_layout(&self) -> Result<u8>;
    fn ui_mode(&self) -> Result<u8>;
    fn smallest_screen(&self) -> Result<u16>;
    fn screen_width(&self) -> Result<u16>;
    fn screen_height(&self) -> Result<u16>;
    fn locale_script(&self) -> Result<Option<String>>;
    fn locale_variant(&self) -> Result<Option<String>>;
    fn secondary_layout(&self) -> Result<Option<u8>>;
}

#[cfg(test)]
mod tests {
    use crate::model::Identifier;

    #[test]
    fn it_extracts_package_id() {
        assert_eq!(2130837685.get_package(), 127)
    }

    #[test]
    fn it_converts_package_id_if_is_0() {
        assert_eq!(131253.get_package(), 1)
    }

    #[test]
    fn it_extracts_spec_id() {
        assert_eq!(2130837685.get_spec(), 2)
    }

    #[test]
    fn it_extracts_id() {
        assert_eq!(2130837685.get_id(), 181)
    }
}
