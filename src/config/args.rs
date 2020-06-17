use crate::error::*;
use config::*;
use std::fs::File;
use std::path::Path as filePath;

pub struct Args {
    config_filename: String,
    input_filename: String,
    output_filename: String,
}

impl Args {
    pub fn new(cmdline: &Vec<String>) -> Result<Args, String> {
        if cmdline.len() < 4 {
            Err(format!(
                "Usage: {} <config_file> <input_file> <output_file>",
                cmdline[0]
            ))
        } else {
            Ok(Args {
                config_filename: cmdline[1].clone(),
                input_filename: cmdline[2].clone(),
                output_filename: cmdline[3].clone(),
            })
        }
    }

    pub fn config_fh(&self) -> NarsilResult<File> {
        Ok(File::open(self.config_filename.clone())?)
    }

    pub fn config(&self) -> NarsilResult<Config> {
        Config::new(self.config_fh()?)
    }

    pub fn input_fh(&self) -> NarsilResult<File> {
        Ok(File::open(self.input_filename.clone())?)
    }

    pub fn output_fh(&self) -> NarsilResult<File> {
        Ok(File::create(self.output_filename.clone())?)
    }

    pub fn name(&self) -> String {
        let path = filePath::new(self.input_filename.as_str());
        path.file_name().unwrap().to_str().unwrap().to_string()
    }
}
