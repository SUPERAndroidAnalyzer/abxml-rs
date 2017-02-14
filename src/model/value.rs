use std::rc::Rc;
use chunks::StringTable;
use std::mem;
use errors::*;
use std::ops::Deref;

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

impl Value {
    pub fn to_string(&self) -> String {
        match *self {
            Value::String(ref s) => s.deref().clone(),
            Value::Dimension(ref s) | Value::Fraction(ref s) |
            Value::Color(ref s) | Value::Color2(ref s) => {
                s.clone()
            },
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
                let value = Self::complex(data);
                let unit_idx = data & 0xF;

                match units.get(unit_idx as usize) {
                    Some(unit) => {
                        let formatted = format!("{:.*}{}", 1, value, unit);
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
                let final_value = Self::complex(data) * 100.0;

                let integer = final_value.round() as f32;
                let diff = final_value - integer;
                let formatted_fraction = if diff > 0.0000001 {
                    format!("{:.*}{}", 6, final_value, u)
                } else {
                    format!("{:.*}{}", 1, final_value, u)
                };

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
                let formatted_color = format!("#{:x}", data);
                Value::Color(formatted_color)
            }
            TOKEN_TYPE_COLOR2 => {
                let formatted_color = format!("#{:x}", data);
                Value::Color2(formatted_color)
            }
            _ => Value::Unknown,

        };

        Ok(value)
    }

    fn complex(data: u32) -> f32 {
        // TODO: Clean this mess
        let mantissa = 0xffffff << 8;
        let uvalue = (data & mantissa);
        let ivalue: i32 = unsafe {mem::transmute(uvalue)};
        let m = ivalue as f32;
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