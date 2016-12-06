use std::io::{Error, ErrorKind};
use chunks::{Chunk, ChunkLoader, ChunkHeader};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::rc::Rc;
use document::{HeaderStringTable, StringTable};

pub struct PackageDecoder;

impl PackageDecoder {
    pub fn decode(cursor: &mut Cursor<&[u8]>, header: &ChunkHeader)  -> Result<Chunk, Error> {
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

        //chunks.push(Chunk::Package);

        let cursor_len = cursor.get_ref().len() as u64;
        let inner_chunks = ChunkLoader::read_all(cursor, cursor_len)?;
        // chunks.extend(inner_chunks);

        Ok(Chunk::Package)
    }
}
