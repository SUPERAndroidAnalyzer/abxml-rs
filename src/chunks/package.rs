use chunks::{Chunk, ChunkLoader, ChunkHeader};
use chunks::table_type::Entry;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use document::{HeaderStringTable, StringTable, Value};
use errors::*;

pub struct PackageDecoder;

impl PackageDecoder {
    pub fn decode(cursor: &mut Cursor<&[u8]>, header: &ChunkHeader)  -> Result<Chunk> {
        let id = cursor.read_u32::<LittleEndian>()?;
        println!("Package name position: {:X}", header.get_offset() + 4);
        // let package_name = self.package_name(raw_data, cursor.position() as u32)?;
        // TODO: Read package name
        let pos = cursor.position() + 256;
        cursor.set_position(pos);

        let offset = cursor.read_u32::<LittleEndian>()?;
        let type_string_offset = cursor.read_u32::<LittleEndian>()?;
        let last_public_type = cursor.read_u32::<LittleEndian>()?;
        let key_string_offset = cursor.read_u32::<LittleEndian>()?;
        let last_public_key = cursor.read_u32::<LittleEndian>()?;
        let type_id_offset = cursor.read_u32::<LittleEndian>()?;
        println!("Id: {}", id);
        println!("Type String offset: {}", type_string_offset);
        println!("Last public type: {}", last_public_type);
        println!("Key string offset: {}", key_string_offset);
        println!("Last public key: {}", last_public_key);
        println!("Type ID offset: {}", type_id_offset);

        cursor.set_position(header.get_data_offset());

        let cursor_len = cursor.get_ref().len() as u64;
        let type_string_table = Self::get_string_table(ChunkLoader::read(cursor).unwrap()).unwrap();
        let key_string_table = Self::get_string_table(ChunkLoader::read(cursor).unwrap()).unwrap();

        let inner_chunks = ChunkLoader::read_all(cursor, cursor_len)?;

        for c in inner_chunks {
            match c {
                Chunk::TableType(i, _, entries) => {
                    for e in entries {
                        match e {
                            Entry::Simple{
                                key_index: ki,
                                size: s,
                                value_type: vt,
                                value_data: vd
                            } => {
                                //println!("VT: {}; KI: {}", vt, ki);
                                let v = Value::new(vt as u32, vd, &type_string_table).chain_err(|| "Error decoding data")?;
                                //println!("{}", v.to_string());
                            },
                            _ => (),
                        }
                    }
                    println!("Table type: {}", i);
                },
                Chunk::TableTypeSpec(i, masks) => {
                    let type_name = type_string_table.get_string((i - 1) as usize);
                    println!("Table type spec: {:?}", type_name);
                    // println!("Table type spec: {} {:?}", i, masks);
                },
                Chunk::StringTable(_) => {
                    println!("String table!");
                },
                _ => (),
            }
        }
        // chunks.extend(inner_chunks);

        Ok(Chunk::Package)
    }

    fn get_string_table(chunk: Chunk) -> Option<StringTable> {
        match chunk {
            Chunk::StringTable(st) => Some(st),
            _ => None,
        }
    }
}
