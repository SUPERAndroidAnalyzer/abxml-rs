use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

use failure::{Error, ResultExt};

use chunks::*;
use encoder::Xml;
use model::owned::SimpleEntry;
use model::{AttributeTrait, Element, ElementContainer, Identifier, Library, NamespaceStart,
            Namespaces, Resources as ResourceTrait, StringTable, Tag, TagStart, Value};
use visitor::model::Resources;

use super::ChunkVisitor;
use super::Origin;

#[derive(Debug)]
pub struct XmlVisitor<'a> {
    main_string_table: Option<StringTableCache<StringTableWrapper<'a>>>,
    namespaces: Namespaces,
    container: ElementContainer,
    res: Vec<u32>,
    resources: &'a Resources<'a>,
    namespace_prefixes: Vec<Rc<String>>,
}

impl<'a> XmlVisitor<'a> {
    pub fn new(resources: &'a Resources<'a>) -> Self {
        Self {
            main_string_table: None,
            namespaces: Namespaces::default(),
            container: ElementContainer::default(),
            res: Vec::new(),
            resources,
            namespace_prefixes: Vec::new(),
        }
    }

    pub fn get_namespaces(&self) -> &Namespaces {
        &self.namespaces
    }

    pub fn get_root(&self) -> &Option<Element> {
        self.container.get_root()
    }

    pub fn get_string_table(&self) -> &Option<StringTableCache<StringTableWrapper<'a>>> {
        &self.main_string_table
    }

    pub fn get_resources(&self) -> &Vec<u32> {
        &self.res
    }

    pub fn arsc(&self) -> &Resources {
        self.resources
    }

    // TODO: Convert to TryInto once it will be stable
    pub fn into_string(self) -> Result<String, Error> {
        if let Some(root) = self.get_root() {
            if self.get_string_table().is_some() {
                let res =
                    Xml::encode(self.get_namespaces(), root).context("could note encode XML")?;
                return Ok(res);
            } else {
                warn!("No string table found");
            }
        } else {
            warn!("No root on target XML");
        }

        Err(format_err!("could not decode XML"))
    }

    fn build_element(&self, tag_start: &XmlTagStartWrapper) -> Result<Element, Error> {
        match &self.main_string_table {
            Some(string_table) => {
                let (tag, attributes) = self.get_element_data(&string_table, tag_start)
                    .context("could not get element data")?;
                Ok(Element::new(tag, attributes))
            }
            None => Err(format_err!("no main string table found")),
        }
    }

    fn get_element_data(
        &self,
        string_table: &StringTableCache<StringTableWrapper<'a>>,
        tag_start: &XmlTagStartWrapper,
    ) -> Result<(Tag, HashMap<String, String>), Error> {
        let name_index = tag_start
            .get_element_name_index()
            .context("name index not found")?;
        let rc_string = string_table
            .get_string(name_index)
            .context("element name is not on the string table")?;
        let tag = Tag::new(rc_string.clone(), self.namespace_prefixes.clone());

        let mut attributes = HashMap::new();
        let num_attributes = tag_start
            .get_attributes_amount()
            .context("could not get the amount of attributes")?;

        for i in 0..num_attributes {
            let mut final_name = String::new();
            let current_attribute = tag_start
                .get_attribute(i)
                .context(format_err!("could not read attribute {} ", i))?;

            let namespace_index = current_attribute.get_namespace()?;
            if namespace_index != 0xFFFFFFFF {
                let namespace = (*string_table.get_string(namespace_index)?).clone();
                let prefix = self.namespaces
                    .get(&namespace)
                    .ok_or_else(|| format_err!("namespace not found"))?;
                final_name.push_str(prefix);
                final_name.push(':');
            }

            let name_index = current_attribute.get_name()?;
            let name = string_table.get_string(name_index)?;
            final_name.push_str((*name).as_str());

            let current_value = current_attribute.get_value()?;
            let value = match current_value {
                Value::StringReference(index) => (*string_table.get_string(index)?).clone(),
                Value::ReferenceId(id) => {
                    AttributeHelper::resolve_reference(self.resources, id, "@")
                        .context("could not resolve reference")?
                }
                Value::AttributeReferenceId(id) => {
                    AttributeHelper::resolve_reference(self.resources, id, "?")
                        .context("could not resolve attribute reference")?
                }
                Value::Integer(value) | Value::Flags(value) => {
                    let flag_resolution = AttributeHelper::resolve_flags(
                        &current_attribute,
                        value,
                        &self.res,
                        self.resources,
                    );

                    if flag_resolution.is_none() {
                        current_attribute.get_value()?.to_string()
                    } else {
                        flag_resolution.unwrap()
                    }
                }
                _ => current_value.to_string(),
            };

            attributes.insert(final_name, value);
        }

        Ok((tag, attributes))
    }
}

impl<'a> ChunkVisitor<'a> for XmlVisitor<'a> {
    fn visit_string_table(&mut self, string_table: StringTableWrapper<'a>, _: Origin) {
        if self.main_string_table.is_some() {
            error!("Secondary table!");
        } else {
            self.main_string_table = Some(StringTableCache::new(string_table));
        }
    }

    fn visit_xml_namespace_start(&mut self, namespace_start: XmlNamespaceStartWrapper<'a>) {
        if let Some(ref mut string_table) = self.main_string_table {
            if let (Ok(namespace), Ok(prefix)) = (
                namespace_start.get_namespace(string_table),
                namespace_start.get_prefix(string_table),
            ) {
                self.namespaces
                    .insert((*namespace).clone(), (*prefix).clone());
                self.namespace_prefixes.push(namespace.clone());
            } else {
                error!("Error reading namespace from the string table");
            }
        }
    }

    fn visit_xml_tag_start(&mut self, tag_start: XmlTagStartWrapper<'a>) {
        let element_result = self.build_element(&tag_start);
        if let Ok(element) = element_result {
            self.container.start_element(element);
        } else {
            error!("Could not build a XML element")
        }
    }

    fn visit_xml_tag_end(&mut self, _: XmlTagEndWrapper<'a>) {
        self.container.end_element()
    }

    fn visit_xml_namespace_end(&mut self, _: XmlNamespaceEndWrapper<'a>) {
        let _ = self.namespace_prefixes.pop();
    }

    fn visit_resource(&mut self, resource: ResourceWrapper<'a>) {
        if let Ok(res) = resource.get_resources() {
            self.res = res;
        }
    }
}

pub struct AttributeHelper;

impl AttributeHelper {
    pub fn resolve_reference<'a, R: ResourceTrait<'a>>(
        resources: &R,
        id: u32,
        prefix: &str,
    ) -> Result<String, Error> {
        let res_id = id;
        let package_id = id.get_package();

        if id == 0 {
            return Ok("@null".to_string());
        }

        let is_main = resources.is_main_package(package_id);
        let package = resources
            .get_package(package_id)
            .ok_or_else(|| format_err!("package not found"))?;

        let entry_key = package.get_entry(res_id).and_then(|e| Ok(e.get_key())).ok();

        if let Some(key) = entry_key {
            let namespace = if is_main { None } else { package.get_name() };
            return package.format_reference(id, key, namespace, prefix);
        }

        Err(format_err!("error resolving reference"))
    }

    pub fn resolve_flags<'a, R: ResourceTrait<'a>, A: AttributeTrait>(
        attribute: &A,
        flags: u32,
        xml_resources: &[u32],
        resources: &R,
    ) -> Option<String> {
        // Check if it's the special value in which the integer is an Enum
        // In that case, we return a crafted string instead of the integer itself
        let name_index = attribute.get_name().unwrap();
        if name_index < xml_resources.len() as u32 {
            Self::search_values(flags, name_index, xml_resources, resources)
        } else {
            let str = format!("@flags:{}", flags);

            Some(str.to_string())
        }
    }

    fn search_values<'a, R: ResourceTrait<'a>>(
        flags: u32,
        name_index: u32,
        xml_resources: &[u32],
        resources: &R,
    ) -> Option<String> {
        let entry_ref = match xml_resources.get(name_index as usize) {
            Some(entry_ref) => entry_ref,
            None => return None,
        };

        let package_id = entry_ref.get_package();
        resources
            .get_package(package_id)
            .and_then(|package| Self::search_flags(flags, *entry_ref, package))
    }

    fn search_flags(flags: u32, entry_ref: u32, package: &Library) -> Option<String> {
        let str_indexes = Self::get_strings(flags, entry_ref, package);
        let str_strs: Vec<String> = str_indexes
            .iter()
            .map(|si| {
                if let Ok(str) = package.get_entries_string(*si) {
                    (*str).clone()
                } else {
                    error!("Key not found on the string table");
                    String::new()
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

    fn get_strings(flags: u32, entry_ref: u32, package: &Library) -> Vec<u32> {
        let mut strs = Vec::new();
        let mut masks = Vec::new();

        let inner_entries = package
            .get_entry(entry_ref)
            .and_then(|e| e.complex())
            .and_then(|c| Ok(c.get_entries().to_vec()))
            .unwrap_or_else(|_| Vec::new());

        let mut sorted = inner_entries.to_vec();

        sorted.sort_by(Self::compare_entries);

        for ie in sorted {
            let mask = ie.get_value();
            if (mask & flags) == mask {
                let maybe_entry = package.get_entry(ie.get_id());

                if let Ok(entry) = maybe_entry {
                    let mut has_to_add = true;

                    for s in &masks {
                        if mask & s == mask {
                            has_to_add = false;
                            break;
                        }
                    }

                    if has_to_add {
                        entry
                            .simple()
                            .and_then(|s| Ok(s.get_key()))
                            .and_then(|key| {
                                strs.push(key);
                                masks.push(mask);
                                Ok(())
                            })
                            .unwrap_or_else(|_| {
                                error!(
                                    "Value should be added but there was an issue reading \
                                     the entry"
                                );
                            });
                    }
                } else {
                    info!("Some entry matched but could not found on entries");
                }
            }
        }

        strs
    }

    fn compare_entries(a: &SimpleEntry, b: &SimpleEntry) -> Ordering {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use failure::Error;
    use model::Entries;
    use model::TypeSpec;
    use model::owned::{AttributeBuf, ComplexEntry, Entry, SimpleEntry};
    use model::{Library, LibraryBuilder, Resources, StringTable};
    use test::FakeStringTable;
    use visitor::Origin;

    struct FakeLibrary {
        entries: Entries,
    }

    impl FakeLibrary {
        pub fn new() -> Self {
            let simple_entry1 = SimpleEntry::new(1, 1, 1, 1);
            let entry1 = Entry::Simple(simple_entry1);

            let simple_entry2 = SimpleEntry::new(1, 1, 1, 1);
            let entry2 = Entry::Simple(simple_entry2);

            let simple_entry3 = SimpleEntry::new((2 << 24) | 4, 1, 1, 1 << 8);
            let entry3 = Entry::Simple(simple_entry3.clone());

            let simple_entry4 = SimpleEntry::new((2 << 24) | 4, 456, 1, 1 << 8);
            let entry4 = Entry::Simple(simple_entry4);

            let simple_entry5 = SimpleEntry::new((2 << 24) | 5, 789, 1, 1 << 9);
            let entry5 = Entry::Simple(simple_entry5.clone());

            let simple_entry6 = SimpleEntry::new((2 << 24) | 6, 123, 1, 1 << 10);
            let entry6 = Entry::Simple(simple_entry6.clone());

            let mut ce1_childen_entries = Vec::new();
            ce1_childen_entries.push(simple_entry3);
            ce1_childen_entries.push(simple_entry5);
            ce1_childen_entries.push(simple_entry6);

            let complex_entry1 = ComplexEntry::new(1, 1, 1, ce1_childen_entries);
            let entry_ce1 = Entry::Complex(complex_entry1);

            let mut entries = Entries::new();
            entries.insert((1 << 24) | 1, entry1);
            entries.insert((2 << 24) | 1, entry2);
            entries.insert((2 << 24) | 2, entry3);
            entries.insert((2 << 24) | 3, entry_ce1);
            entries.insert((2 << 24) | 4, entry4);
            entries.insert((2 << 24) | 5, entry5);
            entries.insert((2 << 24) | 6, entry6);

            Self { entries }
        }
    }

    impl Library for FakeLibrary {
        fn get_name(&self) -> Option<String> {
            Some("Package name".to_string())
        }

        fn format_reference(
            &self,
            id: u32,
            _: u32,
            namespace: Option<String>,
            _: &str,
        ) -> Result<String, Error> {
            if id == (1 << 24) | 1 && namespace.is_none() {
                Ok("reference#1".to_string())
            } else if id == (2 << 24) | 1 && namespace.is_some() {
                Ok("NS:reference#2".to_string())
            } else {
                Err(format_err!("could not format"))
            }
        }

        fn get_entry(&self, id: u32) -> Result<&Entry, Error> {
            self.entries
                .get(&id)
                .ok_or_else(|| format_err!("could not find entry"))
        }

        fn get_entries_string(&self, str_id: u32) -> Result<Rc<String>, Error> {
            let st = FakeStringTable;

            Ok(st.get_string(str_id)?)
        }

        fn get_spec_string(&self, _: u32) -> Result<Rc<String>, Error> {
            bail!("spec string")
        }
    }

    impl<'a> LibraryBuilder<'a> for FakeLibrary {
        type StringTable = FakeStringTable;
        type TypeSpec = FakeTypeSpec;

        fn set_string_table(&mut self, _: Self::StringTable, _: Origin) {}

        fn add_entries(&mut self, _: Entries) {}

        fn add_type_spec(&mut self, _: Self::TypeSpec) {}
    }

    struct FakeTypeSpec;

    impl TypeSpec for FakeTypeSpec {
        fn get_id(&self) -> Result<u16, Error> {
            Ok(82)
        }
        fn get_amount(&self) -> Result<u32, Error> {
            Ok(3)
        }

        fn get_flag(&self, index: u32) -> Result<u32, Error> {
            let flags = vec![0, 4, 16];

            flags
                .get(index as usize)
                .map(|x| *x)
                .ok_or_else(|| format_err!("flag out of bounds"))
        }
    }

    struct FakeResources {
        library: FakeLibrary,
    }

    impl FakeResources {
        pub fn fake() -> Self {
            let library = FakeLibrary::new();

            FakeResources { library: library }
        }
    }

    impl<'a> Resources<'a> for FakeResources {
        type Library = FakeLibrary;

        fn get_package(&self, package_id: u8) -> Option<&Self::Library> {
            if package_id == 1 || package_id == 2 {
                Some(&self.library)
            } else {
                None
            }
        }

        fn get_mut_package(&mut self, _: u8) -> Option<&mut Self::Library> {
            None
        }

        fn get_main_package(&self) -> Option<&Self::Library> {
            None
        }

        fn is_main_package(&self, package_id: u8) -> bool {
            package_id == 1
        }
    }

    #[test]
    fn it_resolves_to_null_if_id_is_0() {
        let resources = FakeResources::fake();

        let reference = AttributeHelper::resolve_reference(&resources, 0, "prefix");

        assert_eq!("@null", reference.unwrap());
    }

    #[test]
    fn it_returns_error_if_the_provided_id_is_related_to_a_non_existing_package() {
        let resources = FakeResources::fake();

        let reference = AttributeHelper::resolve_reference(&resources, 3 << 24, "prefix");

        assert!(reference.is_err());
        assert_eq!("package not found", reference.err().unwrap().to_string());
    }

    #[test]
    fn it_resolves_a_reference_without_namespace() {
        let resources = FakeResources::fake();

        let reference = AttributeHelper::resolve_reference(&resources, (1 << 24) | 1, "prefix");

        assert_eq!("reference#1", reference.unwrap());
    }

    #[test]
    fn it_resolves_a_reference_with_namespace() {
        let resources = FakeResources::fake();

        let result = AttributeHelper::resolve_reference(&resources, (2 << 24) | 1, "prefix");

        assert_eq!("NS:reference#2", result.unwrap());
    }

    #[test]
    fn it_resolves_flags_if_index_out_of_bounds() {
        let attribute = AttributeBuf::new(0, 1, 0, 0, 0);
        let resources = FakeResources::fake();
        let default_flags = format!("@flags:{}", 567);
        let resc = vec![];

        let result = AttributeHelper::resolve_flags(&attribute, 567, &resc, &resources);

        assert_eq!(default_flags, result.unwrap());
    }

    #[test]
    fn it_resolves_flags_if_in_resources() {
        let attribute = AttributeBuf::new(0, 0, 0, 0x1 << 24, 11);
        let resources = FakeResources::fake();

        let resc = vec![2 << 24 | 3];
        let flags = 1 << 8;

        let result = AttributeHelper::resolve_flags(&attribute, flags, &resc, &resources);

        assert_eq!("left", result.unwrap());
    }

    #[test]
    fn it_resolves_flags_if_in_resources_multiple() {
        let resources = FakeResources::fake();
        let attribute = AttributeBuf::new(0, 0, 0, 1, 1);

        let resc = vec![2 << 24 | 3];
        let flags = 1 << 8 | 1 << 9;

        let result = AttributeHelper::resolve_flags(&attribute, flags, &resc, &resources);

        assert_eq!("left|right", result.unwrap());
    }
}
