use super::{ChunkVisitor, Origin};
use crate::{
    chunks::{
        PackageWrapper, StringTableCache, StringTableWrapper, TableTypeWrapper, TypeSpecWrapper,
    },
    model::{
        owned::Entry, Entries, Identifier, Library as LibraryTrait, LibraryBuilder,
        Resources as ResourcesTrait, StringTable as StringTableTrait, TypeSpec as TypeSpecTrait,
    },
};
use anyhow::{anyhow, bail, Context, Result};
use log::error;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Default, Debug)]
pub struct ModelVisitor<'a> {
    package_mask: u32,
    resources: Resources<'a>,
    current_spec: Option<TypeSpecWrapper<'a>>,
    tables: HashMap<Origin, StringTableCache<StringTableWrapper<'a>>>,
}

impl<'a> ModelVisitor<'a> {
    pub fn resources(&self) -> &'a Resources {
        &self.resources
    }

    pub fn mut_resources(&mut self) -> &'a mut Resources {
        &mut self.resources
    }
}

impl<'a> ChunkVisitor<'a> for ModelVisitor<'a> {
    fn visit_string_table(&mut self, string_table: StringTableWrapper<'a>, origin: Origin) {
        if let Origin::Global = origin {
            self.tables
                .insert(origin, StringTableCache::new(string_table));
        } else {
            let package_id = self.package_mask.package();

            let st_res = self.resources.mut_package(package_id).and_then(|package| {
                package.set_string_table(StringTableCache::new(string_table), origin);
                Some(())
            });

            if st_res.is_none() {
                error!("Could not retrieve target package");
            }
        }
    }

    fn visit_package(&mut self, package: PackageWrapper<'a>) {
        if let Ok(package_id) = package.id() {
            self.package_mask = package_id << 24;

            let package_id = self.package_mask.package();
            let rp = Library::new(package);

            self.resources.push_package(package_id, rp);

            if self.tables.contains_key(&Origin::Global) {
                if let Some(st) = self.tables.remove(&Origin::Global) {
                    let set_result = self.resources.mut_package(package_id).and_then(|package| {
                        package.set_string_table(st, Origin::Global);

                        Some(())
                    });

                    if set_result.is_none() {
                        error!(
                            "could not set the string table because it refers to a non-existing \
                             package"
                        );
                    }
                }
            }
        }
    }

    fn visit_table_type(&mut self, table_type: TableTypeWrapper<'a>) {
        let mut entries = Entries::new();

        if let Some(ts) = &self.current_spec {
            let mask = ts
                .id()
                .and_then(|id| Ok(self.package_mask | (u32::from(id) << 16)))
                .unwrap_or(0);

            let entries_result = table_type.entries();

            match entries_result {
                Ok(ventries) => {
                    for e in &ventries {
                        let id = mask | e.id();

                        if !e.is_empty() {
                            entries.insert(id, e.clone());
                        }
                    }
                }
                Err(err) => error!("Error visiting table_type: {}", err),
            }
        }

        let package_id = self.package_mask.package();

        self.resources.mut_package(package_id).and_then(|package| {
            package.add_entries(entries);
            Some(())
        });
    }

    fn visit_type_spec(&mut self, type_spec: TypeSpecWrapper<'a>) {
        self.current_spec = Some(type_spec.clone());
        let package_id = (self.package_mask >> 24) as u8;
        if let Some(package) = self.resources.mut_package(package_id) {
            let _ = package
                .add_type_spec(type_spec)
                .map_err(|e| error!("Could not add type spec: {}", e));
        } else {
            error!("Type spec refers to a non existing package");
        }
    }
}

pub type RefPackage<'a> = Rc<RefCell<Library<'a>>>;

#[derive(Default, Debug)]
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

    fn package(&self, package_id: u8) -> Option<&Self::Library> {
        self.packages.get(&package_id)
    }

    fn mut_package(&mut self, package_id: u8) -> Option<&mut Self::Library> {
        self.packages.get_mut(&package_id)
    }

    fn main_package(&self) -> Option<&Self::Library> {
        match self.main_package {
            Some(package_id) => match self.packages.get(&package_id) {
                Some(package) => Some(package),
                None => None,
            },
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

#[derive(Debug)]
pub struct Library<'a> {
    package: PackageWrapper<'a>,
    specs: HashMap<u32, TypeSpecWrapper<'a>>,
    string_table: Option<StringTableCache<StringTableWrapper<'a>>>,
    spec_string_table: Option<StringTableCache<StringTableWrapper<'a>>>,
    entries_string_table: Option<StringTableCache<StringTableWrapper<'a>>>,
    entries: Entries,
}

impl<'a> Library<'a> {
    pub fn new(package: PackageWrapper<'a>) -> Self {
        Self {
            package,
            specs: HashMap::new(),
            string_table: None,
            spec_string_table: None,
            entries_string_table: None,
            entries: Entries::default(),
        }
    }

    fn spec_as_str(&self, spec_id: u32) -> Result<String> {
        if self.specs.get(&(spec_id)).is_some() {
            if let Some(spec_string_table) = &self.spec_string_table {
                if let Ok(spec_str) = spec_string_table.get_string(spec_id - 1) {
                    return Ok((*spec_str).clone());
                }
            }
        }

        bail!("could not retrieve spec as string")
    }
}

impl<'a> LibraryTrait for Library<'a> {
    fn name(&self) -> Option<String> {
        self.package.name().ok()
    }

    fn format_reference(
        &self,
        id: u32,
        key: u32,
        namespace: Option<String>,
        prefix: &str,
    ) -> Result<String> {
        let spec_id = u32::from(id.spec());
        let spec_str = self
            .spec_as_str(spec_id)
            .with_context(|| format!("could not find spec: {}", spec_id))?;
        let string = self
            .entries_string(key)
            .with_context(|| format!("could not find key {} on entries string table", key))?;

        let ending = if spec_str == "attr" {
            string
        } else {
            Rc::new(format!("{}/{}", spec_str, string))
        };

        match namespace {
            Some(ns) => Ok(format!("{}{}:{}", prefix, ns, ending)),
            None => Ok(format!("{}{}", prefix, ending)),
        }
    }

    fn entry(&self, id: u32) -> Result<&Entry> {
        self.entries
            .get(&id)
            .ok_or_else(|| anyhow!("could not find entry"))
    }

    fn entries_string(&self, str_id: u32) -> Result<Rc<String>> {
        if let Some(string_table) = &self.entries_string_table {
            let out_string = string_table.get_string(str_id).with_context(|| {
                format!("could not find string {} on entries string table", str_id)
            })?;

            return Ok(out_string);
        }

        bail!("string not found on entries string table")
    }

    fn spec_string(&self, str_id: u32) -> Result<Rc<String>> {
        if let Some(string_table) = &self.spec_string_table {
            let out_string = string_table.get_string(str_id).with_context(|| {
                format!("could not find string {} on spec string table", str_id)
            })?;

            return Ok(out_string);
        }

        bail!("string not found on spec string table")
    }
}

impl<'a> LibraryBuilder<'a> for Library<'a> {
    type StringTable = StringTableCache<StringTableWrapper<'a>>;
    type TypeSpec = TypeSpecWrapper<'a>;

    fn set_string_table(&mut self, string_table: Self::StringTable, origin: Origin) {
        match origin {
            Origin::Global => self.string_table = Some(string_table),
            Origin::Spec => self.spec_string_table = Some(string_table),
            Origin::Entries => self.entries_string_table = Some(string_table),
        }
    }

    fn add_entries(&mut self, entries: Entries) {
        self.entries.extend(entries);
    }

    fn add_type_spec(&mut self, type_spec: Self::TypeSpec) -> Result<()> {
        let id = u32::from(type_spec.id()?);
        self.specs.insert(id, type_spec);

        Ok(())
    }
}
