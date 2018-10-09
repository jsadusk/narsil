#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate byteorder;
extern crate hedge;

#[macro_use]
extern crate failure;

use failure::Error;
use std::fs::File;

mod model_file;
mod slicer;

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
    let mesh = model_file::load(config.input_fh()?)?;
    println!("num triangles {}", mesh.faces().count());

    let slice = slicer::slice(mesh)?;

    for point in slice.iter() {
        println!("Point {}, {}, {}", point[0], point[1], point[2]);
    }
    Ok(())
}
