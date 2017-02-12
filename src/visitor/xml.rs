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

                println!(
                    "TagStart! {:?} {:?} {:?} {:?} {:?}",
                    tag_start.get_attribute_id(),
                    tag_start.get_name(),
                    tag_start.get_namespace(),
                    tag_start.get_attributes_amount(),
                    tag_start.get_class(),
                );

                if tag_start.get_name().unwrap() == 14 {
                    for i in 0..amount {
                        let attr = tag_start.get_attribute(i as usize).unwrap();
                        let name_idx = attr.get_name().unwrap() as usize;

                        if name_idx < self.res.len() {
                            let entry_ref = self.res.get(name_idx).unwrap();
                            let package_id = entry_ref >> 24;
                            /*let spec_id = (self.res.get(name_idx).unwrap() & 0x00FF0000) >> 16;
                            let package = self.resources.get_package(package_id as u8);
                            let pb = package.borrow();
                            let spec = pb.specs.get((spec_id - 1) as usize).unwrap().get_id();
                            println!("ResId: {:?} {:X}", self.res.get(name_idx), self.res.get(name_idx).unwrap());
                            println!("PackageID: {}; SpecId: {} Spec: {}", package_id, spec_id, spec);*/

                            let package = self.resources.get_package(package_id as u8);
                            let key = {
                                let pb = package.borrow();
                                let entry = pb.get_entries().get(&entry_ref).unwrap();
                                let parent_entry_id = entry.get_referent_id(attr.get_data().unwrap()).unwrap();
                                let parent_entry = pb.get_entries().get(&parent_entry_id).unwrap();

                                // println!("Entry: {:?}", entry);
                                // println!("Referent: {:?}", parent_entry_id);
                                // println!("Final entry: {:?}", parent_entry);

                                parent_entry.get_key()
                            };

                            let mut pb = package.borrow_mut();
                            let test = pb.get_entries_string(key);

                            // println!("String: {:?}", test);
                        }

                        /*println!(
                            "Attribute: NS-{:?} N-{:?} RV-{:?} {:?}",
                            attr.get_namespace(),
                            attr.get_name(),
                            attr.get_resource_value(),
                            attr.get_data(),
                        );*/
                    }
                }

                // let attr = tag_start.get_attribute(0).unwrap();
                /*let p = self.resources.get_main_package().unwrap();
                let p_mut = p.borrow_mut();
                println!("Resources: {:?}", self.res);

                println!(
                    "Tag start: {:?} {:?} {:?} {:?} Class: {:?} Attr: NS: {:?}; Name: {:?} Type: 0x{:X} D: {:?}",
                    tag_start.get_attribute_id(),
                    tag_start.get_name(),
                    tag_start.get_namespace(),
                    tag_start.get_attributes_amount(),
                    tag_start.get_class(),
                    attr.get_namespace(),
                    attr.get_name(),
                    attr.get_resource_value().unwrap(),
                    attr.get_data(),
                );*/
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
