use chunks::{Chunk, ChunkLoader, ChunkHeader};
use chunks::table_type::Entry;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use document::{HeaderStringTable, StringTable, Value};
use errors::*;
use parser::Decoder;

pub struct PackageDecoder;

impl PackageDecoder {
    pub fn decode(mut decoder: &mut Decoder, cursor: &mut Cursor<&[u8]>, header: &ChunkHeader)  -> Result<Chunk> {
        let id = cursor.read_u32::<LittleEndian>()?;
        let package_name = Self::package_name(cursor)?;
        println!("Package name: {}", package_name);

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
        let type_string_table = Self::get_string_table(ChunkLoader::read(decoder, cursor).unwrap()).unwrap();
        let key_string_table = Self::get_string_table(ChunkLoader::read(decoder, cursor).unwrap()).unwrap();

        let inner_chunks = ChunkLoader::read_all(decoder, cursor, cursor_len)?;
        let st = decoder.get_string_table();
        let rc_st = match st {
            &Some(ref rc_st) => {println!("Has string table"); rc_st.clone()},
            &None => {return Err("No string table found".into());}
        };

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
                                if i == 1 {
                                    println!("VT: {}; KI: {}; VD: {}", vt, ki, vd);
                                }
                                let v = Value::new(vt, vd, &rc_st).chain_err(|| "Error decoding data")?;
                                //println!("{}", v.to_string());
                            },
                            Entry::Complex{
                                key_index: ki,
                                parent_entry_id: pei,
                                entries: entries,
                            } => {
                                if i == 1 {
                                    let key = key_string_table.get_string(ki as usize);
                                    println!("Complex types! {}({:?}) parent: {}", ki, key, pei);

                                    for e in entries {
                                        let v = e.to_value(&type_string_table).chain_err(|| "Could not convert entry to value")?;
                                        println!("Value: {}", v.to_string());
                                        println!("{:?}", v);
                                    }
                                }
                            },
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

    fn get_string_table(chunk: Chunk) -> Option<Rc<StringTable>> {
        match chunk {
            Chunk::StringTable(st) => Some(st.clone()),
            _ => None,
        }
    }

    fn package_name(cursor: &mut Cursor<&[u8]>) -> Result<String> {
        let initial_position = cursor.position();
        let raw_str = cursor.get_ref()[initial_position as usize..(initial_position+256) as usize].to_vec();
        String::from_utf8(raw_str).chain_err(|| "Could not convert to UTF-8")
    }
}
