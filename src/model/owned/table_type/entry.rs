use errors::*;

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
}
