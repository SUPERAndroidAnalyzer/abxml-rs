use chunks::*;
use model::{Element, ElementContainer, Namespaces};
use encoder::Xml;
use visitor::model::Resources;
use errors::*;
use model::{NamespaceStart, TagStart};
use std::rc::Rc;
use std::collections::HashMap;
use model::Resources as ResourceTrait;
use model::Library;
use model::Identifier;
use model::AttributeTrait;
use model::StringTable;
use model::Value;

use super::ChunkVisitor;
use super::Origin;

pub struct XmlVisitor<'a> {
    main_string_table: Option<StringTableWrapper<'a>>,
    namespaces: Namespaces,
    container: ElementContainer,
    res: Vec<u32>,
    resources: &'a Resources<'a>,
}

impl<'a> XmlVisitor<'a> {
    pub fn new(resources: &'a Resources<'a>) -> Self {
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

    pub fn get_string_table(&self) -> &Option<StringTableWrapper> {
        &self.main_string_table
    }

    pub fn get_resources(&self) -> &Vec<u32> {
        &self.res
    }

    pub fn arsc(&self) -> &Resources {
        self.resources
    }

    // TODO: Convert to TryInto once it will be stable
    pub fn into_string(self) -> Result<String> {
        match *self.get_root() {
            Some(ref root) => {
                match *self.get_string_table() {
                    Some(_) => {
                        return Xml::encode(self.get_namespaces(),
                                           root,
                                           self.get_resources(),
                                           self.arsc())
                                       .chain_err(|| "Could note encode XML");
                    }
                    None => {
                        warn!("No string table found");
                    }
                }
            }
            None => {
                warn!("No root on target XML");
            }
        }

        Err("Could not decode XML".into())
    }

    fn build_element(&self, tag_start: &XmlTagStartWrapper) -> Result<Element> {
        match self.main_string_table {
            Some(ref string_table) => {
                let (name, attributes) = self.get_element_data(string_table, tag_start)
                    .chain_err(|| "Could not get element data")?;
                Ok(Element::new(Rc::new(name), attributes))
            }
            None => Err("No main string table found!".into()),
        }
    }

    fn get_element_data(&self,
                        string_table: &StringTableWrapper,
                        tag_start: &XmlTagStartWrapper)
                        -> Result<(String, HashMap<String, String>)> {
        let name_index = tag_start.get_element_name_index().chain_err(|| "Name index not found")?;
        println!("Name index: {}", name_index);
        let rc_string = string_table.get_string(name_index)
            .chain_err(|| "Element name is not on the string table")?;
        println!("Rc string: {}", rc_string);
        let string = (*rc_string).clone();

        let mut attributes = HashMap::new();
        let num_attributes = tag_start.get_attributes_amount()
            .chain_err(|| "Could not get the amount of attributes")?;

        for i in 0..num_attributes {
            let current_attribute = tag_start.get_attribute(i)
                .chain_err(|| format!("Could not read attribute {} ", i))?;
            let name_index = current_attribute.get_name()?;
            let name = string_table.get_string(name_index)?;

            let value = match current_attribute.get_value()? {
                Value::StringReference(index) => (*string_table.get_string(index)?).clone(),
                _ => "".to_string(),
            };

            attributes.insert((*name).clone(), value);
        }

        Ok((string, attributes))
    }

    fn resolve_flags<R: ResourceTrait<'a>, A: AttributeTrait>(&self,
                                                              attribute: &A,
                                                              flags: u32,
                                                              xml_resources: &[u32],
                                                              resources: &R)
                                                              -> Option<String> {
        // Check if it's the special value in which the integer is an Enum
        // In that case, we return a crafted string instead of the integer itself
        let name_index = attribute.get_name().unwrap();
        if name_index < xml_resources.len() as u32 {
            self.search_values(flags, name_index, xml_resources, resources)
        } else {
            let str = format!("@flags:{}", flags);

            Some(str.to_string())
        }
    }

    fn resolve_reference<R: ResourceTrait<'a>>(&self,
                                               id: u32,
                                               resources: &R,
                                               prefix: &str)
                                               -> Result<String> {
        let res_id = id;
        let package_id = id.get_package();

        if id == 0 {
            return Ok("@null".to_string());
        }

        let is_main = resources.is_main_package(package_id);
        let package = resources.get_package(package_id)
            .ok_or_else(|| ErrorKind::Msg("Package not found".into()))?;

        let entry_key = package.get_entry(res_id).and_then(|e| Ok(e.get_key())).ok();

        if let Some(key) = entry_key {
            let namespace = if !is_main { package.get_name() } else { None };

            return package.format_reference(id, key, namespace, prefix);
        }

        Err("Error resolving reference".into())
    }

    fn search_values<R: ResourceTrait<'a>>(&self,
                                           flags: u32,
                                           name_index: u32,
                                           xml_resources: &[u32],
                                           resources: &R)
                                           -> Option<String> {
        let entry_ref = match xml_resources.get(name_index as usize) {
            Some(entry_ref) => entry_ref,
            None => return None,
        };

        let package_id = entry_ref.get_package() as u8;
        resources.get_package(package_id).and_then(|package| {
                                                       self.search_flags(flags, *entry_ref, package)
                                                   })
    }

    fn search_flags(&self, flags: u32, entry_ref: u32, package: &Library) -> Option<String> {
        let str_indexes = self.get_strings(flags, entry_ref, package);
        let str_strs: Vec<String> = str_indexes.iter()
            .map(|si| match package.get_entries_string(*si) {
                     Ok(str) => str,
                     Err(_) => {
                error!("Key not found on the string table");

                "".to_string()
            }
                 })
            .collect();

        if str_strs.is_empty() {
            None
        } else {
            let final_string = str_strs.join("|");
            Some(final_string)
        }
    }

    fn get_strings(&self, flags: u32, entry_ref: u32, package: &Library) -> Vec<u32> {
        let mut strs = Vec::new();
        let mut masks = Vec::new();

        let inner_entries = package.get_entry(entry_ref)
            .and_then(|e| e.complex())
            .and_then(|c| Ok(c.get_entries().to_vec()))
            .unwrap_or_else(|_| Vec::new());

        let mut sorted = inner_entries.to_vec();

        sorted.sort_by(|a, b| {
            let id_a = a.get_value();
            let id_b = b.get_value();

            // TODO: This code is to create an exact match with Apktool.
            // A simple descending ordering seems to be also ok.
            let mut i = id_a;
            i -= (i >> 1) & 0x55555555;
            i = (i & 0x33333333) + ((i >> 2) & 0x33333333);
            i = (i + (i >> 4)) & 0x0f0f0f0f;
            i += i >> 8;
            i += i >> 16;
            i &= 0x3f;

            let mut j = id_b;
            j -= (j >> 1) & 0x55555555;
            j = (j & 0x33333333) + ((j >> 2) & 0x33333333);
            j = (j + (j >> 4)) & 0x0f0f0f0f;
            j += j >> 8;
            j += j >> 16;
            j &= 0x3f;

            j.cmp(&i)
        });

        for ie in sorted {
            let mask = ie.get_value();
            if (mask & flags) == mask {
                let maybe_entry = package.get_entry(ie.get_id());

                match maybe_entry {
                    Ok(entry) => {
                        let mut has_to_add = true;

                        for s in &masks {
                            if mask & s == mask {
                                has_to_add = false;
                                break;
                            }
                        }

                        if has_to_add {
                            entry.simple()
                                .and_then(|s| Ok(s.get_key()))
                                .and_then(|key| {
                                    strs.push(key);
                                    masks.push(mask);
                                    Ok(())
                                })
                                .unwrap_or_else(|_| {
                                    error!("Value should be added but there was an issue reading \
                                            the entry");
                                });
                        }
                    }
                    Err(_) => {
                        info!("Some entry matched but could not found on entries");
                    }
                }
            }
        }

        strs
    }
}

impl<'a> ChunkVisitor<'a> for XmlVisitor<'a> {
    fn visit_string_table(&mut self, string_table: StringTableWrapper<'a>, _: Origin) {
        match self.main_string_table {
            Some(_) => {
                error!("Secondary table!");
            }
            None => {
                self.main_string_table = Some(string_table);
            }
        }
    }

    fn visit_xml_namespace_start(&mut self, namespace_start: XmlNamespaceStartWrapper<'a>) {
        if let Some(ref mut string_table) = self.main_string_table {
            match (namespace_start.get_namespace(string_table),
                   namespace_start.get_prefix(string_table)) {
                (Ok(namespace), Ok(prefix)) => {
                    self.namespaces.insert(namespace, prefix);
                }
                _ => {
                    error!("Error reading namespace from the string table");
                }
            }
        }
    }

    fn visit_xml_tag_start(&mut self, tag_start: XmlTagStartWrapper<'a>) {
        let element_result = self.build_element(&tag_start);
        match element_result {
            Ok(element) => {
                self.container.start_element(element);
            }
            Err(e) => {
                println!("Error: {}", e);
                error!("Could not build a XML element")
            }
        }
    }

    fn visit_xml_tag_end(&mut self, _: XmlTagEndWrapper<'a>) {
        self.container.end_element()
    }

    fn visit_resource(&mut self, resource: ResourceWrapper<'a>) {
        if let Ok(res) = resource.get_resources() {
            self.res = res;
        }
    }
}
