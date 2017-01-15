use chunks::{Chunk, ChunkHeader, TypeSpec   };
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use document::{HeaderStringTable, StringTable, Value};
use errors::*;
use std::collections::HashMap;
// use parser::Decoder;

pub struct TableTypeDecoder;

const MASK_COMPLEX: u16 = 0x0001;

impl TableTypeDecoder {
    pub fn decode<'a>(cursor: &mut Cursor<&'a [u8]>, header: &ChunkHeader)  -> Result<Chunk<'a>> {
        /*info!("Table type decoding @{}", header.get_offset());
        let id = cursor.read_u8()?;
        cursor.read_u8()?;  // Padding
        cursor.read_u16::<LittleEndian>()?; // Padding
        let count =  cursor.read_u32::<LittleEndian>()?;
        let start = cursor.read_u32::<LittleEndian>()?;

        info!("Resources count: {} [@{}..@{}]", count, start, header.get_chunk_end());

        let config = ResourceConfiguration::from_cursor(cursor)?;
        let a = header.get_offset() + (start as u64) - ((count * 4) as u64);
        cursor.set_position(header.get_data_offset());

        let entries = Self::decode_entries(cursor, id as u32, count).chain_err(|| "Entry decoding failed")?;

        Ok(Chunk::TableType)*/

        let ttw = TableTypeWrapper::new(cursor.get_ref(), (*header).clone());
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

    pub fn get_id(&self) -> u32 {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(8));

        cursor.read_u32::<LittleEndian>().unwrap()
    }

    pub fn get_amount(&self) -> u32 {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(12));

        cursor.read_u32::<LittleEndian>().unwrap()
    }

    pub fn get_configuration(&self) -> Result<ResourceConfiguration> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.absolute(16));

        ResourceConfiguration::from_cursor(&mut cursor)
    }

    pub fn get_entries(&self, type_spec: &TypeSpec<'a>, mask: u32) -> Result<HashMap<u32, Entry>> {
        let mut cursor = Cursor::new(self.raw_data);
        cursor.set_position(self.header.get_data_offset());
        // println!("-> {}", self.get_amount());

        self.decode_entries(&mut cursor, type_spec, mask)
    }

    fn decode_entries(&self, mut cursor: &mut Cursor<&[u8]>, type_spec: &TypeSpec<'a>, mask: u32) -> Result<HashMap<u32, Entry>> {
        let base_offset = cursor.position();
        let mut offsets = Vec::new();
        let mut hentries = HashMap::new();

        let mut prev_offset = base_offset;
        for i in 0..self.get_amount() {
            offsets.push(cursor.read_u32::<LittleEndian>()?);
        }

        for i in 0..self.get_amount() {
            if offsets[i as usize] == 0xFFFFFFFF {
                let position = cursor.position();
                // cursor.set_position(position + 4);
            } else {
                let id = mask | (i & 0xFFFF);
                let maybe_entry = Self::decode_entry(cursor, id)?;

                match maybe_entry {
                    Some(e) => {
                        hentries.insert(id, e);
                    },
                    None => {
                        debug!("Entry with a negative count");
                    }
                }
            }
        }

        Ok(hentries)
    }

    fn decode_entry(cursor: &mut Cursor<&[u8]>, id: u32) -> Result<Option<Entry>> {
        let position = cursor.position();

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

    fn decode_simple_entry(cursor: &mut Cursor<&[u8]>, header: &EntryHeader, id: u32) -> Result<Option<Entry>> {
        let size = cursor.read_u16::<LittleEndian>()?;
        // Padding
        cursor.read_u8()?;
        let val_type = cursor.read_u8()?;
        let data = cursor.read_u32::<LittleEndian>()?;

        let entry = Entry::new_simple(
            id,
            size,
            val_type,
            data,
        );

        Ok(Some(entry))
    }

    fn decode_complex_entry(cursor: &mut Cursor<&[u8]>, header: &EntryHeader, id: u32) -> Result<Option<Entry>> {
        let parent_entry = cursor.read_u32::<LittleEndian>()?;
        let value_count = cursor.read_u32::<LittleEndian>()?;
        let mut entries = Vec::with_capacity(value_count as usize);
        //println!("Current: {}/{}; Amount of values: {}; parent: {}", cursor.position(), cursor.get_ref().len(), value_count, parent_entry);

        if value_count == 0xFFFFFFFF {
            return Ok(None);
        }

        for j in 0..value_count {
            //println!("Current: {}", cursor.position());
            debug!("Parsing value: {}/{} (@{})", j, value_count - 1, cursor.position());
            // println!("Parsing value #{}", j);
            let val_id = cursor.read_u32::<LittleEndian>()?;
            // Resource value
            let size = cursor.read_u16::<LittleEndian>()?;
            // Padding
            cursor.read_u8()?;
            let val_type = cursor.read_u8()?;
            let data = cursor.read_u32::<LittleEndian>()?;

            let simple_entry = Entry::new_simple(
                header.get_key_index(),
                size,
                val_type,
                data,
            );

            entries.push(simple_entry);
        }

        let entry = Entry::new_complex(id, parent_entry, entries);

        Ok(Some(entry))
    }
}

pub struct TableType<'a> {
    wrapper: TableTypeWrapper<'a>,
}

impl<'a> TableType<'a> {
    pub fn new(wrapper: TableTypeWrapper<'a>) -> Self {
        TableType {
            wrapper: wrapper,
        }
    }

    pub fn get_id(&self) -> u8 {
        (self.wrapper.get_id() & 0xF) as u8
    }

    pub fn get_amount(&self) -> u32 {
        self.wrapper.get_amount()
    }

    pub fn get_configuration(&self) -> Result<ResourceConfiguration> {
        self.wrapper.get_configuration()
    }

    pub fn get_entries(&self, type_spec: &TypeSpec<'a>, mask: u32) -> Result<HashMap<u32,   Entry>> {
        self.wrapper.get_entries(type_spec, mask)
    }
}

pub struct EntryHeader {
    header_size: u16,
    flags: u16,
    key_index: u32,
}

impl EntryHeader {
    pub fn new(header_size: u16, flags: u16, key_index: u32) -> Self {
        EntryHeader {
            header_size: header_size,
            flags: flags,
            key_index: key_index,
        }
    }

    pub fn is_complex(&self) -> bool {
        (self.flags & MASK_COMPLEX) == MASK_COMPLEX
    }

    pub fn get_key_index(&self) -> u32 {
        self.key_index
    }
}

#[derive(Debug)]
pub enum Entry {
    Simple {
        key_index: u32,
        size: u16,
        value_type: u8,
        value_data: u32,
    },
    Complex {
        key_index: u32,
        parent_entry_id: u32,
        entries: Vec<Entry>,   // TODO: split this class, Entry will be Entry::Simple here and it can be enforce by type system
    }
}

impl Entry {
    pub fn new_simple(
        key_index: u32,
        size: u16,
        value_type: u8,
        value_data: u32,
    ) -> Self {
        Entry::Simple {
            key_index: key_index,
            size: size,
            value_type: value_type,
            value_data: value_data,
        }
    }

    pub fn new_complex(
        key_index: u32,
        parent_entry_id: u32,
        entries: Vec<Entry>,
    ) -> Self {
        Entry::Complex{
            key_index: key_index,
            parent_entry_id: parent_entry_id,
            entries: entries,
        }
    }

    pub fn get_key(&self) -> u32 {
        match self {
            &Entry::Simple{key_index: ki, size: _, value_type: _, value_data: _} => {
                ki
            },
            &Entry::Complex{key_index: ki, parent_entry_id: _, entries: _} => {
                ki
            },
        }
    }

    pub fn to_value(&self, string_table: &StringTable) -> Result<Value> {
        match self {
            &Entry::Simple {
                key_index: ki,
                size: s,
                value_type: vt,
                value_data: vd
            } => {
                Value::new(vt, vd, &string_table)
            },
            _ => Err("Complex entry can not be converted to value".into()),
        }
    }
}

pub struct Region {
    low: u8,
    high: u8,
}

impl Region {
    pub fn new(low: u8, high: u8) -> Self {
        Region {
            low: low,
            high: high,
        }
    }

    pub fn to_string(&self) -> Result<String> {
        let mut chrs = Vec::new();

        if ((self.low >> 7) & 1) == 1 {
            chrs.push(self.high & 0x1F);
            chrs.push(((self.high & 0xE0) >> 5 ) + ((self.low & 0x03) << 3));
            chrs.push((self.low & 0x7C) >> 2);
        } else {
            chrs.push(self.low);
            chrs.push(self.high);
        }

        String::from_utf8(chrs).chain_err(|| "Could not UTF-8 encode string")
    }
}

#[derive(Debug)]
pub struct ResourceConfiguration {
    size: u32,
    mcc: u16,
    mnc: u16,
    language: String,
    region: String,
    orientation : u8,
    touchscreen: u8,
    density: u16,
    keyboard: u8,
    navigation: u8,
    input_flags: u8,
    width: u16,
    height: u16,
    sdk_version: u16,
    min_sdk_version: u16,
    screen_layout: u8,
    ui_mode: u8,
    smallest_screen: u16,
    screen_width_dp: u16,
    screen_height_dp: u16,
    locale_script: Option<String>,
    locale_variant: Option<String>,
    secondary_screen_layout: Option<u8>,
}

impl ResourceConfiguration {
    pub fn from_cursor(mut cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        let initial_position = cursor.position();
        let size = cursor.read_u32::<LittleEndian>()?;
        let mcc = cursor.read_u16::<LittleEndian>()?;
        let mnc = cursor.read_u16::<LittleEndian>()?;

        let lang1 = cursor.read_u8()?;
        let lang2 = cursor.read_u8()?;

        let lang = Region::new(lang1, lang2);
        let str_lang = lang.to_string()?;

        let reg1 = cursor.read_u8()?;
        let reg2 = cursor.read_u8()?;

        let reg = Region::new(reg1, reg2);
        let str_reg = reg.to_string()?;

        let orientation = cursor.read_u8()?;
        let touchscreen = cursor.read_u8()?;

        let density = cursor.read_u16::<LittleEndian>()?;

        let keyboard = cursor.read_u8()?;
        let navigation = cursor.read_u8()?;
        let input_flags = cursor.read_u8()?;

        cursor.read_u8()?; // Padding

        let width = cursor.read_u16::<LittleEndian>()?;
        let height = cursor.read_u16::<LittleEndian>()?;
        let sdk_version = cursor.read_u16::<LittleEndian>()?;
        let min_sdk_version = cursor.read_u16::<LittleEndian>()?;

        let mut screen_layout = 0;
        let mut ui_mode = 0;
        let mut smallest_screen = 0;
        let mut screen_width_dp = 0;
        let mut screen_height_dp = 0;

        if size >= 32 {
            screen_layout = cursor.read_u8()?;
            ui_mode = cursor.read_u8()?;
            smallest_screen = cursor.read_u16::<LittleEndian>()?;
        }

        if size >= 36 {
            screen_width_dp = cursor.read_u16::<LittleEndian>()?;
            screen_height_dp = cursor.read_u16::<LittleEndian>()?;
        }

        if size >= 48 {
            // TODO: Read following bytes
            cursor.read_u32::<LittleEndian>()?;
            cursor.read_u32::<LittleEndian>()?;
            cursor.read_u32::<LittleEndian>()?;
        }

        if size >= 52 {
            // TODO: Read bytes
        }

        let rc = ResourceConfiguration {
            size: size,
            mcc: mcc,
            mnc: mnc,
            language: str_lang,
            region: str_reg,
            orientation: orientation,
            touchscreen: touchscreen,
            density: density,
            keyboard: keyboard,
            navigation: navigation,
            input_flags: input_flags,
            width: width,
            height: height,
            sdk_version: sdk_version,
            min_sdk_version: min_sdk_version,
            screen_layout: screen_layout,
            ui_mode: ui_mode,
            smallest_screen: smallest_screen,
            screen_width_dp: screen_width_dp,
            screen_height_dp: screen_height_dp,
            locale_script: None,
            locale_variant: None,
            secondary_screen_layout: None,
        };

        Ok(rc)
    }
}
