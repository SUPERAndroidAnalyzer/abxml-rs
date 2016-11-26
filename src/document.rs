use std::collections::BTreeMap;
use std::fmt::{Display, Formatter, Error};
use std::rc::Rc;
use std::ops::Deref;

pub type Namespaces = BTreeMap<Rc<String>, Rc<String>>;
#[derive(Default, Debug)]
pub struct Document {
    pub header: Header,
    pub header_string_table: HeaderStringTable,
    pub header_resource_table: HeaderResourceTable,
    pub header_namespace: HeaderNamespace,

    pub string_table: StringTable,
    pub resource_table: ResourceTable,
    pub resources: Namespaces,
    pub root_element: Element,
}

#[derive(Default, Debug)]
pub struct Header {
    pub size: u32,
}

#[derive(Default, Debug)]
pub struct HeaderStringTable {
    pub chunk: u32,
    pub string_amount: u32,
    pub style_amount: u32,
    pub unknown: u32,
    pub string_offset: u32,
    pub style_offset: u32,
}

#[derive(Default, Debug)]
pub struct StringTable {
    pub strings: Vec<Rc<String>>,
    pub styles: Vec<Rc<String>>,
}

#[derive(Default, Debug)]
pub struct HeaderResourceTable {
    pub chunk: u32,
}

#[derive(Default, Debug)]
pub struct ResourceTable {
    pub resources: Vec<u32>,
}

#[derive(Default, Debug)]
pub struct HeaderNamespace {
    pub chunk: u32,
}

#[derive(Default, Debug)]
pub struct Element {
    tag: Rc<String>,
    attrs: Vec<Attribute>,
    children: Vec<Element>,
    level: u32,
}

impl Element {
    pub fn new(tag: Rc<String>, attrs: Vec<Attribute>) -> Self {
        Element {
            tag: tag,
            attrs: attrs,
            children: Vec::new(),
            level: 0,
        }
    }

    pub fn append(&mut self, element: Element) {
        self.children.push(element)
    }

    pub fn set_level(&mut self, level: u32) {
        self.level = level;
    }

    pub fn get_attributes(&self) -> &Vec<Attribute> {
        &self.attrs
    }

    pub fn get_tag(&self) -> Rc<String> {
        self.tag.clone()
    }

    pub fn get_children(&self) -> &Vec<Element> {
        &self.children
    }
}

impl Display for Element {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        let tabs = "\t".to_string().repeat(self.level as usize);
        write!(formatter, "{}Element: {}\n", tabs, self.tag)?;

        for c in &self.children {
            write!(formatter, "{}", c)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum Value {
    String(Rc<String>),
    Dimension(String),
    Fraction(String),
    Float(f64),
    Integer(u64),
    Flags(u64),
    Boolean(bool),
    Color(String),
    Color2(String),
    ReferenceId(String),
    AttributeReferenceId(String),
    Unknown,
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            &Value::String(ref s) => s.deref().clone(),
            &Value::Dimension(ref s) => s.clone(),
            &Value::Fraction(ref s) => s.clone(),
            &Value::Float(f) => f.to_string(),
            &Value::Integer(i) => i.to_string(),
            &Value::Flags(i) => i.to_string(),
            &Value::Boolean(b) => b.to_string(),
            &Value::Color(ref s) => s.clone(),
            &Value::Color2(ref s) => s.clone(),
            &Value::ReferenceId(ref s) => s.clone(),
            &Value::AttributeReferenceId(ref s) => s.clone(),
            _ => "Unknown".to_string(),
        }

    }
}

#[derive(Debug)]
pub struct Attribute {
    name: Rc<String>,
    namespace: Option<Rc<String>>,
    prefix: Option<Rc<String>>,
    value: Value,
}

impl Attribute {
    pub fn new(name: Rc<String>,
               value: Value,
               namespace: Option<Rc<String>>,
               prefix: Option<Rc<String>>)
               -> Self {
        Attribute {
            name: name,
            namespace: namespace,
            prefix: prefix,
            value: value,
        }
    }

    pub fn get_name(&self) -> Rc<String> {
        self.name.clone()
    }

    pub fn get_value(&self) -> String {
        self.value.to_string()
    }

    pub fn get_prefix(&self) -> Option<Rc<String>> {
        self.prefix.clone()
    }
}

pub struct ElementContainer {
    stack: Vec<Element>,
    root: Option<Element>,
}

impl ElementContainer {
    pub fn new() -> Self {
        ElementContainer {
            stack: Vec::new(),
            root: None,
        }
    }

    pub fn start_element(&mut self, mut element: Element) {
        element.set_level(self.stack.len() as u32);
        self.stack.push(element);
    }

    pub fn end_element(&mut self) {
        let element = self.stack.pop().unwrap();

        if self.stack.len() == 0 {
            self.root = Some(element);
        } else {
            // Append child to current element
            let last_element = self.stack.len();
            self.stack[last_element - 1].append(element);
        }
    }

    pub fn get_root(self) -> Option<Element> {
        self.root
    }
}
