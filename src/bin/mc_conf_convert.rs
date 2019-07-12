extern crate clap;
use std::io::Write;
use serde_json::Value;
use std::io::Read;
use clap::{App, Arg};

use std::fs::File;
use std::io;

use mc_pointy::convert_json_dirty;


fn main() -> Result<(), io::Error>{
    let matches = App::new("MC Config Converter")
        .version("0.1.0")
        .author("Ben Beauregard <bot190@gmail.com")
        .about("Converts original block replacement definition files to new format")
        .arg(Arg::with_name("Input")
            .required(true)
            .takes_value(true)
            .index(1))
        .arg(Arg::with_name("Output")
            .required(true)
            .takes_value(true)
            .index(2))
        .get_matches();

    // Both of these arguments are required, so unwrap is safe
    let input_path = matches.value_of("Input").unwrap();
    let output_path = matches.value_of("Output").unwrap();

    // We need to validate the provided paths
    // input_path should be open for reading

    let mut input_file = match File::open(input_path) {
        io::Result::Ok(f) => f,
        io::Result::Err(e) => panic!("Error opening input {}: {:?}", input_path, e),
    };

    let mut input_json = String::new();
    input_file.read_to_string(&mut input_json)?;

    // Try to parse the file as json
    let values: Value = serde_json::from_str(input_json.as_str())?;

    // Convert loosely typed json to strict rust types
    let replace_obj = convert_json_dirty(values);

    // Serialize strictly typed data
    let serialized = serde_json::to_string(&replace_obj)?;

    // Open the output file to write the newly serialized data
    let mut output_file = match File::create(output_path) {
        io::Result::Ok(f) => f,
        io::Result::Err(e) => panic!("Error opening output file {}: {:?}", output_path, e),
    };

    output_file.write_all(serialized.as_bytes())?;

    Ok(())
}