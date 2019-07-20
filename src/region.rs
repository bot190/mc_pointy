pub mod region_file {
    use std::io::Read;

    use byteorder::{ReadBytesExt, BigEndian};

    #[derive(Debug)]
    pub struct Header {
        locations: Vec<Option<Location>>,
        timestamps: Vec<u32>,
    }

    impl Header {
        pub fn new<R>(src: &mut R) -> Option<Header>
        where R: Read
        {
            let mut processed_locations: Vec<Option<Location>> = Vec::new();
            let mut processed_timestamps: Vec<u32> = Vec::new();
            let mut buffer = [0u32; 1024];

            // Parse the locations first
            if !src.read_u32_into::<BigEndian>(&mut buffer[..]).is_ok() {
                return None
            }

            // Create location objects for each chunk position and put them into the vector
            for val in buffer.iter() {
                processed_locations.push(Location::new(*val));
            }

            // Parse the Timestamps second
            if !src.read_u32_into::<BigEndian>(&mut buffer[..]).is_ok() {
                return None
            }

            // Push the timestamps directly
            processed_timestamps.extend(buffer.iter());

            Some(Header {
                locations: processed_locations,
                timestamps: processed_timestamps,
            })
        }
    }

    #[derive(Debug)]
    pub struct Location {
        offset: u32,
        count: u8,
    }

    impl Location {
        pub fn new(val: u32) -> Option<Location> {
            if val == 0 {
                return None
            }
            println!("{:b}", val);
            Some(Location {
                offset: (val & 0xFFFFFF00) >> 8,
                count: (val & 0xFF) as u8,
            })
        }
    }

}