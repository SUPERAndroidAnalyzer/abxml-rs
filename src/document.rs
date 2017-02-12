
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Display, Formatter};
use std::fmt::Error as FmtError;
use std::rc::Rc;
use std::ops::Deref;
use errors::*;
use std::result::Result as StdResult;
use chunks::*;
use std::mem;

pub type Namespaces = BTreeMap<Rc<String>, Rc<String>>;
pub type Entries = HashMap<u32, Entry>;

#[derive(Default, Debug)]
pub struct Document {
    pub header: Header,
    pub header_string_table: HeaderStringTable,
    pub header_resource_table: HeaderResourceTable,
    pub header_namespace: HeaderNamespace,

    // pub string_table: StringTable,
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
    pub flags: u32,
    pub string_offset: u32,
    pub style_offset: u32,
}

/*
#[derive(Default, Debug, Clone)]
pub struct StringTable {
    pub strings: Vec<Rc<String>>,
    pub styles: Vec<Rc<String>>,
}

impl StringTable {
    pub fn get_string(&self, i: usize) -> Option<Rc<String>> {
        if i < self.strings.len() {
            Some(self.strings[i].clone())
        } else {
            None
        }
    }
}
*/
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
    fn fmt(&self, formatter: &mut Formatter) -> StdResult<(), FmtError> {
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
    Float(f32),
    Integer(u64),
    Flags(u64),
    Boolean(bool),
    Color(String),
    Color2(String),
    ReferenceId(u32),
    AttributeReferenceId(u32),
    Unknown,
}

const TOKEN_TYPE_REFERENCE_ID: u8 = 0x01;
const TOKEN_TYPE_ATTRIBUTE_REFERENCE_ID: u8 = 0x02;
const TOKEN_TYPE_STRING: u8 = 0x03;
const TOKEN_TYPE_FLOAT: u8 = 0x04;
const TOKEN_TYPE_DIMENSION: u8 = 0x05;
const TOKEN_TYPE_FRACTION: u8 = 0x06;
const TOKEN_TYPE_DYN_REFERENCE: u8 = 0x07;
const TOKEN_TYPE_DYN_ATTRIBUTE: u8 = 0x08;
const TOKEN_TYPE_INTEGER: u8 = 0x10;
const TOKEN_TYPE_FLAGS: u8 = 0x11;
const TOKEN_TYPE_BOOLEAN: u8 = 0x12;
const TOKEN_TYPE_COLOR: u8 = 0x1C; // ARGB8
const TOKEN_TYPE_COLOR2: u8 = 0x1D; // RGB8

impl Value {
    pub fn to_string(&self) -> String {
        match *self {
            Value::String(ref s) => s.deref().clone(),
            Value::Dimension(ref s) | Value::Fraction(ref s) |
            Value::Color(ref s) | Value::Color2(ref s) => s.clone(),
            Value::Float(f) => {
                format!("{:.*}", 1, f)
            },
            Value::Integer(i) | Value::Flags(i) => i.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::ReferenceId(ref s) => {
                format!("@id/0x{:#8}", s)
            },
            Value::AttributeReferenceId(ref s) => {
                format!("@id/0x{:#8}", s)
            },
            _ => "Unknown".to_string(),
        }
    }

    pub fn new(value_type: u8, data: u32, str_table: &mut StringTable) -> Result<Self> {
        let value = match value_type {
            TOKEN_TYPE_REFERENCE_ID | TOKEN_TYPE_DYN_REFERENCE => {
                Value::ReferenceId(data)
            },
            TOKEN_TYPE_ATTRIBUTE_REFERENCE_ID | TOKEN_TYPE_DYN_ATTRIBUTE => {
                Value::AttributeReferenceId(data)
            }
            TOKEN_TYPE_STRING => {
                let string = str_table.get_string(data)?;

                Value::String(string.clone())
            }
            TOKEN_TYPE_DIMENSION => {
                let units: [&str; 6] = ["px", "dip", "sp", "pt", "in", "mm"];
                let size = (data >> 8) as f32;
                let unit_idx = data & 0xF;

                match units.get(unit_idx as usize) {
                    Some(unit) => {
                        let formatted = format!("{:.*}{}", 1, size, unit);
                        Value::Dimension(formatted)
                    },
                    None => {
                        return Err(format!("Expected a valid unit index. Got: {}",
                                unit_idx).into())
                    }
                }
            }
            TOKEN_TYPE_FRACTION => {
                let units: [&str; 2] = ["%", "%p"];
                let u = units[(data & 0xF) as usize];
                let value = Self::complex(data) * 100.0;
                // let value = unsafe {mem::transmute::<u32, f32>(data)};
                // let div = unsafe {mem::transmute::<u32, f32>(0x7FFFFFFF)};
                // let div = 100.0;

//                println!("Value: {} Div: {}", value, div);
                // let div: f32 = 1000.0;
                // let value = value * div;
                let formatted_fraction = format!("{:.*}{}", 6, value, u);

                Value::Fraction(formatted_fraction)
            }
            TOKEN_TYPE_INTEGER => Value::Integer(data as u64),
            TOKEN_TYPE_FLAGS => Value::Flags(data as u64),
            TOKEN_TYPE_FLOAT => {
                let f = unsafe { mem::transmute::<u32, f32>(data)};
                Value::Float(f)
            },
            TOKEN_TYPE_BOOLEAN => {
                if data > 0 {
                    Value::Boolean(true)
                } else {
                    Value::Boolean(false)
                }
            }
            TOKEN_TYPE_COLOR => {
                let formatted_color = format!("#{:#8}", data);
                Value::Color(formatted_color)
            }
            TOKEN_TYPE_COLOR2 => {
                let formatted_color = format!("#{:#8}", data);
                Value::Color2(formatted_color)
            }
            _ => Value::Unknown,

        };

        Ok(value)
    }

    fn complex(data: u32) -> f32 {
        // TODO: Clean this mess
        let mantissa = 0xffffff << 8;
        let m = (data & mantissa) as f32;
        let mm = 1.0 / ((1 << 8) as f32);
        let radix = [
            1.0 * mm,
            1.0 / ((1 << 7) as f32) * mm,
            1.0 / ((1 << 15) as f32) * mm,
            1.0 / ((1 << 23) as f32) * mm,
        ];

        let idx = (data >> 4) & 0x3;

        m * radix[idx as usize]
    }
}

#[derive(Debug)]
pub struct Attribute {
    name: Rc<String>,
    namespace: Option<Rc<String>>,
    prefix: Option<Rc<String>>,
    value: Value,
    name_index: u32,
    //resource_id: Option<u32>,
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
            //resource_id: None,
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
}

#[derive(Default)]
pub struct ElementContainer {
    stack: Vec<Element>,
    root: Option<Element>,
}

impl ElementContainer {
    pub fn start_element(&mut self, mut element: Element) {
        element.set_level(self.stack.len() as u32);
        self.stack.push(element);
    }

    pub fn end_element(&mut self) {
        let element = self.stack.pop().unwrap();

        if self.stack.is_empty() {
            self.root = Some(element);
        } else {
            // Append child to current element
            let last_element = self.stack.len();
            self.stack[last_element - 1].append(element);
        }
    }

    pub fn get_root(&self) -> &Option<Element> {
        &self.root
    }
}
