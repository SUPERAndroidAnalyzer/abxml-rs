use std::io::Cursor;
use chunks::*;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;
use document::{Namespaces, Element, ElementContainer, Entries};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use super::Resources;
use super::ChunkVisitor;
use super::Origin;

pub struct XmlVisitor<'a, 'b> {
    main_string_table: Option<StringTable<'a>>,
    namespaces: Namespaces,
    container: ElementContainer,
    res: Vec<u32>,
    resources: &'b Resources<'b>,
}

impl<'a, 'b> XmlVisitor<'a, 'b> {
    pub fn new(resources: &'b Resources<'b>) -> XmlVisitor {
        XmlVisitor {
            main_string_table: None,
            namespaces: Namespaces::default(),
            container: ElementContainer::default(),
            res: Vec::new(),
            resources: resources,
        }
    }

    pub fn get_namespaces(&self) -> &Namespaces {
        &self.namespaces
    }

    pub fn get_root(&self) -> &Option<Element> {
        self.container.get_root()
    }

    pub fn get_string_table(&self) -> &Option<StringTable> {
        &self.main_string_table
    }

    pub fn get_resources(&self) -> &Vec<u32> {
        &self.res
    }
}

impl <'a, 'b> ChunkVisitor<'a> for XmlVisitor<'a, 'b> {
    fn visit_string_table(&mut self, string_table: StringTable<'a>, _: Origin) {
        match self.main_string_table {
            Some(_) => {
                println!("Secondary table!");
            },
            None => {
                self.main_string_table = Some(string_table);
            },
        }
    }

    fn visit_xml_namespace_start(&mut self, namespace_start: XmlNamespaceStart<'a>) {
        match self.main_string_table {
            Some(ref mut string_table) => {
                self.namespaces.insert(
                    namespace_start.get_namespace(string_table).unwrap(),
                    namespace_start.get_prefix(string_table).unwrap(),
                );
            },
            None => {
                println!("No main string table found!");
            }
        }
    }

    fn visit_xml_tag_start(&mut self, tag_start: XmlTagStart<'a>) {
        match self.main_string_table {
            Some(ref mut string_table) => {
                let amount = tag_start.get_attributes_amount().unwrap();
                let (attributes, element_name) = tag_start.get_tag(&self.namespaces, string_table).unwrap();
                let element = Element::new(element_name, attributes);
                self.container.start_element(element);
            },
            None => {
                println!("No main string table found!");
            }
        }
    }

    fn visit_xml_tag_end(&mut self, _: XmlTagEnd<'a>) {
        self.container.end_element()
    }

    fn visit_resource(&mut self, resource: Resource<'a>) {
        let res = resource.get_resources();
        self.res = res;
    }
}
