use std::rc::Rc;
use model::Value;

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
               name_index: u32,
    ) -> Self {
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
}
