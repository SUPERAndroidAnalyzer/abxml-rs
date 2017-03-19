use errors::*;
use model::AttributeTrait;

#[derive(Clone)]
pub struct AttributeBuf {
    namespace: u32,
    name: u32,
    class: u32,
    value: u32,
    data: u32,
}

impl AttributeBuf {
    pub fn new(namespace: u32, name: u32, class: u32, value: u32, data: u32) -> Self {
        AttributeBuf {
            namespace: namespace,
            name: name,
            class: class,
            value: value,
            data: data,
        }
    }
}

impl AttributeTrait for AttributeBuf {
    fn get_namespace(&self) -> Result<u32> {
        Ok(self.namespace)
    }

    fn get_name(&self) -> Result<u32> {
        Ok(self.name)
    }

    fn get_class(&self) -> Result<u32> {
        Ok(self.class)
    }

    fn get_resource_value(&self) -> Result<u32> {
        Ok(self.value)
    }

    fn get_data(&self) -> Result<u32> {
        Ok(self.data)
    }
}
