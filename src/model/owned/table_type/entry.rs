use anyhow::{bail, Result};
use byteorder::{LittleEndian, WriteBytesExt};

const MASK_COMPLEX: u16 = 0x0001;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub struct EntryHeader {
    header_size: u16,
    flags: u16,
    key_index: u32,
}

impl EntryHeader {
    pub fn new(header_size: u16, flags: u16, key_index: u32) -> Self {
        Self {
            header_size,
            flags,
            key_index,
        }
    }

    pub fn is_complex(self) -> bool {
        (self.flags & MASK_COMPLEX) == MASK_COMPLEX
    }

    pub fn key_index(self) -> u32 {
        self.key_index
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SimpleEntry {
    id: u32,
    key_index: u32,
    value_type: u8,
    value_data: u32,
}

impl SimpleEntry {
    pub fn new(id: u32, key_index: u32, value_type: u8, value_data: u32) -> Self {
        Self {
            id,
            key_index,
            value_type,
            value_data,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn key(&self) -> u32 {
        self.key_index
    }

    pub fn get_type(&self) -> u8 {
        self.value_type
    }

    pub fn value(&self) -> u32 {
        self.value_data
    }

    pub fn to_vec(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();

        // Header size
        out.write_u16::<LittleEndian>(8)?;

        // Flags => Simple entry
        out.write_u16::<LittleEndian>(0)?;

        // Key index
        out.write_u32::<LittleEndian>(self.key())?;

        // Value type
        out.write_u16::<LittleEndian>(8)?;
        out.write_u8(0)?;
        out.write_u8(self.get_type())?;

        // Value
        out.write_u32::<LittleEndian>(self.value())?;

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
        Self {
            id,
            key_index,
            parent_entry_id,
            entries,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn key(&self) -> u32 {
        self.key_index
    }

    pub fn referent_id(&self, value: u32) -> Option<u32> {
        for e in &self.entries {
            if e.value() == value {
                return Some(e.id());
            }
        }

        None
    }

    pub fn entries(&self) -> &[SimpleEntry] {
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
            out.write_u32::<LittleEndian>(0xFFFF_FFFF)?;
        } else {
            out.write_u32::<LittleEndian>(self.entries.len() as u32)?;
        }

        for e in &self.entries {
            // TODO: Unify this with simple entry without header
            // Key index
            out.write_u32::<LittleEndian>(e.id())?;

            // Value type
            out.write_u16::<LittleEndian>(8)?;
            out.write_u8(0)?;
            out.write_u8(e.get_type())?;

            // Value
            out.write_u32::<LittleEndian>(e.value())?;
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
        if let Self::Simple(simple) = self {
            Ok(simple)
        } else {
            bail!("asked for a complex entry on a simple one")
        }
    }

    pub fn complex(&self) -> Result<&ComplexEntry> {
        if let Self::Complex(complex) = self {
            Ok(complex)
        } else {
            bail!("asked for a simple entry on a complex one")
        }
    }

    pub fn is_empty(&self) -> bool {
        if let Self::Empty(_, _) = self {
            true
        } else {
            false
        }
    }

    pub fn id(&self) -> u32 {
        match self {
            Self::Complex(complex) => complex.id(),
            Self::Simple(simple) => simple.id(),
            Self::Empty(id, _) => *id,
        }
    }

    pub fn key(&self) -> u32 {
        match self {
            Self::Complex(complex) => complex.key(),
            Self::Simple(simple) => simple.key(),
            Self::Empty(_, key) => *key,
        }
    }

    pub fn to_vec(&self) -> Result<Vec<u8>> {
        match self {
            Self::Complex(complex) => complex.to_vec(),
            Self::Simple(simple) => simple.to_vec(),
            Self::Empty(_, _) => Ok(Vec::new()),
        }
    }
}
