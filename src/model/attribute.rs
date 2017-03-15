use std::rc::Rc;
use model::Value;
use model::Identifier;
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
               name_index: u32)
               -> Self {
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

    pub fn resolve_flags<'a, R: ResourcesTrait<'a>>(&self,
                                                    flags: u32,
                                                    xml_resources: &[u32],
                                                    resources: &R)
                                                    -> Option<String> {
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

    pub fn resolve_reference<'a, R: ResourcesTrait<'a>>(&self,
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

    fn search_values<'a, R: ResourcesTrait<'a>>(&self,
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

#[cfg(test)]
mod tests {
    use super::*;
    use model::Value;
    use model::{StringTable, Resources, Library, LibraryBuilder};
    use model::Entries;
    use model::owned::{Entry, SimpleEntry, ComplexEntry};
    use visitor::Origin;
    use std::rc::Rc;
    use model::TypeSpec;
    use test::FakeStringTable;

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

            FakeLibrary { entries: entries }
        }
    }

    impl Library for FakeLibrary {
        fn get_name(&self) -> Option<String> {
            Some("Package name".to_string())
        }

        fn format_reference(&self,
                            id: u32,
                            _: u32,
                            namespace: Option<String>,
                            _: &str)
                            -> Result<String> {
            if id == (1 << 24) | 1 && namespace.is_none() {
                Ok("reference#1".to_string())
            } else if id == (2 << 24) | 1 && namespace.is_some() {
                Ok("NS:reference#2".to_string())
            } else {
                Err("Could not format".into())
            }
        }

        fn get_entry(&self, id: u32) -> Result<&Entry> {
            self.entries.get(&id).ok_or_else(|| "Could not find entry".into())
        }

        fn get_entries_string(&self, str_id: u32) -> Result<String> {
            let st = FakeStringTable;

            Ok((*st.get_string(str_id)?).clone())
        }

        fn get_spec_string(&self, _: u32) -> Result<String> {
            Err("Sepc string".into())
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
        fn get_id(&self) -> Result<u16> {
            Ok(82)
        }
        fn get_amount(&self) -> Result<u32> {
            Ok(3)
        }

        fn get_flag(&self, index: u32) -> Result<u32> {
            let flags = vec![0, 4, 16];

            flags.get(index as usize).map(|x| *x).ok_or("Flag out of bounds".into())
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
        let string_table = FakeStringTable;
        let value = Value::new(0x01, 0 as u32, &string_table).unwrap();
        let a = Attribute::new(Rc::new("attribute".to_string()), value, None, None, 1234);
        let resources = FakeResources::fake();

        let reference = a.resolve_reference(0, &resources, "prefix");

        assert_eq!("@null", reference.unwrap());
    }

    #[test]
    fn it_returns_error_if_the_provided_id_is_related_to_a_non_existing_package() {
        let string_table = FakeStringTable;
        let value = Value::new(0x01, 0 as u32, &string_table).unwrap();
        let a = Attribute::new(Rc::new("attribute".to_string()), value, None, None, 1234);
        let resources = FakeResources::fake();

        let reference = a.resolve_reference(3 << 24, &resources, "prefix");

        assert!(reference.is_err());
        assert_eq!("Package not found", reference.err().unwrap().to_string());
    }

    #[test]
    fn it_resolves_a_reference_without_namespace() {
        let string_table = FakeStringTable;
        let value = Value::new(0x01, 1 as u32, &string_table).unwrap();
        let a = Attribute::new(Rc::new("attribute".to_string()), value, None, None, 1234);
        let resources = FakeResources::fake();
        let reference = (1 << 24) | 1;

        let result = a.resolve_reference(reference, &resources, "prefix");

        assert_eq!("reference#1", result.unwrap());
    }

    #[test]
    fn it_resolves_a_reference_with_namespace() {
        let string_table = FakeStringTable;
        let value = Value::new(0x01, 1 as u32, &string_table).unwrap();
        let a = Attribute::new(Rc::new("attribute".to_string()), value, None, None, 1234);
        let resources = FakeResources::fake();
        let reference = (2 << 24) | 1;

        let result = a.resolve_reference(reference, &resources, "prefix");

        assert_eq!("NS:reference#2", result.unwrap());
    }

    #[test]
    fn it_resolves_flags_if_index_out_of_bounds() {
        let string_table = FakeStringTable;
        let value = Value::new(0x01, 1 as u32, &string_table).unwrap();
        let a = Attribute::new(Rc::new("attribute".to_string()), value, None, None, 1234);
        let resources = FakeResources::fake();

        let default_flags = format!("@flags:{}", 567);
        let resc = vec![];
        let result = a.resolve_flags(567, &resc, &resources);

        assert_eq!(default_flags, result.unwrap());
    }

    #[test]
    fn it_resolves_flags_if_in_resources() {
        let string_table = FakeStringTable;
        let value = Value::new(0x01, 11 as u32, &string_table).unwrap();
        let a = Attribute::new(Rc::new("attribute".to_string()), value, None, None, 0);
        let resources = FakeResources::fake();

        let resc = vec![2 << 24 | 3];
        let flags = 1 << 8;

        let result = a.resolve_flags(flags, &resc, &resources);

        assert_eq!("left", result.unwrap());
    }

    #[test]
    fn it_resolves_flags_if_in_resources_multiple() {
        let string_table = FakeStringTable;
        let value = Value::new(0x01, 1 as u32, &string_table).unwrap();
        let a = Attribute::new(Rc::new("attribute".to_string()), value, None, None, 0);
        let resources = FakeResources::fake();

        let resc = vec![2 << 24 | 3];
        let flags = 1 << 8 | 1 << 9;

        let result = a.resolve_flags(flags, &resc, &resources);

        assert_eq!("left|right", result.unwrap());
    }
}
