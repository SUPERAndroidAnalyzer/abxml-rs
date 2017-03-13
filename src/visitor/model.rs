use chunks::*;
use errors::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use model::{Identifier, Entries};
use model::Package;
use model::Resources as ResourcesTrait;
use model::Library as LibraryTrait;
use model::StringTable as StringTableTrait;
use model::LibraryBuilder;
use model::TypeSpec as TypeSpecTrait;
use model::owned::Entry;

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
            }
            _ => {
                let package_id = self.package_mask.get_package();

                let st_res = self.resources
                    .get_mut_package(package_id)
                    .and_then(|package| {
                        package.set_string_table(string_table, origin);
                        Some(())
                    });

                if st_res.is_none() {
                    error!("Could not retrieve target package");
                }
            }
        }
    }

    fn visit_package(&mut self, package: PackageRef<'a>) {
        if let Ok(package_id) = package.get_id() {
            self.package_mask = package_id << 24;

            let package_id = self.package_mask.get_package();
            let rp = Library::new(package);

            self.resources.push_package(package_id, rp);

            if self.tables.contains_key(&Origin::Global) {
                if let Some(st) = self.tables.remove(&Origin::Global) {
                    let set_result = self.resources
                        .get_mut_package(package_id)
                        .and_then(|package| {
                                      package.set_string_table(st, Origin::Global);

                                      Some(())
                                  });

                    if set_result.is_none() {
                        error!("Could not set the string table because it refers to a \
                                non-existing package");
                    }
                }
            }
        }
    }

    fn visit_table_type(&mut self, table_type: TableType<'a>) {
        let mut entries = Entries::new();

        if let Some(ref ts) = self.current_spec {
            let mask =
                ts.get_id().and_then(|id| Ok(self.package_mask | ((id as u32) << 16))).unwrap_or(0);

            let entries_result = table_type.get_entries();

            if entries_result.is_err() {
                error!("Error visiting table_type");
            } else {
                let ventries = entries_result.unwrap();
                for e in &ventries {
                    let id = mask | e.get_id();

                    if !e.is_empty() {
                        entries.insert(id, e.clone());
                    }
                }
            }
        }

        let package_id = self.package_mask.get_package();

        self.resources.get_mut_package(package_id).and_then(|package| {
                                                                package.add_entries(entries);
                                                                Some(())
                                                            });
    }

    fn visit_type_spec(&mut self, type_spec: TypeSpec<'a>) {
        self.current_spec = Some(type_spec.clone());
        let package_id = (self.package_mask >> 24) as u8;
        match self.resources.get_mut_package(package_id) {
            Some(package) => {
                package.add_type_spec(type_spec);
            }
            None => {
                error!("Type spec refers to a non existing package");
            }
        }
    }
}

pub type RefPackage<'a> = Rc<RefCell<Library<'a>>>;

#[derive(Default)]
pub struct Resources<'a> {
    packages: HashMap<u8, Library<'a>>,
    main_package: Option<u8>,
}

impl<'a> Resources<'a> {
    pub fn push_package(&mut self, package_id: u8, package: Library<'a>) {
        if self.packages.is_empty() {
            self.main_package = Some(package_id);
        }

        self.packages.insert(package_id, package);
    }
}

impl<'a> ResourcesTrait<'a> for Resources<'a> {
    type Library = Library<'a>;

    fn get_package(&self, package_id: u8) -> Option<&Self::Library> {
        self.packages.get(&package_id)
    }

    fn get_mut_package(&mut self, package_id: u8) -> Option<&mut Self::Library> {
        self.packages.get_mut(&package_id)
    }

    fn get_main_package(&self) -> Option<&Self::Library> {
        match self.main_package {
            Some(package_id) => {
                match self.packages.get(&package_id) {
                    Some(package) => Some(package),
                    None => None,
                }
            }
            _ => None,
        }
    }

    fn is_main_package(&self, package_id: u8) -> bool {
        match self.main_package {
            Some(pid) => pid == package_id,
            None => false,
        }
    }
}

pub struct Library<'a> {
    package: PackageRef<'a>,
    specs: Vec<TypeSpec<'a>>,
    string_table: Option<StringTable<'a>>,
    spec_string_table: Option<StringTable<'a>>,
    entries_string_table: Option<StringTable<'a>>,
    entries: Entries,
}

impl<'a> Library<'a> {
    pub fn new(package: PackageRef<'a>) -> Library {
        Library {
            package: package,
            specs: Vec::new(),
            string_table: None,
            spec_string_table: None,
            entries_string_table: None,
            entries: Entries::default(),
        }
    }

    fn get_spec_as_str(&self, spec_id: u32) -> Result<String> {
        if self.specs.get((spec_id - 1) as usize).is_some() {
            if let Some(ref spec_string_table) = self.spec_string_table {
                if let Ok(spec_str) = spec_string_table.get_string((spec_id - 1) as u32) {
                    return Ok((*spec_str).clone());
                }
            }
        }

        Err("Could not retrieve spec as string".into())
    }
}

impl<'a> LibraryTrait for Library<'a> {
    fn get_name(&self) -> Option<String> {
        self.package.get_name().ok()
    }

    fn format_reference(&self,
                        id: u32,
                        key: u32,
                        namespace: Option<String>,
                        prefix: &str)
                        -> Result<String> {
        let spec_id = id.get_spec() as u32;
        let spec_str = self.get_spec_as_str(spec_id)
            .chain_err(|| format!("Could not find spec: {}", spec_id))?;
        let string =
            self.get_entries_string(key)
                .chain_err(|| format!("Could not find key {} on entries string table", key))?;

        let ending = if spec_str == "attr" {
            string
        } else {
            format!("{}/{}", spec_str, string)
        };

        match namespace {
            Some(ns) => Ok(format!("{}{}:{}", prefix, ns, ending)),
            None => Ok(format!("{}{}", prefix, ending)),
        }
    }

    fn get_entry(&self, id: u32) -> Result<&Entry> {
        self.entries.get(&id).ok_or_else(|| "Could not find entry".into())
    }

    fn get_entries_string(&self, str_id: u32) -> Result<String> {
        if let Some(ref string_table) = self.entries_string_table {
            let out_string =
                string_table.get_string(str_id)
                    .chain_err(|| {
                                   format!("Could not find string {} on entries string table",
                                           str_id)
                               })?;

            return Ok((*out_string).clone());
        }

        Err("String not found on entries string table".into())
    }

    fn get_spec_string(&self, str_id: u32) -> Result<String> {
        if let Some(ref string_table) = self.spec_string_table {
            let out_string = string_table.get_string(str_id)
                .chain_err(|| format!("Could not find string {} on spec string table", str_id))?;

            return Ok((*out_string).clone());
        }

        Err("String not found on spec string table".into())
    }
}

impl<'a> LibraryBuilder<'a> for Library<'a> {
    type StringTable = StringTable<'a>;
    type TypeSpec = TypeSpec<'a>;

    fn set_string_table(&mut self, string_table: Self::StringTable, origin: Origin) {
        match origin {
            Origin::Global => self.string_table = Some(string_table),
            Origin::Spec => self.spec_string_table = Some(string_table),
            Origin::Entries => self.entries_string_table = Some(string_table),
        }
    }

    /*fn add_package(&mut self, package: Self::Library) {
        self.package = Some(package);
    }*/

    fn add_entries(&mut self, entries: Entries) {
        self.entries.extend(entries);
    }

    fn add_type_spec(&mut self, type_spec: Self::TypeSpec) {
        self.specs.push(type_spec);
    }
}
