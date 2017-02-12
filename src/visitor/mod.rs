use std::io::Cursor;
use chunks::*;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;
use document::{Namespaces, Element, ElementContainer, Entries};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

pub trait ChunkVisitor<'a> {
    fn visit_string_table(&mut self, _string_table: StringTable<'a>, _origin: Origin) {}
    fn visit_package(&mut self, _package: Package<'a>) {}
    fn visit_table_type(&mut self, _table_type: TableType<'a>) {}
    fn visit_type_spec(&mut self, _type_spec: TypeSpec<'a>) {}
    fn visit_xml_namespace_start(&mut self, _namespace_start: XmlNamespaceStart<'a>) {}
    fn visit_xml_namespace_end(&mut self, _namespace_end: XmlNamespaceEnd<'a>) {}
    fn visit_xml_tag_start(&mut self, _tag_start: XmlTagStart<'a>) {}
    fn visit_xml_tag_end(&mut self, _tag_end: XmlTagEnd<'a>) {}
    fn visit_xml_text(&mut self, _text: XmlText<'a>) {}
    fn visit_resource(&mut self, _resource: Resource<'a>) {}
}

pub struct Executor;

impl Executor {
    pub fn arsc<'a, V: ChunkVisitor<'a>>(mut cursor: Cursor<&'a [u8]>, mut visitor: &mut V) -> Result<()> {
        let _token = cursor.read_u16::<LittleEndian>()?;
        let _header_size = cursor.read_u16::<LittleEndian>()?;
        let _chunk_size = cursor.read_u32::<LittleEndian>()?;
        let _package_amount = cursor.read_u32::<LittleEndian>()?;

        let stream = ChunkLoaderStream::new(cursor);
        let mut origin = Origin::Global;

        for c in stream {
            match c? {
                Chunk::StringTable(stw) => {
                    let st = StringTable::new(stw);
                    visitor.visit_string_table(st, origin);
                    origin = Origin::next(origin);
                },
                Chunk::Package(pw) => {
                    let package = Package::new(pw);
                    visitor.visit_package(package);
                },
                Chunk::TableType(ttw) => {
                    let tt = TableType::new(ttw);
                    visitor.visit_table_type(tt);
                },
                Chunk::TableTypeSpec(tsw) => {
                    let ts = TypeSpec::new(tsw);
                    visitor.visit_type_spec(ts);
                },
                _ => (),
            }
        }

        Ok(())
    }

    pub fn xml<'a, V: ChunkVisitor<'a>>(mut cursor: Cursor<&'a [u8]>, mut visitor: &mut V) -> Result<()> {
        let _token = cursor.read_u16::<LittleEndian>()?;
        let _header_size = cursor.read_u16::<LittleEndian>()?;
        let _chunk_size = cursor.read_u32::<LittleEndian>()?;

        let stream = ChunkLoaderStream::new(cursor);
        let mut origin = Origin::Global;

        for c in stream {
            match c? {
                Chunk::StringTable(stw) => {
                    let st = StringTable::new(stw);
                    visitor.visit_string_table(st, origin);
                },
                Chunk::Package(pw) => {
                    let package = Package::new(pw);
                    visitor.visit_package(package);
                },
                Chunk::TableType(ttw) => {
                    origin = Origin::Entries;
                    let tt = TableType::new(ttw);
                    visitor.visit_table_type(tt);
                },
                Chunk::TableTypeSpec(tsw) => {
                    origin = Origin::Spec;
                    let ts = TypeSpec::new(tsw);
                    visitor.visit_type_spec(ts);
                },
                Chunk::XmlNamespaceStart(xnsw) => {
                    let ts = XmlNamespaceStart::new(xnsw);
                    visitor.visit_xml_namespace_start(ts);
                },
                Chunk::XmlNamespaceEnd(xnsw) => {
                    let ts = XmlNamespaceEnd::new(xnsw);
                    visitor.visit_xml_namespace_end(ts);
                },
                Chunk::XmlTagStart(xnsw) => {
                    let ts = XmlTagStart::new(xnsw);
                    visitor.visit_xml_tag_start(ts);
                },
                Chunk::XmlTagEnd(xnsw) => {
                    let ts = XmlTagEnd::new(xnsw);
                    visitor.visit_xml_tag_end(ts);
                },
                Chunk::XmlText(xsnw) => {
                    let ts = XmlText::new(xsnw);
                    visitor.visit_xml_text(ts);
                }
                Chunk::Resource(rw) => {
                    let ts = Resource::new(rw);
                    visitor.visit_resource(ts);
                }
                _ => (),
            }
        }

        Ok(())
    }
}

pub struct DummyVisitor;

impl<'a> ChunkVisitor<'a> for DummyVisitor {}

pub struct PrintVisitor;

impl<'a> ChunkVisitor<'a> for PrintVisitor {
    fn visit_string_table(&mut self, string_table: StringTable, origin: Origin) {
        println!("String Table!");
        println!("\tLength ({:?}): {} ", origin, string_table.get_strings_len());
    }

    fn visit_package(&mut self, package: Package) {
        println!("Package!");
        println!("\tId: {}", package.get_id());
        println!("\tName: {}", package.get_name().unwrap());
    }

    fn visit_table_type(&mut self, table_type: TableType) {
        println!("Table type!");
        println!("\tId: {}", table_type.get_id());
    }

    fn visit_type_spec(&mut self, type_spec: TypeSpec) {
        println!("Type spec!");
        println!("\tId: {}", type_spec.get_id());
    }
}

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
                                let entry = pb.entries.get(&entry_ref).unwrap();
                                let parent_entry_id = entry.get_referent_id(attr.get_data().unwrap()).unwrap();
                                let parent_entry = pb.entries.get(&parent_entry_id).unwrap();

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
                let package_id = (self.package_mask >> 24) as u8;
                let package = self.resources.get_package(package_id);
                let mut package_borrow = package.borrow_mut();

                package_borrow.set_string_table(string_table, origin);
            },
        }


        /*let package_id = (self.package_mask >> 24) as u8;
        let package = self.resources.get_package(package_id);
        let mut package_borrow = package.borrow_mut();

        package_borrow.set_string_table(string_table, origin);*/
    }

    fn visit_package(&mut self, package: Package<'a>) {
        self.package_mask = package.get_id() << 24;

        let package_id = (self.package_mask >> 24) as u8;
        let mut rp = ResourcesPackage::default();
        rp.add_package(package);
        self.resources.push_package(package_id, rp);

        if self.tables.contains_key(&Origin::Global) {
            let st = self.tables.remove(&Origin::Global).unwrap();
            let package_mut = self.resources.get_package(package_id);
            let mut package_borrow = package_mut.borrow_mut();
            package_borrow.set_string_table(st, Origin::Global);
        }
    }

    fn visit_table_type(&mut self, table_type: TableType<'a>) {
        if let Some(ref ts) = self.current_spec {
            let mask = self.package_mask |
                ((ts.get_id() as u32) << 16);
            let entries = table_type.get_entries(ts, mask).unwrap();

            let package_id = (self.package_mask >> 24) as u8;
            let package = self.resources.get_package(package_id);
            let mut package_borrow = package.borrow_mut();

            package_borrow.add_entries(entries);
        }
    }

    fn visit_type_spec(&mut self, type_spec: TypeSpec<'a>) {
        self.current_spec = Some(type_spec.clone());
        let package_id = (self.package_mask >> 24) as u8;
        let package = self.resources.get_package(package_id);
        let mut package_borrow = package.borrow_mut();

        package_borrow.add_type_spec(type_spec);
    }
}

type RefPackage<'a> = Rc<RefCell<ResourcesPackage<'a>>>;

#[derive(Default)]
pub struct Resources<'a> {
    packages: HashMap<u8, RefPackage<'a>>,
    main_package: Option<u8>,
}

impl<'a> Resources<'a> {
    pub fn push_package(&mut self, package_id: u8, package: ResourcesPackage<'a>) {
        if self.packages.is_empty() {
            self.main_package = Some(package_id);
        }

        self.packages.insert(package_id, Rc::new(RefCell::new(package)));
    }

    pub fn get_package(&self, package_id: u8) -> RefPackage<'a> {
        self.packages.get(&package_id).unwrap().clone()
    }

    pub fn get_main_package(&self) -> Option<RefPackage<'a>> {
        match self.main_package {
            Some(package_id) => {
                Some(self.packages.get(&package_id).unwrap().clone())
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
pub struct ResourcesPackage<'a> {
    package: Option<Package<'a>>,
    specs: Vec<TypeSpec<'a>>,
    string_table: Option<StringTable<'a>>,
    spec_string_table: Option<StringTable<'a>>,
    entries_string_table: Option<StringTable<'a>>,
    entries: Entries,
    resources: Vec<u32>,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Origin {
    Global,
    Spec,
    Entries,
}

impl Origin {
    pub fn next(origin: Origin) -> Origin {
        match origin {
            Origin::Global => Origin::Spec,
            Origin::Spec => Origin::Entries,
            Origin::Entries => Origin::Global,
        }
    }
}

impl<'a> ResourcesPackage<'a> {
    pub fn set_string_table(&mut self, string_table: StringTable<'a>, origin: Origin) {
        //println!("ST: {}", string_table);
        // println!("Setting table: {:?}", origin);
        match origin {
            Origin::Global => self.string_table = Some(string_table),
            Origin::Spec => self.spec_string_table = Some(string_table),
            Origin::Entries => self.entries_string_table = Some(string_table),
        }
    }

    pub fn add_package(&mut self, package: Package<'a>) {
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

    pub fn format_reference(&mut self, id: u32, key: u32, namespace: Option<String>) -> Option<String> {
        let spec_id = (id & 0x00FF0000) >> 16;
        let spec_str = self.get_spec_as_str(spec_id).unwrap();
        let string = self.get_entries_string(key).unwrap();

        match namespace {
            Some(ns) => Some(format!("@{}:{}/{}", ns, spec_str, string)),
            None => Some(format!("@{}/{}", spec_str, string)),
        }
    }

    pub fn get_entries(&self) -> &Entries {
        &self.entries
    }

    fn get_entries_string(&mut self, str_id: u32) -> Option<String> {
        if let Some(ref mut string_table) = self.entries_string_table {
            let out_string = string_table.get_string(str_id).unwrap();

            return Some((*out_string).clone())
        }

        None
    }

    fn get_spec_as_str(&mut self, spec_id: u32) -> Option<String>
    {
        if let Some(spec) = self.specs.get((spec_id - 1) as usize) {
            if let Some(ref mut spec_string_table) = self.spec_string_table {
                if let Ok(spec_str) = spec_string_table.get_string((spec.get_id() - 1) as u32) {
                    return Some((*spec_str).clone());
                }
            }
        }

        None
    }
}