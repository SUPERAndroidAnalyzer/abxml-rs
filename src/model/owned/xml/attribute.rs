use failure::Error;

use crate::model::AttributeTrait;

#[derive(Clone, Debug, Copy)]
pub struct AttributeBuf {
    namespace: u32,
    name: u32,
    class: u32,
    value: u32,
    data: u32,
}

impl AttributeBuf {
    pub fn new(namespace: u32, name: u32, class: u32, value: u32, data: u32) -> Self {
        Self {
            namespace,
            name,
            class,
            value,
            data,
        }
    }
}

impl AttributeTrait for AttributeBuf {
    fn get_namespace(&self) -> Result<u32, Error> {
        Ok(self.namespace)
    }

    fn get_name(&self) -> Result<u32, Error> {
        Ok(self.name)
    }

    fn get_class(&self) -> Result<u32, Error> {
        Ok(self.class)
    }

    fn get_resource_value(&self) -> Result<u32, Error> {
        Ok(self.value)
    }

    fn get_data(&self) -> Result<u32, Error> {
        Ok(self.data)
    }
}
