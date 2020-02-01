pub use self::configuration::{ConfigurationWrapper, Region};
use crate::model::{
    owned::{ComplexEntry, Entry, EntryHeader, SimpleEntry, TableTypeBuf},
    TableType,
};
use anyhow::{anyhow, ensure, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use log::debug;
use std::io::Cursor;

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

    pub fn to_buffer(&self) -> Result<TableTypeBuf> {
        let id = self.id()?;
        let amount = self.amount()?;
        let config = self.configuration()?.to_buffer()?;
        let mut owned = TableTypeBuf::new(id & 0xF, config);

        for i in 0..amount {
            let entry = self.entry(i)?;
            owned.add_entry(entry);
        }

        Ok(owned)
    }

    pub fn entries(&self) -> Result<Vec<Entry>> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.data_offset);

        self.decode_entries(&mut cursor)
    }

    fn decode_entries(&self, cursor: &mut Cursor<&[u8]>) -> Result<Vec<Entry>> {
        let mut offsets = Vec::new();
        let mut entries = Vec::new();
        let amount = self.amount()?;

        for _ in 0..amount {
            offsets.push(cursor.read_u32::<LittleEndian>()?);
        }

        for i in 0..amount {
            let id = i & 0xFFFF;

            if offsets[i as usize] == 0xFFFF_FFFF {
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

    fn decode_entry(cursor: &mut Cursor<&[u8]>, id: u32) -> Result<Option<Entry>> {
        let header_size = cursor.read_u16::<LittleEndian>()?;
        let flags = cursor.read_u16::<LittleEndian>()?;
        let key_index = cursor.read_u32::<LittleEndian>()?;
        let header_entry = EntryHeader::new(header_size, flags, key_index);

        if header_entry.is_complex() {
            Self::decode_complex_entry(cursor, header_entry, id)
        } else {
            Self::decode_simple_entry(cursor, header_entry, id)
        }
    }

    fn decode_simple_entry(
        cursor: &mut Cursor<&[u8]>,
        header: EntryHeader,
        id: u32,
    ) -> Result<Option<Entry>> {
        cursor.read_u16::<LittleEndian>()?;
        // Padding
        cursor.read_u8()?;
        let val_type = cursor.read_u8()?;
        let data = cursor.read_u32::<LittleEndian>()?;

        let simple = SimpleEntry::new(id, header.key_index(), val_type, data);
        let entry = Entry::Simple(simple);

        Ok(Some(entry))
    }

    fn decode_complex_entry(
        cursor: &mut Cursor<&[u8]>,
        header: EntryHeader,
        id: u32,
    ) -> Result<Option<Entry>> {
        let parent_entry = cursor.read_u32::<LittleEndian>()?;
        let value_count = cursor.read_u32::<LittleEndian>()?;
        let mut entries = Vec::with_capacity(value_count as usize);

        if value_count == 0xFFFF_FFFF {
            return Ok(None);
        }

        for _ in 0..value_count {
            let val_id = cursor.read_u32::<LittleEndian>()?;
            cursor.read_u16::<LittleEndian>()?;
            // Padding
            cursor.read_u8()?;
            let val_type = cursor.read_u8()?;
            let data = cursor.read_u32::<LittleEndian>()?;

            let simple_entry = SimpleEntry::new(val_id, header.key_index(), val_type, data);

            entries.push(simple_entry);
        }

        let complex = ComplexEntry::new(id, header.key_index(), parent_entry, entries);
        let entry = Entry::Complex(complex);

        Ok(Some(entry))
    }
}

impl<'a> TableType for TableTypeWrapper<'a> {
    type Configuration = ConfigurationWrapper<'a>;

    fn id(&self) -> Result<u8> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(8);
        let out_value = cursor.read_u32::<LittleEndian>()? & 0xF;

        Ok(out_value as u8)
    }

    fn amount(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(12);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    fn configuration(&self) -> Result<Self::Configuration> {
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

    fn entry(&self, index: u32) -> Result<Entry> {
        let entries = self.entries()?;
        entries
            .get(index as usize)
            .cloned()
            .ok_or_else(|| anyhow!("entry not found"))
    }
}
