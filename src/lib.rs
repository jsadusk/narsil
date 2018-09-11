#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::error::Error;
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

pub fn run(config : Config) -> Result<(), Box<Error>> {
    let solids = model_file::ascii_stl::load(config.input_fh()?)?;
    println!("num solids {}", solids.len());

    let mut sum = 0;
    for solid in solids {
        sum += solid.len();
    }

    println!("num triangles {}", sum);
    
    Ok(())
}
