use crate::model::AttributeTrait;
use anyhow::Result;

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
    fn namespace(&self) -> Result<u32> {
        Ok(self.namespace)
    }

    fn name(&self) -> Result<u32> {
        Ok(self.name)
    }

    fn class(&self) -> Result<u32> {
        Ok(self.class)
    }

    fn resource_value(&self) -> Result<u32> {
        Ok(self.value)
    }

    fn data(&self) -> Result<u32> {
        Ok(self.data)
    }
}
