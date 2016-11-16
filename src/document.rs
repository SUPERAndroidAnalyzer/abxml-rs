use std::collections::BTreeMap;

#[derive(Default, Debug)]
pub struct Document {
    pub header: Header,
    pub header_string_table: HeaderStringTable,
    pub header_resource_table: HeaderResourceTable,
    pub header_namespace: HeaderNamespace,

    pub string_table: StringTable,
    pub resource_table: ResourceTable,
    pub resources: BTreeMap<String, String>,
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
    pub strings: Vec<String>,
    pub styles: Vec<String>,
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
    tag: String,
    attrs: Vec<Attribute>,
}

impl Element {
    pub fn new(tag: String, attrs: Vec<Attribute>) -> Self {
        Element {
            tag: tag,
            attrs: attrs,
        }
    }
}

#[derive(Debug)]
pub enum Value {
    String(String),
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
    Unknown
}

#[derive(Debug)]
pub struct Attribute {
    name: String,
    namespace: Option<String>,
    prefix: Option<String>,
    value: Value,
}

impl Attribute {
    pub fn new(name: String, value: Value, namespace: Option<String>, prefix: Option<String>) -> Self {
        Attribute {
            name: name,
            namespace: namespace,
            prefix: prefix,
            value: value,
        }
    }
}
