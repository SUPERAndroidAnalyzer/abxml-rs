use std::io::{Error, ErrorKind};
use chunks::{Chunk, ChunkHeader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use document::{HeaderStringTable, StringTable};

pub struct TableTypeDecoder;

const MASK_COMPLEX: u16 = 0x0001;

impl TableTypeDecoder {
    pub fn decode(cursor: &mut Cursor<&[u8]>, header: &ChunkHeader)  -> Result<Chunk, Error> {
        let id = cursor.read_u32::<LittleEndian>()?;
        let count =  cursor.read_u32::<LittleEndian>()?;
        let start = cursor.read_u32::<LittleEndian>()?;

        let config = ResourceConfiguration::from_cursor(cursor)?;

        cursor.set_position(start as u64);

        Self::decode_entries(cursor, count);

        Ok(Chunk::TableType(id, Box::new(config)))
    }

    fn decode_entries(cursor: &mut Cursor<&[u8]>, entry_amount: u32) -> Result<Vec<Entry>, Error> {
        let base_offset = cursor.position();
        let mut entries = Vec::new();

        for i in 0..entry_amount {
            // let entry = Self::decode_entry(raw_data)
            let offset = cursor.read_u32::<LittleEndian>()?;
            if (offset == 0xFFFFFFFF) {
                continue;
            }

            let position = cursor.position();
            cursor.set_position(base_offset + offset as u64);

            let header_size = cursor.read_u16::<LittleEndian>()?;
            let flags = cursor.read_u16::<LittleEndian>()?;
            let key_index = cursor.read_u32::<LittleEndian>()?;

            //println!("Header size: {}", header_size);
            //println!("Flags: {}", flags);
            //println!("Key index: {}", key_index);

            let mut parent_entry = 0;
            if (flags & MASK_COMPLEX) == MASK_COMPLEX {
                parent_entry = cursor.read_u32::<LittleEndian>()?;
                let value_count = cursor.read_u32::<LittleEndian>()?;

                for j in 0..value_count {
                    let val_id = cursor.read_u32::<LittleEndian>()?;
                    // Resource value
                    let size = cursor.read_u16::<LittleEndian>()?;
                    // Padding
                    cursor.read_u8()?;
                    let val_type = cursor.read_u8()?;
                    let data = cursor.read_u32::<LittleEndian>()?;
                }
            } else {
                let size = cursor.read_u16::<LittleEndian>()?;
                // Padding
                cursor.read_u8()?;
                let val_type = cursor.read_u8()?;
                let data = cursor.read_u32::<LittleEndian>()?;
            }

            cursor.set_position(position);

            let entry = Entry::new(
                header_size,
                flags,
                key_index,
                None,
                parent_entry
            );

            entries.push(entry);
        }

        Ok(entries)
    }
}

#[derive(Debug)]
pub struct Entry {
    header_size: u16,
    flags: u16,
    key_index: u32,
    value: Option<u32>,
    parent_entry: u32,
}

impl Entry {
    pub fn new(
        header_size: u16,
        flags: u16,
        key_index: u32,
        value: Option<u32>,
        parent_entry: u32,
    ) -> Self {
        Entry {
            header_size: header_size,
            flags: flags,
            key_index: key_index,
            value: value,
            parent_entry: parent_entry,
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

    pub fn to_string(&self) -> Result<String, Error> {
        let mut chrs = Vec::new();

        if ((self.low >> 7) & 1) == 1 {
            chrs.push(self.high & 0x1F);
            chrs.push(((self.high & 0xE0) >> 5 ) + ((self.low & 0x03) << 3));
            chrs.push((self.low & 0x7C) >> 2);
        } else {
            chrs.push(self.low);
            chrs.push(self.high);
        }

        match String::from_utf8(chrs) {
            Ok(s) => Ok(s),
            Err(e) => Err(Error::new(ErrorKind::Other, e)),
        }
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
    pub fn from_cursor(mut cursor: &mut Cursor<&[u8]>) -> Result<Self, Error> {
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
