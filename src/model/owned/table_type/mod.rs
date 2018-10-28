use byteorder::{LittleEndian, WriteBytesExt};
use failure::{format_err, Error};

use crate::model::{owned::OwnedBuf, TableType};

mod configuration;
mod entry;

pub use self::{
    configuration::ConfigurationBuf,
    entry::{ComplexEntry, Entry, EntryHeader, SimpleEntry},
};

#[derive(Debug)]
pub struct TableTypeBuf {
    id: u8,
    config: ConfigurationBuf,
    entries: Vec<Entry>,
}

impl TableTypeBuf {
    pub fn new(id: u8, config: ConfigurationBuf) -> Self {
        Self {
            id,
            config,
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: Entry) {
        self.entries.push(entry);
    }
}

impl OwnedBuf for TableTypeBuf {
    fn get_token(&self) -> u16 {
        0x201
    }

    fn get_body_data(&self) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();

        let mut i = 0;
        let mut entries_body = Vec::new();

        for e in &self.entries {
            let current_entry = e.to_vec()?;

            if e.is_empty() {
                out.write_u32::<LittleEndian>(0xFFFF_FFFF)?;
            } else {
                out.write_u32::<LittleEndian>(i)?;
                i += current_entry.len() as u32;
            }

            entries_body.extend(&current_entry);
        }

        out.extend(&entries_body);

        Ok(out)
    }

    fn get_header(&self) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();

        let vec_config = self.config.to_vec()?;
        let header_size = (5 * 4) + vec_config.len() as u32;
        out.write_u32::<LittleEndian>(u32::from(self.id))?;
        out.write_u32::<LittleEndian>(self.entries.len() as u32)?;
        out.write_u32::<LittleEndian>(header_size + (self.entries.len() as u32 * 4))?;
        out.extend(&vec_config);

        Ok(out)
    }
}

impl TableType for TableTypeBuf {
    type Configuration = ConfigurationBuf;

    fn get_id(&self) -> Result<u8, Error> {
        Ok(self.id)
    }

    fn get_amount(&self) -> Result<u32, Error> {
        Ok(self.entries.len() as u32)
    }

    fn get_configuration(&self) -> Result<Self::Configuration, Error> {
        Ok(self.config.clone())
    }

    fn get_entry(&self, index: u32) -> Result<Entry, Error> {
        self.entries
            .get(index as usize)
            .cloned()
            .ok_or_else(|| format_err!("entry out of bound"))
    }
}

#[cfg(test)]
mod tests {
    use super::{ComplexEntry, ConfigurationBuf, Entry, SimpleEntry, TableTypeBuf};
    use crate::{
        chunks::TableTypeWrapper,
        model::{owned::OwnedBuf, TableType},
        raw_chunks,
        test::compare_chunks,
    };

    #[test]
    fn it_can_generate_a_chunk_with_the_given_data() {
        let mut table_type = TableTypeBuf::new(5, ConfigurationBuf::default());

        let entry = Entry::Simple(SimpleEntry::new(1, 2, 3, 4));
        let sub_entry = SimpleEntry::new(5, 6, 7, 8);
        let sub_entry2 = SimpleEntry::new(9, 0, 1, 2);

        let entry2 = Entry::Complex(ComplexEntry::new(10, 11, 12, vec![sub_entry, sub_entry2]));
        let entry3 = Entry::Simple(SimpleEntry::new(20, 21, 22, 23));

        table_type.add_entry(entry);
        table_type.add_entry(entry2);
        table_type.add_entry(entry3);

        assert_eq!(5, table_type.get_id().unwrap());
        assert_eq!(3, table_type.get_amount().unwrap());
        assert_eq!(
            10,
            table_type.get_entry(1).unwrap().complex().unwrap().get_id()
        )
    }

    #[test]
    fn identity() {
        let wrapper = TableTypeWrapper::new(raw_chunks::EXAMPLE_TABLE_TYPE, 68);
        let _ = wrapper.get_entries();

        let owned = wrapper.to_buffer().unwrap();
        let new_raw = owned.to_vec().unwrap();

        compare_chunks(&new_raw, &raw_chunks::EXAMPLE_TABLE_TYPE);
    }

    #[test]
    fn identity_with_mixed_complex_and_simple_entries() {
        let wrapper = TableTypeWrapper::new(raw_chunks::EXAMPLE_TABLE_TYPE_WITH_COMPLEX, 76);
        let _ = wrapper.get_entries();

        let owned = wrapper.to_buffer().unwrap();
        let new_raw = owned.to_vec().unwrap();

        compare_chunks(&new_raw, &raw_chunks::EXAMPLE_TABLE_TYPE_WITH_COMPLEX);
    }
}
