use chunks::{Chunk, ChunkHeader, TypeSpec};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;
use std::collections::HashMap;
use model::TableType as TableTypeTrait;
use model::Configuration;
use model::owned::TableTypeBuf;
use model::owned::{Entry, SimpleEntry, ComplexEntry, EntryHeader};

pub use self::configuration::ConfigurationWrapper;
pub use self::configuration::Region;

pub struct TableTypeDecoder;

mod configuration;

impl TableTypeDecoder {
    pub fn decode<'a>(cursor: &mut Cursor<&'a [u8]>, header: &ChunkHeader) -> Result<Chunk<'a>> {
        let ttw = TableTypeWrapper::new(cursor.get_ref(), *header);
        let configuration = ttw.get_configuration().unwrap();
        let language = configuration.get_language().unwrap();
        let region = configuration.get_language().unwrap();

        println!("Language: {}; Region: {}", language, region);

        Ok(Chunk::TableType(ttw))
    }
}

pub struct TableTypeWrapper<'a> {
    raw_data: &'a [u8],
    header: ChunkHeader,
}

impl<'a> TableTypeWrapper<'a> {
    pub fn new(raw_data: &'a [u8], header: ChunkHeader) -> Self {
        TableTypeWrapper {
            raw_data: raw_data,
            header: header,
        }
    }

    pub fn to_owned(&self) -> Result<TableTypeBuf> {
        let id = self.get_id()?;
        let amount = self.get_amount()?;
        let config = self.get_configuration()?.to_owned()?;
        let mut owned = TableTypeBuf::new((id & 0xF) as u8, config);

        for _ in 0..amount {
            let simple = SimpleEntry::new(1, 2, 3, 4);
            let entry = Entry::Simple(simple);
            owned.add_entry(entry)
        }

        Ok(owned)
    }

    pub fn get_id(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(8));

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    pub fn get_amount(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(12));

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    pub fn get_configuration(&self) -> Result<ConfigurationWrapper<'a>> {
        let ini = self.header.absolute(20) as usize;
        let end = self.header.get_data_offset() as usize;

        if ini > end || (end-ini) <= 28 {
            return Err("Configuration slice is not valid".into());
        }

        let slice = &self.raw_data[ini..end];
        let wrapper = ConfigurationWrapper::new(slice);

        Ok(wrapper)
    }

    pub fn get_entries(&self, type_spec: &TypeSpec<'a>, mask: u32) -> Result<HashMap<u32, Entry>> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.get_data_offset());
        // println!("-> {}", self.get_amount());

        self.decode_entries(&mut cursor, type_spec, mask)
    }

    fn decode_entries(&self,
                      mut cursor: &mut Cursor<&[u8]>,
                      _: &TypeSpec<'a>,
                      mask: u32)
                      -> Result<HashMap<u32, Entry>> {
        let mut offsets = Vec::new();
        let mut entries = HashMap::new();

        for _ in 0..self.get_amount()? {
            offsets.push(cursor.read_u32::<LittleEndian>()?);
        }

        for i in 0..self.get_amount()? {
            let id = mask | (i & 0xFFFF);

            if offsets[i as usize] != 0xFFFFFFFF {
                let maybe_entry = Self::decode_entry(cursor, id)?;

                match maybe_entry {
                    Some(e) => {
                        entries.insert(id, e);
                    }
                    None => {
                        debug!("Entry with a negative count");
                    }
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
            Self::decode_complex_entry(cursor, &header_entry, id)
        } else {
            Self::decode_simple_entry(cursor, &header_entry, id)
        }
    }

    fn decode_simple_entry(cursor: &mut Cursor<&[u8]>,
                           header: &EntryHeader,
                           id: u32)
                           -> Result<Option<Entry>> {
        cursor.read_u16::<LittleEndian>()?;
        // Padding
        cursor.read_u8()?;
        let val_type = cursor.read_u8()?;
        let data = cursor.read_u32::<LittleEndian>()?;

        let simple = SimpleEntry::new(id, header.get_key_index(), val_type, data);
        let entry = Entry::Simple(simple);

        Ok(Some(entry))
    }

    fn decode_complex_entry(cursor: &mut Cursor<&[u8]>,
                            header: &EntryHeader,
                            id: u32)
                            -> Result<Option<Entry>> {
        let parent_entry = cursor.read_u32::<LittleEndian>()?;
        let value_count = cursor.read_u32::<LittleEndian>()?;
        let mut entries = Vec::with_capacity(value_count as usize);

        if value_count == 0xFFFFFFFF {
            return Ok(None);
        }

        for j in 0..value_count {
            debug!("Parsing value: {}/{} (@{})",
            j,
            value_count - 1,
            cursor.position());
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

pub struct TableType<'a> {
    wrapper: TableTypeWrapper<'a>,
}

impl<'a> TableType<'a> {
    pub fn new(wrapper: TableTypeWrapper<'a>) -> Self {
        TableType { wrapper: wrapper }
    }

    pub fn get_entries(&self, type_spec: &TypeSpec<'a>, mask: u32) -> Result<HashMap<u32, Entry>> {
        self.wrapper.get_entries(type_spec, mask)
    }
}

impl<'a> TableTypeTrait for TableType<'a> {
    type Configuration = ConfigurationWrapper<'a>;

    fn get_id(&self) -> Result<u8> {
        Ok((self.wrapper.get_id()? & 0xF) as u8)
    }

    fn get_amount(&self) -> Result<u32> {
        self.wrapper.get_amount()
    }

    fn get_configuration(&self) -> Result<Self::Configuration> {
        self.wrapper.get_configuration()
    }

    fn get_entry(&self, _: u32) -> Result<Entry> {
        // let entries = self.wrapper.get_entries(type_spec, mask);
        let simple = SimpleEntry::new(1, 1, 1, 1);
        Ok(Entry::Simple(simple))
    }
}