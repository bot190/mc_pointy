pub mod region_file {
    use std::io::{Read, Seek, SeekFrom, Error, ErrorKind};
    use std::convert::TryInto;
    use std::fmt;

    use nbt;

    use byteorder::{ReadBytesExt, BigEndian};

    const SECTOR_SIZE: u32 = 4096;

    #[derive(Debug)]
    pub struct Header {
        pub chunks: Vec<Option<Chunk>>,
        timestamps: Vec<u32>,
    }

    // Wraps a vec iterator for the chunks vec.
    pub struct IterHeader<'a> {
        iter: std::slice::Iter<'a, Option<Chunk>>,
    }

    impl<'a> Iterator for IterHeader<'a> {
        type Item = &'a Chunk;

        fn next(&mut self) -> Option<Self::Item> {
            while let Some(chunk) = self.iter.next() {
                if chunk.is_some() {
                    return chunk.as_ref();
                }
            }
            None
        }
    }

    // Wraps a mut vec iterator for the chunks vec.
    pub struct IterHeaderMut<'a> {
        iter: std::slice::IterMut<'a, Option<Chunk>>,
    }

    impl<'a> Iterator for IterHeaderMut<'a> {
        type Item = &'a mut Chunk;

        fn next(&mut self) -> Option<Self::Item> {
            while let Some(chunk) = self.iter.next() {
                if chunk.is_some() {
                    return chunk.as_mut();
                }
            }
            None
        }
    }

    impl fmt::Display for Header {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for (index, chunk) in self.chunks.iter().enumerate() {
                match chunk {
                    Some(chunk) => {
                        write!(f, "{}\n \
                        Modified: {}\n", chunk, self.timestamps[index])?;
                    },
                    None => continue,
                }
            }
            Ok(())
        }
    }

    impl Header {
        pub fn new<R>(src: &mut R, visit_chunks: bool) -> Option<Header>
        where R: Read + Seek
        {
            let mut processed_chunks: Vec<Option<Chunk>> = Vec::new();
            let mut processed_timestamps: Vec<u32> = Vec::new();
            let mut buffer = [0u32; 1024];

            // Parse the locations first
            if !src.read_u32_into::<BigEndian>(&mut buffer[..]).is_ok() {
                return None
            }

            // Create location objects for each chunk position and put them into the vector
            for (index, val) in buffer.iter().enumerate() {
                let x = index % 32;
                let z = index / 32;
                processed_chunks.push(Chunk::new(x.try_into().unwrap(), z.try_into().unwrap(), *val));
            }

            // Parse the Timestamps second
            if !src.read_u32_into::<BigEndian>(&mut buffer[..]).is_ok() {
                return None
            }

            // Push the timestamps directly
            processed_timestamps.extend(buffer.iter());

            if visit_chunks {
                let mut chunk_iter = processed_chunks.iter_mut();
                while let Some(chunk) = chunk_iter.next() {
                    match chunk {
                        Some(c) => c.parse_chunk_header(src).is_err(),
                        None => continue,
                    };
                }
            }

            Some(Header {
                chunks: processed_chunks,
                timestamps: processed_timestamps,
            })
        }

        pub fn iter<'a>(&'a self) -> IterHeader<'a> {
            IterHeader {
                iter: self.chunks.iter(),
            }
        }

        pub fn iter_mut(&mut self) -> IterHeaderMut {
            IterHeaderMut {
                iter: self.chunks.iter_mut(),
            }
        }
    }

    #[derive(Debug)]
    struct Coords {
        x: u32,
        z: u32
    }

    impl fmt::Display for Coords {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "(X, Z) ({}, {})", self.x, self.z)
        }
    }

    #[derive(Debug)]
    enum CompressionType {
        Gzip,
        Zlib,
    }
    impl fmt::Display for CompressionType {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match *self {
                CompressionType::Gzip => write!(f, "GZIP"),
                CompressionType::Zlib => write!(f, "ZLIB"),
            }
        }
    }

    #[derive(Debug)]
    pub struct Chunk {
        // Chunk Coordinates, relative to this region.
        location: Coords,
        // Offset in sectors of the chunks data in this region file.
        offset: u32,
        // Length of the chunk data in sectors.
        sector_count: u8,
        // Length of the chunk data in bytes, read from the chunk header.
        byte_count: Option<u32>,
        // Type of compression used for the chunk data, read from the chunk header.
        compression_type: Option<CompressionType>,
    }

    impl fmt::Display for Chunk {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Chunk at: {}\n \
            Uses: {} sectors at: {}\n ", self.location, self.sector_count, self.offset)?;
            match self.byte_count {
                Some (count) => {
                    write!(f, "Has {} bytes of {} compressed data",
                        count,
                        self.compression_type.as_ref().unwrap())
                },
                None => write!(f, "Hasn't been processed")
            }
        }
    }

    impl Chunk {
        // Chunks are created while reading in the region header.
        pub fn new(x: u32, z: u32, val: u32) -> Option<Chunk> {
            if val == 0 {
                return None
            }
            Some(Chunk {
                location: Coords {x, z},
                offset: (val & 0xFFFFFF00) >> 8,
                sector_count: (val & 0xFF) as u8,
                byte_count: None,
                compression_type: None
            })
        }

        // Read the header for this chunk and parse the data length and compression type.
        pub fn parse_chunk_header<R>(&mut self, src: &mut R) -> Result<(), Error>
        where R: Read + Seek
        {
            let seek_loc = self.offset * SECTOR_SIZE;
            src.seek(SeekFrom::Start(seek_loc as u64))?;
            let byte_count = src.read_u32::<BigEndian>()?;
            self.byte_count = Some(byte_count);
            match src.read_u8()? {
                1 => self.compression_type = Some(CompressionType::Gzip),
                2 => self.compression_type = Some(CompressionType::Zlib),
                _ => {},
            };
            Ok(())
        }

        // Gets the size of the chunk in bytes.
        // If the region file is passed in it will attempt to parse the chunk header
        // if it hasn't already.
        pub fn get_byte_count<R>(&mut self, src: Option<&mut R>) -> Option<u32>
        where R: Read + Seek
        {
            match self.byte_count {
                Some(count) => Some(count),
                None => {
                    if src.is_some() {
                        if self.parse_chunk_header(src.unwrap()).is_err() {
                            return None
                        }
                        self.byte_count
                    } else {
                        None
                    }
                }
            }
        }

        // Reads the chunk data from the given region file and parses it into an NBT blob.
        // Eventually this should parse the NBT data into a more useful format.
        pub fn parse_nbt<R>(& self, src: &mut R) -> Result<nbt::Blob, Error>
        where R: Read + Seek
        {
            // Chunk data starts at Sector Offset * Sector Size + 4 byte length, + compression type byte.
            let seek_loc = self.offset * SECTOR_SIZE + 5;
            src.seek(SeekFrom::Start(seek_loc as u64))?;
            let blob = match self.compression_type {
                Some(CompressionType::Gzip) => nbt::Blob::from_gzip_reader(src)?,
                Some(CompressionType::Zlib) => nbt::Blob::from_zlib_reader(src)?,
                None => return Err(Error::new(ErrorKind::Other, "Need to parse chunk header before trying to parse NBT"))
            };
            Ok(blob)
        }
    }
}