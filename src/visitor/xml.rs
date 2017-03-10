use chunks::*;
use model::{Element, ElementContainer, Namespaces};
use encoder::Xml;
use visitor::model::Resources;
use errors::*;

use super::ChunkVisitor;
use super::Origin;

pub struct XmlVisitor<'a> {
    main_string_table: Option<StringTable<'a>>,
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

    pub fn get_string_table(&self) -> &Option<StringTable> {
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
}

impl<'a> ChunkVisitor<'a> for XmlVisitor<'a> {
    fn visit_string_table(&mut self, string_table: StringTable<'a>, _: Origin) {
        match self.main_string_table {
            Some(_) => {
                error!("Secondary table!");
            }
            None => {
                self.main_string_table = Some(string_table);
            }
        }
    }

    fn visit_xml_namespace_start(&mut self, namespace_start: XmlNamespaceStart<'a>) {
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

    fn visit_xml_tag_start(&mut self, tag_start: XmlTagStart<'a>) {
        match self.main_string_table {
            Some(ref mut string_table) => {
                match tag_start.get_tag(&self.namespaces, string_table) {
                    Ok((attributes, element_name)) => {
                        let element = Element::new(element_name, attributes);
                        self.container.start_element(element);
                    }
                    Err(e) => {
                        println!("{}", string_table);
                        println!("Could not start tag: {:?}", e);
                        error!("Could not retrieve tag");
                    }
                }
            }
            None => {
                error!("No main string table found!");
            }
        }
    }

    fn visit_xml_tag_end(&mut self, _: XmlTagEnd<'a>) {
        self.container.end_element()
    }

    fn visit_resource(&mut self, resource: Resource<'a>) {
        if let Ok(res) = resource.get_resources() {
            self.res = res;
        }
    }
}
