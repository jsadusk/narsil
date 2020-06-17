use crate::error::*;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs::File;
use std::io::BufReader;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub layer_height: f64,
    pub resolution: f64,
}

impl Config {
    pub fn new(fh: File) -> NarsilResult<Config> {
        Ok(serde_yaml::from_reader(BufReader::new(fh))?)
    }
}
