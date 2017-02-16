use std::fmt;

#[derive(Clone, Copy)]
pub struct ChunkHeader {
    offset: u64,
    header_size: u16,
    chunk_size: u32,
    chunk_type: u16
}

impl ChunkHeader {
    pub fn new(offset: u64, header_size: u16, chunk_size: u32, chunk_type: u16) -> Self {
        ChunkHeader {
            offset: offset,
            header_size: header_size,
            chunk_size: chunk_size,
            chunk_type: chunk_type,
        }
    }

    pub fn get_offset(&self) -> u64 {
        self.offset
    }

    pub fn get_header_size(&self) -> u16 {
        self.header_size
    }

    pub fn get_data_offset(&self) -> u64 {
        self.offset + self.header_size as u64
    }

    pub fn get_chunk_end(&self) -> u64 {
        self.offset + self.chunk_size as u64
    }

    pub fn absolute(&self, relative: u64) -> u64 {
        let absolute = self.offset + relative;

        if absolute > self.get_chunk_end() {
            panic!("Requested a relative value out of bounds");
        }

        absolute
    }
}

impl fmt::Display for ChunkHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(Token:{:X}; Start: {}; Data: {}; End {})",
            self.chunk_type,
            self.offset,
            self.get_data_offset(),
            self.get_chunk_end()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn it_returns_data_offset() {
        let chunk = ChunkHeader::new(4000, 8, 16, 0);

        assert_eq!(4008, chunk.get_data_offset());
    }

    #[test]
    pub fn it_returns_chunk_end() {
        let chunk = ChunkHeader::new(4000, 8, 16, 0);

        assert_eq!(4016, chunk.get_chunk_end());
    }

    #[test]
    #[should_panic]
    pub fn it_panics_from_relative_out_of_bound() {
        let chunk = ChunkHeader::new(4000, 8, 500, 0);
        let res = chunk.absolute(510);
    }

    #[test]
    pub fn it_returns_absolute_offsets_from_relative_ones() {
        let chunk = ChunkHeader::new(4000, 8, 500, 0);
        let res = chunk.absolute(490);

        assert_eq!(4490, res);
    }
}