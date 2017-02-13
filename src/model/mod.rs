pub trait Identifier {
    fn get_package(&self) -> u8;
    fn get_spec(&self) -> u8;
    fn get_id(&self) -> u16;
}

impl Identifier for u32 {
    fn get_package(&self) -> u8 {
        let mut package_id = (self >> 24) as u8;

        if package_id == 0 {
            package_id = 1;
            info!("Resource with package id 0 found. Recreate id with current package id");
        }

        package_id
    }

    fn get_spec(&self) -> u8 {
        ((self & 0x00FF0000) >> 16) as u8
    }

    fn get_id(&self) -> u16 {
        (self & 0xFFFF) as u16
    }
}

#[cfg(test)]
mod tests {
    use model::Identifier;

    #[test]
    fn it_extracts_package_id() {
        assert_eq!(2130837685.get_package(), 127)
    }

    #[test]
    fn it_converts_package_id_if_is_0() {
        assert_eq!(131253.get_package(), 1)
    }

    #[test]
    fn it_extracts_spec_id() {
        assert_eq!(2130837685.get_spec(), 2)
    }

    #[test]
    fn it_extracts_id() {
        assert_eq!(2130837685.get_id(), 181)
    }
}