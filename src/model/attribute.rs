use std::rc::Rc;
use model::Value;
use visitor::Resources;
use model::Identifier;
// use visitor::model::RefPackage;
use errors::*;
use model::Resources as ResourcesTrait;
use model::Library;

#[derive(Debug)]
pub struct Attribute {
    name: Rc<String>,
    namespace: Option<Rc<String>>,
    prefix: Option<Rc<String>>,
    value: Value,
    name_index: u32,
}

impl Attribute {
    pub fn new(name: Rc<String>,
               value: Value,
               namespace: Option<Rc<String>>,
               prefix: Option<Rc<String>>,
               name_index: u32,
    ) -> Self {
        Attribute {
            name: name,
            namespace: namespace,
            prefix: prefix,
            value: value,
            name_index: name_index,
        }
    }

    pub fn get_name(&self) -> Rc<String> {
        self.name.clone()
    }

    pub fn get_value_as_str(&self) -> String {
        self.value.to_string()
    }

    pub fn get_value(&self) -> &Value {
        &self.value
    }

    pub fn get_prefix(&self) -> Option<Rc<String>> {
        self.prefix.clone()
    }

    pub fn get_name_index(&self) -> u32 {
        self.name_index
    }

    pub fn resolve_flags(&self, flags: u32, xml_resources: &[u32], resources: &Resources) -> Option<String> {
        // Check if it's the special value in which the integer is an Enum
        // In that case, we return a crafted string instead of the integer itself
        let name_index = self.get_name_index();
        if name_index < xml_resources.len() as u32 {
            self.search_values(flags, name_index, xml_resources, resources)
        } else {
            let str = format!("@flags:{}", flags);

            Some(str.to_string())
        }
    }

    pub fn resolve_reference(&self, id: u32, resources: &Resources, prefix: &str) -> Result<String> {
        let res_id = id;
        let package_id = id.get_package();

        if id == 0 {
            return Ok("@null".to_string());
        }

        let is_main = resources.is_main_package(package_id);
        let package = resources.get_package(package_id).unwrap();

        let entry_key = package
            .get_entries()
            .get(&res_id)
            .and_then(|e| Some(e.get_key()));

        if let Some(key) = entry_key {
            let namespace = if !is_main {
                package.get_name()
            } else {
                None
            };

            return package.format_reference(id, key, namespace, prefix);
        }

        Err("Error resolving reference".into())
    }

    fn search_values(&self, flags: u32, name_index: u32, xml_resources: &[u32], resources: &Resources) -> Option<String> {
        let entry_ref = match xml_resources.get(name_index as usize) {
            Some(entry_ref) => entry_ref,
            None => return None,
        };

        let package_id = entry_ref.get_package() as u8;

        resources.get_package(package_id)
            .and_then(|package| {
                self.search_flags(flags, *entry_ref, package)
            })
    }

    fn search_flags(&self, flags: u32, entry_ref: u32, package: &Library) -> Option<String> {
        let str_indexes = self.get_strings(flags, entry_ref, package);

        let str_strs: Vec<String> = str_indexes
            .iter()
            .map(|si| {
                match package.get_entries_string(*si) {
                    Ok(str) => str,
                    Err(_) => {
                        error!("Key not found on the string table");

                        "".to_string()
                    },
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
            .unwrap_or(Vec::new());
        let mut sorted = inner_entries.to_vec();

        sorted.sort_by(|a, b| {
            let id_a = a.get_value();
            let id_b = b.get_value();

            // TODO: This code is to create an exact match with Apktool. A simple descending ordering seems to be also ok.
            let mut i = id_a;
            i = i - ((i >> 1) & 0x55555555);
            i = (i & 0x33333333) + ((i >> 2) & 0x33333333);
            i = (i + (i >> 4)) & 0x0f0f0f0f;
            i = i + (i >> 8);
            i = i + (i >> 16);
            i = i & 0x3f;

            let mut j = id_b;
            j = j - ((j >> 1) & 0x55555555);
            j = (j & 0x33333333) + ((j >> 2) & 0x33333333);
            j = (j + (j >> 4)) & 0x0f0f0f0f;
            j = j + (j >> 8);
            j = j + (j >> 16);
            j = j & 0x3f;

            j.cmp(&i)
        });

        for ie in sorted {
            let mask = ie.get_value();

            if (mask & flags) == mask {
                let maybe_entry = package.get_entry(ie.get_id());

                match maybe_entry {
                    Ok(entry) => {
                        let mut has_to_add = true;

                        for s in masks.iter() {
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
                                    error!("Value should be added but there was an issue reading the entry");
                                });
                        }
                    },
                    Err(_) => {
                        info!("Some entry matched but could not found on entries");
                    }
                }
            }
        }

        strs
    }
}
