use chunks::*;
use errors::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use model::{Identifier, Entries};
use model::Package;

use super::ChunkVisitor;
use super::Origin;

#[derive(Default)]
pub struct ModelVisitor<'a> {
    package_mask: u32,
    resources: Resources<'a>,
    current_spec: Option<TypeSpec<'a>>,
    tables: HashMap<Origin, StringTable<'a>>,
}

impl<'a> ModelVisitor<'a> {
    pub fn get_resources(&self) -> &'a Resources {
        &self.resources
    }

    pub fn get_mut_resources(&mut self) -> &'a mut Resources {
        &mut self.resources
    }
}

impl<'a> ChunkVisitor<'a> for ModelVisitor<'a> {
    fn visit_string_table(&mut self, string_table: StringTable<'a>, origin: Origin) {
        match origin {
            Origin::Global => {
                self.tables.insert(origin, string_table);
            },
            _ => {
                let package_id = self.package_mask.get_package();

                let st_res = self.resources.get_package(package_id)
                    .and_then(|package| {
                        let mut package_borrow = package.borrow_mut();
                        package_borrow.set_string_table(string_table, origin);

                        Ok(())
                    });

                if st_res.is_err() {
                    error!("Could not retrieve target package");
                }
            },
        }
    }

    fn visit_package(&mut self, package: PackageRef<'a>) {
        if let Ok(package_id) = package.get_id() {
            self.package_mask = package_id << 24;

            let package_id = self.package_mask.get_package();
            let mut rp = Library::default();
            rp.add_package(package);
            self.resources.push_package(package_id, rp);

            if self.tables.contains_key(&Origin::Global) {
                if let Some(st) = self.tables.remove(&Origin::Global) {
                    let set_result = self.resources.get_package(package_id)
                        .and_then(|package_mut| {
                            let mut package_borrow = package_mut.borrow_mut();
                            package_borrow.set_string_table(st, Origin::Global);

                            Ok(())
                        });

                    if set_result.is_err() {
                        error!("Could not set the string table because it refers to a non-existing package");
                    }
                }
            }
        }
    }

    fn visit_table_type(&mut self, table_type: TableType<'a>) {
        if let Some(ref ts) = self.current_spec {
            let result = ts.get_id()
                .and_then(|id| {
                    Ok(self.package_mask | ((id as u32) << 16))
                })
                .and_then(|mask| table_type.get_entries(ts, mask))
                .and_then(|entries| {
                    let package_id = self.package_mask.get_package();

                    self.resources.get_package(package_id)
                        .and_then(|package| {
                            let mut package_borrow = package.borrow_mut();
                            package_borrow.add_entries(entries);

                            Ok(())
                        })
                });

            if result.is_err() {
                error!("Error visiting table_type");
            }
        }
    }

    fn visit_type_spec(&mut self, type_spec: TypeSpec<'a>) {
        self.current_spec = Some(type_spec.clone());
        let package_id = (self.package_mask >> 24) as u8;
        match self.resources.get_package(package_id) {
            Ok(package) => {
                let mut package_borrow = package.borrow_mut();
                package_borrow.add_type_spec(type_spec);
            },
            Err(_) => {
                error!("Type spec refers to a non existing package");
            }
        }
    }
}

pub type RefPackage<'a> = Rc<RefCell<Library<'a>>>;

#[derive(Default)]
pub struct Resources<'a> {
    packages: HashMap<u8, RefPackage<'a>>,
    main_package: Option<u8>,
}

impl<'a> Resources<'a> {
    pub fn push_package(&mut self, package_id: u8, package: Library<'a>) {
        if self.packages.is_empty() {
            self.main_package = Some(package_id);
        }

        self.packages.insert(package_id, Rc::new(RefCell::new(package)));
    }

    pub fn get_package(&self, package_id: u8) -> Result<RefPackage<'a>> {
        self.packages.get(&package_id)
            .map(|package| package.clone())
            .ok_or("Target package not found".into())
    }

    pub fn get_main_package(&self) -> Option<RefPackage<'a>> {
        match self.main_package {
            Some(package_id) => {
                match self.packages.get(&package_id) {
                    Some(package) => Some(package.clone()),
                    None => None,
                }
            },
            _ => None,
        }
    }

    pub fn is_main_package(&self, package_id: u8) -> bool {
        match self.main_package {
            Some(pid) => pid == package_id,
            None => false,
        }
    }
}

#[derive(Default)]
pub struct Library<'a> {
    package: Option<PackageRef<'a>>,
    specs: Vec<TypeSpec<'a>>,
    string_table: Option<StringTable<'a>>,
    spec_string_table: Option<StringTable<'a>>,
    entries_string_table: Option<StringTable<'a>>,
    entries: Entries,
}

impl<'a> Library<'a> {
    pub fn set_string_table(&mut self, string_table: StringTable<'a>, origin: Origin) {
        match origin {
            Origin::Global => self.string_table = Some(string_table),
            Origin::Spec => self.spec_string_table = Some(string_table),
            Origin::Entries => self.entries_string_table = Some(string_table),
        }
    }

    pub fn add_package(&mut self, package: PackageRef<'a>) {
        self.package = Some(package);
    }

    pub fn add_entries(&mut self, entries: Entries) {
        self.entries.extend(entries);
    }

    pub fn add_type_spec(&mut self, type_spec: TypeSpec<'a>) {
        self.specs.push(type_spec);
    }

    pub fn get_name(&self) -> Option<String> {
        match self.package {
            Some(ref p) => p.get_name().ok(),
            _ => None,
        }
    }

    pub fn format_reference(&mut self, id: u32, key: u32, namespace: Option<String>, prefix: &str) -> Result<String> {
        let spec_id = id.get_spec() as u32;
        let spec_str = self.get_spec_as_str(spec_id).chain_err(|| format!("Could not find spec: {}", spec_id))?;
        let string = self.get_entries_string(key).chain_err(|| format!("Could not find key {} on entries string table", key))?;

        let ending = if spec_str == "attr" {
            string
        } else {
            format!("{}/{}", spec_str, string)
        };

        match namespace {
            Some(ns) => {
                Ok(format!("{}{}:{}", prefix, ns, ending))
            },
            None => Ok(format!("{}{}", prefix, ending)),
        }
    }

    pub fn get_entries(&self) -> &Entries {
        &self.entries
    }

    pub fn get_entry(&self, id: u32) -> Result<&Entry> {
        self.entries.get(&id).ok_or_else(|| "Could not find entry".into())
    }

    pub fn get_entries_string(&mut self, str_id: u32) -> Result<String> {
        if let Some(ref mut string_table) = self.entries_string_table {
            let out_string = string_table.get_string(str_id).chain_err(|| format!("Could not find string {} on entries string table", str_id))?;

            return Ok((*out_string).clone())
        }

        Err("String not found on entries string table".into())
    }

    pub fn get_spec_string(&mut self, str_id: u32) -> Result<String> {
        if let Some(ref mut string_table) = self.spec_string_table {
            let out_string = string_table.get_string(str_id).chain_err(|| format!("Could not find string {} on spec string table", str_id))?;

            return Ok((*out_string).clone())
        }

        Err("String not found on spec string table".into())
    }

    fn get_spec_as_str(&mut self, spec_id: u32) -> Result<String>
    {
        if let Some(_) = self.specs.get((spec_id - 1) as usize) {
            if let Some(ref mut spec_string_table) = self.spec_string_table {
                if let Ok(spec_str) = spec_string_table.get_string((spec_id - 1) as u32) {
                    return Ok((*spec_str).clone());
                }
            }
        }

        Err("Could not retrieve spec as string".into())
    }
}