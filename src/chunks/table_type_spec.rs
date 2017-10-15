use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;
use model::TypeSpec;
use model::owned::TableTypeSpecBuf;

#[derive(Clone)]
pub struct TypeSpecWrapper<'a> {
    raw_data: &'a [u8],
}

impl<'a> TypeSpecWrapper<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        TypeSpecWrapper { raw_data: slice }
    }

    pub fn to_buffer(&self) -> Result<TableTypeSpecBuf> {
        let mut owned = TableTypeSpecBuf::new(self.get_id()? as u16);
        let amount = self.get_amount()?;

        for i in 0..amount {
            owned.push_flag(self.get_flag(i)?);
        }

        Ok(owned)
    }
}

impl<'a> TypeSpec for TypeSpecWrapper<'a> {
    fn get_id(&self) -> Result<u16> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(8);
        let out_value = cursor.read_u32::<LittleEndian>()? & 0xFF;

        Ok(out_value as u16)
    }

    fn get_amount(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(12);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    fn get_flag(&self, index: u32) -> Result<u32> {
        let amount = self.get_amount()?;

        if index >= amount {
            return Err(
                format!("Invalid flag on index {} out of {}", index, amount).into(),
            );
        }

        let mut cursor = Cursor::new(self.raw_data);
        let flag_offset = 16 + (index * 4) as u64;
        cursor.set_position(flag_offset);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raw_chunks;

    #[test]
    fn it_can_decode_a_type_spec() {
        let wrapper = TypeSpecWrapper::new(raw_chunks::EXAMPLE_TYPE_SPEC);

        assert_eq!(4, wrapper.get_id().unwrap());
        assert_eq!(1541, wrapper.get_amount().unwrap());

        assert_eq!(0x40000004, wrapper.get_flag(0).unwrap());
        assert_eq!(0, wrapper.get_flag(25).unwrap());
        assert_eq!(6, wrapper.get_flag(1540).unwrap());

        let errored_flag = wrapper.get_flag(1541);
        assert!(errored_flag.is_err());
        assert_eq!(
            "Invalid flag on index 1541 out of 1541",
            errored_flag.err().unwrap().to_string()
        );
    }
}
