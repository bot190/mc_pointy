extern crate clap;
use clap::{App, Arg};

use std::io;
use std::fs::{self, File};
use std::path::PathBuf;
use std::ffi::OsStr;

use mc_pointy::{replacements, region_file};


fn main() -> Result<(), io::Error> {
    let matches = App::new("MC ")
        .version("0.1.0")
        .author("Ben Beauregard <bot190@gmail.com")
        .about("Examines MC world files")
        .arg(Arg::with_name("Input")
            .required(true)
            .takes_value(true)
            .index(1))
        .arg(Arg::with_name("Config")
            .required(false)
            .takes_value(true)
            .index(2))
        .get_matches();

    let input_path = matches.value_of("Input").unwrap();

    // Input path is to a folder, we need to find the MCA files inside it
    let mut region_files: Vec<PathBuf> = Vec::new();
    let path = PathBuf::from(input_path);
    if !path.is_dir() {
        match path.extension() {
            Some(ext) => {
                // if ext == OsStr::new("mca") {
                    region_files.push(path)
                // }
            },
            _ => {},
        }
    } else {
        for entry in fs::read_dir(input_path)? {
            // We can't continue if the path provided is invalid so we just return the failure
            let entry = (entry?).path();
            if !entry.is_dir() {
                match entry.extension() {
                    Some(ext) => {
                        if ext == OsStr::new("mca") {
                            region_files.push(entry)
                        }
                    },
                    _ => continue,
                }
            }
        }
    }

    // Handle the config parameter and parse out the config
    let config: Option<replacements::Replacements> = match matches.value_of("Config") {
        Some(path) => match File::open(path) {
                    io::Result::Ok(f) => serde_json::from_reader(f)?,
                    io::Result::Err(e) => panic!("Error opening input {}: {:?}", input_path, e),
                },
        _ => None,
    };


    // Iterate over region files
    for region in region_files {
        println!("Opening region file: {:?}", region);
        let mut file = fs::File::open(region)?;

        // Region files have a special format that isn't just root tags.
        let mut header = region_file::Header::new(&mut file, true).unwrap();
        println!("{}", header);

        // Iterate over Chunk objects in each file
        for chunk in header.iter_mut() {
            let blob = chunk.parse_nbt(&mut file)?;
            println!("{:?}", blob);
            return Ok(())
        }

        return Ok(());
    }
            // Iterate over sections in each Chunk->Level
                // Continue if marker tag is present
                // Call into function to convert blocks/add and data to block object
                    // May include tile entities in this as well to correlate them
                // Search for block ID/Data in hash maps
                    // Match NBT data if necessary
                        // This may require specialized functions for different blocks, etc
                    // Perform replacement
                // Convert block objects back to blocks/add and data int fields
                    // Create required tile entity objects
                // Add marker tag to indicate the chunk has been updated
                    // Add option to clear these tags
    // Write each region file back out

    // Need a method to handle replacing Item IDs in inventories
    Ok(())
}