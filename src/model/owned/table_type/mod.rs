use model::owned::OwnedBuf;
use errors::*;
use model::TableType;
use model::Entries;
use chunks::table_type::Entry;
use byteorder::{LittleEndian, WriteBytesExt};

mod configuration;

pub use self::configuration::ConfigurationBuf;

pub struct TableTypeBuf {
    id: u8,
    config: ConfigurationBuf,
    entries: Entries,
}

impl TableTypeBuf {
    pub fn new(id: u8, config: ConfigurationBuf) -> Self {
        TableTypeBuf {
            id: id,
            config: config,
            entries: Entries::new(),
        }
    }

    pub fn add_entry(&mut self, entry: Entry) {
        // TODO: Check that parent is already on the entries hash table?
        self.entries.insert(entry.get_id(), entry);
    }
}

impl OwnedBuf for TableTypeBuf {
    fn get_token(&self) -> u16 {
        0x201
    }

    fn get_body_data(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();

        out.write_u32::<LittleEndian>(self.id as u32)?;
        out.write_u32::<LittleEndian>(self.entries.len() as u32)?;

        Ok(out)
    }

    fn get_header_size(&self) -> u16 {
        // It seems that can be either 68 or 76
        68
    }
}

impl TableType for TableTypeBuf {
    type Configuration = ConfigurationBuf;

    fn get_id(&self) -> Result<u8> {
        Ok(self.id)
    }

    fn get_amount(&self) -> Result<u32> {
        Ok(self.entries.len() as u32)
    }

    fn get_configuration(&self) -> Result<Self::Configuration> {
        Ok(self.config.clone())
    }

    fn get_entry(&self, index: u32) -> Result<Entry> {
        self.entries.get(&index).map(|x| x.clone()).ok_or("Entry not found".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chunks::*;
    use model::TableType;

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
        assert_eq!(10, table_type.get_entry(10).unwrap().complex().unwrap().get_id())
    }

    #[test]
    fn identity() {
        /*let header = ChunkHeader::new(0, 16, raw_chunks::EXAMPLE_TYPE_SPEC.len() as u32, 0x202);
        let wrapper = TypeSpecWrapper::new(raw_chunks::EXAMPLE_TYPE_SPEC, header);

        let owned = wrapper.to_owned().unwrap();
        let new_raw = owned.to_vec().unwrap();

        compare_chunks(&new_raw, &raw_chunks::EXAMPLE_TYPE_SPEC);*/
    }
}