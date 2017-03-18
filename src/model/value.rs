use std::rc::Rc;
use model::StringTable;
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
    StringReference(u32),
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
            Value::StringReference(i) => format!("@string/{}", i),
            Value::Dimension(ref s) |
            Value::Fraction(ref s) |
            Value::Color(ref s) |
            Value::Color2(ref s) => s.clone(),
            Value::Float(f) => format!("{:.*}", 1, f),
            Value::Integer(i) |
            Value::Flags(i) => i.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::ReferenceId(ref s) => format!("@id/0x{:x}", s),
            Value::AttributeReferenceId(ref s) => format!("@id/0x{:x}", s),
            _ => "Unknown".to_string(),
        }
    }

    pub fn new(value_type: u8, data: u32) -> Result<Self> {
        let value = match value_type {
            TOKEN_TYPE_REFERENCE_ID |
            TOKEN_TYPE_DYN_REFERENCE => Value::ReferenceId(data),
            TOKEN_TYPE_ATTRIBUTE_REFERENCE_ID |
            TOKEN_TYPE_DYN_ATTRIBUTE => Value::AttributeReferenceId(data),
            TOKEN_TYPE_STRING => Value::StringReference(data),
            TOKEN_TYPE_DIMENSION => {
                let units: [&str; 6] = ["px", "dip", "sp", "pt", "in", "mm"];
                let value = Self::complex(data);
                let unit_idx = data & 0xF;

                match units.get(unit_idx as usize) {
                    Some(unit) => {
                        let formatted = format!("{:.*}{}", 1, value, unit);
                        Value::Dimension(formatted)
                    }
                    None => {
                        return Err(format!("Expected a valid unit index. Got: {}", unit_idx).into())
                    }
                }
            }
            TOKEN_TYPE_FRACTION => {
                let units: [&str; 2] = ["%", "%p"];
                let unit_idx = (data & 0xF) as usize;
                let final_value = Self::complex(data) * 100.0;

                match units.get(unit_idx as usize) {
                    Some(unit) => {
                        let integer = final_value.round() as f32;
                        let diff = final_value - integer;
                        let formatted_fraction = if diff > 0.0000001 {
                            format!("{:.*}{}", 6, final_value, unit)
                        } else {
                            format!("{:.*}{}", 1, final_value, unit)
                        };

                        Value::Fraction(formatted_fraction)
                    }
                    None => {
                        return Err(format!("Expected a valid unit index. Got: {}", unit_idx).into())
                    }
                }
            }
            TOKEN_TYPE_INTEGER => {
                // TODO: Should we transmute to signed integer?
                Value::Integer(data as u64)
            }
            TOKEN_TYPE_FLAGS => Value::Flags(data as u64),
            TOKEN_TYPE_FLOAT => {
                let f = unsafe { mem::transmute::<u32, f32>(data) };
                Value::Float(f)
            }
            TOKEN_TYPE_BOOLEAN => {
                if data > 0 {
                    Value::Boolean(true)
                } else {
                    Value::Boolean(false)
                }
            }
            TOKEN_TYPE_COLOR => {
                let formatted_color = format!("#{:08x}", data);
                Value::Color(formatted_color)
            }
            TOKEN_TYPE_COLOR2 => {
                let formatted_color = format!("#{:08x}", data);
                Value::Color2(formatted_color)
            }
            _ => Value::Unknown,

        };

        Ok(value)
    }

    fn complex(data: u32) -> f32 {
        // TODO: Clean this mess
        let mantissa = 0xffffff << 8;
        let uvalue = data & mantissa;
        let ivalue: i32 = unsafe { mem::transmute(uvalue) };
        let m = ivalue as f32;
        let mm = 1.0 / ((1 << 8) as f32);

        let radix = [1.0 * mm,
                     1.0 / ((1 << 7) as f32) * mm,
                     1.0 / ((1 << 15) as f32) * mm,
                     1.0 / ((1 << 23) as f32) * mm];

        let idx = (data >> 4) & 0x3;

        m * radix[idx as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_generate_a_string_value() {
        let value = Value::new(TOKEN_TYPE_STRING, 33);

        assert_eq!("@string/33", value.unwrap().to_string());
    }

    #[test]
    fn it_can_generate_reference_and_dyn_references() {
        let value = Value::new(TOKEN_TYPE_REFERENCE_ID, 12345).unwrap();
        let value2 = Value::new(TOKEN_TYPE_DYN_REFERENCE, 67890).unwrap();

        assert_eq!("@id/0x3039", value.to_string());
        assert_eq!("@id/0x10932", value2.to_string());
    }

    #[test]
    fn it_can_generate_a_positive_dimension() {
        let dim = 1 << 30; // Positive value 2-complement
        let units = 0x5;

        let value = Value::new(TOKEN_TYPE_DIMENSION, dim | units);
        let str_value = value.unwrap().to_string();

        assert_eq!("4194304.0mm", str_value);
    }

    #[test]
    fn it_can_generate_a_negative_dimension() {
        let dim = 1 << 31; // Negative value 2-complement
        let units = 0x0;

        let value = Value::new(TOKEN_TYPE_DIMENSION, dim | units);
        let str_value = value.unwrap().to_string();

        assert_eq!("-8388608.0px", str_value);
    }

    #[test]
    fn it_can_not_generate_a_dimension_if_units_are_out_of_range() {
        let dim = 0;
        let units = 0x6;

        let value = Value::new(TOKEN_TYPE_DIMENSION, dim | units);

        // TODO: Assert error string!
        assert!(value.is_err());
    }

    #[test]
    fn it_can_generate_a_positive_fraction() {
        let dim = 1 << 25; // Positive value 2-complement
        let units = 0x1;

        let value = Value::new(TOKEN_TYPE_FRACTION, dim | units);
        let str_value = value.unwrap().to_string();

        assert_eq!("13107200.0%p", str_value);
    }

    #[test]
    fn it_can_generate_a_negative_fraction() {
        let dim = 1 << 31 | 1 << 5 | 1 << 10; // Positive value 2-complement
        let units = 0x0;

        let value = Value::new(TOKEN_TYPE_FRACTION, dim | units);
        let str_value = value.unwrap().to_string();

        assert_eq!("-25599.988281%", str_value);
    }

    #[test]
    fn it_can_not_generate_a_fraction_if_units_are_out_of_range() {
        let dim = 1 << 31 | 1 << 5 | 1 << 10; // Positive value 2-complement
        let units = 0x2;

        let value = Value::new(TOKEN_TYPE_FRACTION, dim | units);

        // TODO: Assert error string!
        assert!(value.is_err());
    }

    #[test]
    fn it_can_generate_integer_values() {
        let int = 12345;

        let value = Value::new(TOKEN_TYPE_INTEGER, int);

        assert_eq!("12345", value.unwrap().to_string());
    }

    #[test]
    fn it_can_generate_flag_values() {
        let int = 12345;

        let value = Value::new(TOKEN_TYPE_FLAGS, int);

        assert_eq!("12345", value.unwrap().to_string());
    }

    #[test]
    fn it_can_generate_float_values() {
        // TODO: Improve this test with a IEE754 number
        let float = 0;

        let value = Value::new(TOKEN_TYPE_FLOAT, float);

        assert_eq!("0.0", value.unwrap().to_string());
    }

    #[test]
    fn it_can_generate_a_boolean_true_value() {
        let data = 123;

        let value = Value::new(TOKEN_TYPE_BOOLEAN, data);

        assert_eq!("true", value.unwrap().to_string());
    }

    #[test]
    fn it_can_generate_a_boolean_false_value() {
        let data = 0;

        let value = Value::new(TOKEN_TYPE_BOOLEAN, data);

        assert_eq!("false", value.unwrap().to_string());
    }

    #[test]
    fn it_can_generate_a_color_value() {
        let data = 0x01AB23FE;

        let value = Value::new(TOKEN_TYPE_COLOR, data);

        assert_eq!("#01ab23fe", value.unwrap().to_string());
    }

    #[test]
    fn it_can_generate_a_color2_value() {
        let data = 0x01AB23FE;

        let value = Value::new(TOKEN_TYPE_COLOR2, data);

        assert_eq!("#01ab23fe", value.unwrap().to_string());
    }
}
