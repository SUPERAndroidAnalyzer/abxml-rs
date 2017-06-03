use errors::*;
use byteorder::{LittleEndian, WriteBytesExt};

const MASK_COMPLEX: u16 = 0x0001;

#[allow(dead_code)]
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

#[derive(Debug, Clone)]
pub struct SimpleEntry {
    id: u32,
    key_index: u32,
    value_type: u8,
    value_data: u32,
}

impl SimpleEntry {
    pub fn new(id: u32, key_index: u32, value_type: u8, value_data: u32) -> Self {
        SimpleEntry {
            id: id,
            key_index: key_index,
            value_type: value_type,
            value_data: value_data,
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_key(&self) -> u32 {
        self.key_index
    }

    pub fn get_type(&self) -> u8 {
        self.value_type
    }

    pub fn get_value(&self) -> u32 {
        self.value_data
    }

    pub fn to_vec(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();

        // Header size
        out.write_u16::<LittleEndian>(8)?;

        // Flags => Simple entry
        out.write_u16::<LittleEndian>(0)?;

        // Key index
        out.write_u32::<LittleEndian>(self.get_key() as u32)?;

        // Value type
        out.write_u16::<LittleEndian>(8)?;
        out.write_u8(0)?;
        out.write_u8(self.get_type())?;

        // Value
        out.write_u32::<LittleEndian>(self.get_value() as u32)?;

        Ok(out)
    }
}

#[derive(Debug, Clone)]
pub struct ComplexEntry {
    id: u32,
    key_index: u32,
    parent_entry_id: u32,
    entries: Vec<SimpleEntry>,
}

impl ComplexEntry {
    pub fn new(id: u32, key_index: u32, parent_entry_id: u32, entries: Vec<SimpleEntry>) -> Self {
        ComplexEntry {
            id: id,
            key_index: key_index,
            parent_entry_id: parent_entry_id,
            entries: entries,
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_key(&self) -> u32 {
        self.key_index
    }

    pub fn get_referent_id(&self, value: u32) -> Option<u32> {
        for e in &self.entries {
            if e.get_value() == value {
                return Some(e.get_id());
            }
        }

        None
    }

    pub fn get_entries(&self) -> &Vec<SimpleEntry> {
        &self.entries
    }

    pub fn to_vec(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();

        // Header size
        out.write_u16::<LittleEndian>(16)?;

        // Flags => Complex entry
        out.write_u16::<LittleEndian>(1)?;

        // Key index
        out.write_u32::<LittleEndian>(self.key_index)?;

        // Parent entry
        out.write_u32::<LittleEndian>(self.parent_entry_id)?;

        // Children entry amount
        let children_amount = self.entries.len() as u32;
        if children_amount == 0 {
            out.write_u32::<LittleEndian>(0xFFFFFFFF)?;
        } else {
            out.write_u32::<LittleEndian>(self.entries.len() as u32)?;
        }

        for e in &self.entries {
            // TODO: Unify this with simple entry without header
            // Key index
            out.write_u32::<LittleEndian>(e.get_id())?;

            // Value type
            out.write_u16::<LittleEndian>(8)?;
            out.write_u8(0)?;
            out.write_u8(e.get_type())?;

            // Value
            out.write_u32::<LittleEndian>(e.get_value() as u32)?;
        }

        Ok(out)
    }
}

#[derive(Debug, Clone)]
pub enum Entry {
    Empty(u32, u32),
    Simple(SimpleEntry),
    Complex(ComplexEntry),
}

impl Entry {
    pub fn simple(&self) -> Result<&SimpleEntry> {
        match *self {
            Entry::Simple(ref simple) => Ok(simple),
            _ => Err("Asked for a complex entry on a simple one".into()),
        }
    }

    pub fn complex(&self) -> Result<&ComplexEntry> {
        match *self {
            Entry::Complex(ref complex) => Ok(complex),
            _ => Err("Asked for a simple entry on a complex one".into()),
        }
    }

    pub fn is_empty(&self) -> bool {
        match *self {
            Entry::Empty(_, _) => true,
            _ => false,
        }
    }

    pub fn get_id(&self) -> u32 {
        match *self {
            Entry::Complex(ref complex) => complex.get_id(),
            Entry::Simple(ref simple) => simple.get_id(),
            Entry::Empty(id, _) => id,
        }
    }

    pub fn get_key(&self) -> u32 {
        match *self {
            Entry::Complex(ref complex) => complex.get_key(),
            Entry::Simple(ref simple) => simple.get_key(),
            Entry::Empty(_, key) => key,
        }
    }

    pub fn to_vec(&self) -> Result<Vec<u8>> {
        match *self {
            Entry::Complex(ref complex) => complex.to_vec(),
            Entry::Simple(ref simple) => simple.to_vec(),
            Entry::Empty(_, _) => Ok(Vec::new()),
        }
    }
}
