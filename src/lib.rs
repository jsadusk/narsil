#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate byteorder;

#[macro_use]
extern crate failure;

use failure::Error;
use std::fs::File;

mod model_file;

pub struct Config {
    input_filename : String,
}

impl Config {
    pub fn new(args: &Vec<String>) -> Result<Config, &'static str> {
        if args.len() < 2 {
            Err("Filename missing")
        }
        else {
            let input_filename = args[1].clone();
            Ok(Config { input_filename})
        }
    }

    pub fn input_fh(self) -> Result<File, std::io::Error> {
        File::open(self.input_filename)
    }
}

pub fn run(config : Config) -> Result<(), Error> {
    let (solid, vertices) = model_file::load(config.input_fh()?)?;
    println!("num vertices {}", vertices.len());

    println!("num triangles {}", solid.len());
    
    Ok(())
}
