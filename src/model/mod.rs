use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use chunks::table_type::Entry;
use errors::*;

mod element;
mod value;
mod attribute;
pub mod owned;
mod builder;

pub use self::element::Element as Element;
pub use self::element::ElementContainer as ElementContainer;
pub use self::value::Value as Value;
pub use self::attribute::Attribute as Attribute;

use visitor::Origin;
use chunks::TypeSpec;

pub type Namespaces = BTreeMap<Rc<String>, Rc<String>>;
pub type Entries = HashMap<u32, Entry>;

pub trait Identifier {
    fn get_package(&self) -> u8;
    fn get_spec(&self) -> u8;
    fn get_id(&self) -> u16;
}

impl Identifier for u32 {
    fn get_package(&self) -> u8 {
        let mut package_id = (self >> 24) as u8;

        if package_id == 0 {
            package_id = 1;
            info!("Resource with package id 0 found. Recreate id with current package id");
        }

        package_id
    }

    fn get_spec(&self) -> u8 {
        ((self & 0x00FF0000) >> 16) as u8
    }

    fn get_id(&self) -> u16 {
        (self & 0xFFFF) as u16
    }
}

// Traits
pub trait StringTable {
    fn get_strings_len(&self) -> u32;
    fn get_styles_len(&self) ->  u32;
    fn get_string(&self, idx: u32) -> Result<Rc<String>>;
}

// TODO: Decide if the trait should return Results or not
pub trait Package {
    fn get_id(&self) -> Result<u32>;
    fn get_name(&self) -> Result<String>;
}

pub trait Library {
    fn get_name(&self) -> Option<String>;
    fn format_reference(&self, id: u32, key: u32, namespace: Option<String>, prefix: &str) -> Result<String>;
    // fn get_entries(&self) -> &Entries;
    fn get_entry(&self, id: u32) -> Result<&Entry>;
    fn get_entries_string(&self, str_id: u32) -> Result<String>;
    fn get_spec_string(&self, str_id: u32) -> Result<String>;
}

pub trait LibraryBuilder<'a> {
    type StringTable: StringTable;

    fn set_string_table(&mut self, string_table: Self::StringTable, origin: Origin);
    fn add_entries(&mut self, entries: Entries);
    fn add_type_spec(&mut self, type_spec: TypeSpec<'a>);
}

pub trait Resources<'a> {
    type Library: Library + LibraryBuilder<'a>;

    fn get_package(&self, package_id: u8) -> Option<&Self::Library>;
    fn get_mut_package(&mut self, package_id: u8) -> Option<&mut Self::Library>;
    fn get_main_package(&self) -> Option<&Self::Library>;
    fn is_main_package(&self, package_id: u8) -> bool;
}

pub trait TagStart {
    fn get_tag_start(&self) -> Result<(Vec<Attribute>, Rc<String>)>;
}

#[cfg(test)]
mod tests {
    use model::Identifier;

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