use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};
use failure::{ensure, format_err, Error};

use model::{
    owned::{ComplexEntry, Entry, EntryHeader, SimpleEntry, TableTypeBuf},
    TableType,
};

pub use self::configuration::{ConfigurationWrapper, Region};

mod configuration;

#[derive(Debug)]
pub struct TableTypeWrapper<'a> {
    raw_data: &'a [u8],
    data_offset: u64,
}

impl<'a> TableTypeWrapper<'a> {
    pub fn new(raw_data: &'a [u8], data_offset: u64) -> Self {
        Self {
            raw_data,
            data_offset,
        }
    }

    pub fn to_buffer(&self) -> Result<TableTypeBuf, Error> {
        let id = self.get_id()?;
        let amount = self.get_amount()?;
        let config = self.get_configuration()?.to_buffer()?;
        let mut owned = TableTypeBuf::new(id & 0xF, config);

        for i in 0..amount {
            let entry = self.get_entry(i)?;
            owned.add_entry(entry);
        }

        Ok(owned)
    }

    pub fn get_entries(&self) -> Result<Vec<Entry>, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.data_offset);

        self.decode_entries(&mut cursor)
    }

    fn decode_entries(&self, cursor: &mut Cursor<&[u8]>) -> Result<Vec<Entry>, Error> {
        let mut offsets = Vec::new();
        let mut entries = Vec::new();

        for _ in 0..self.get_amount()? {
            offsets.push(cursor.read_u32::<LittleEndian>()?);
        }

        for i in 0..self.get_amount()? {
            let id = i & 0xFFFF;

            if offsets[i as usize] == 0xFFFFFFFF {
                entries.push(Entry::Empty(id, id));
            } else {
                let maybe_entry = Self::decode_entry(cursor, id)?;

                if let Some(e) = maybe_entry {
                    entries.push(e);
                } else {
                    debug!("Entry with a negative count");
                }
            }
        }

        Ok(entries)
    }

    fn decode_entry(cursor: &mut Cursor<&[u8]>, id: u32) -> Result<Option<Entry>, Error> {
        let header_size = cursor.read_u16::<LittleEndian>()?;
        let flags = cursor.read_u16::<LittleEndian>()?;
        let key_index = cursor.read_u32::<LittleEndian>()?;
        let header_entry = EntryHeader::new(header_size, flags, key_index);

        if header_entry.is_complex() {
            Self::decode_complex_entry(cursor, &header_entry, id)
        } else {
            Self::decode_simple_entry(cursor, &header_entry, id)
        }
    }

    fn decode_simple_entry(
        cursor: &mut Cursor<&[u8]>,
        header: &EntryHeader,
        id: u32,
    ) -> Result<Option<Entry>, Error> {
        cursor.read_u16::<LittleEndian>()?;
        // Padding
        cursor.read_u8()?;
        let val_type = cursor.read_u8()?;
        let data = cursor.read_u32::<LittleEndian>()?;

        let simple = SimpleEntry::new(id, header.get_key_index(), val_type, data);
        let entry = Entry::Simple(simple);

        Ok(Some(entry))
    }

    fn decode_complex_entry(
        cursor: &mut Cursor<&[u8]>,
        header: &EntryHeader,
        id: u32,
    ) -> Result<Option<Entry>, Error> {
        let parent_entry = cursor.read_u32::<LittleEndian>()?;
        let value_count = cursor.read_u32::<LittleEndian>()?;
        let mut entries = Vec::with_capacity(value_count as usize);

        if value_count == 0xFFFFFFFF {
            return Ok(None);
        }

        for j in 0..value_count {
            debug!(
                "Parsing value: {}/{} (@{})",
                j,
                value_count - 1,
                cursor.position()
            );
            let val_id = cursor.read_u32::<LittleEndian>()?;
            cursor.read_u16::<LittleEndian>()?;
            // Padding
            cursor.read_u8()?;
            let val_type = cursor.read_u8()?;
            let data = cursor.read_u32::<LittleEndian>()?;

            let simple_entry = SimpleEntry::new(val_id, header.get_key_index(), val_type, data);

            entries.push(simple_entry);
        }

        let complex = ComplexEntry::new(id, header.get_key_index(), parent_entry, entries);
        let entry = Entry::Complex(complex);

        Ok(Some(entry))
    }
}

impl<'a> TableType for TableTypeWrapper<'a> {
    type Configuration = ConfigurationWrapper<'a>;

    fn get_id(&self) -> Result<u8, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(8);
        let out_value = cursor.read_u32::<LittleEndian>()? & 0xF;

        Ok(out_value as u8)
    }

    fn get_amount(&self) -> Result<u32, Error> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(12);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    fn get_configuration(&self) -> Result<Self::Configuration, Error> {
        let ini = 20;
        let end = self.data_offset as usize;

        ensure!(
            ini <= end
                && (end - ini) > 28
                && self.raw_data.len() >= ini
                && self.raw_data.len() >= end,
            "configuration slice is not valid"
        );

        let slice = &self.raw_data[ini..end];
        let wrapper = ConfigurationWrapper::new(slice);

        Ok(wrapper)
    }

    fn get_entry(&self, index: u32) -> Result<Entry, Error> {
        let entries = self.get_entries()?;
        entries
            .get(index as usize)
            .cloned()
            .ok_or_else(|| format_err!("entry not found"))
    }
}
