pub mod ascii_stl;
pub mod binary_stl;
pub mod data;

use std::result::Result;
use std::io::prelude::*;
use std::fs::File;
use std::io::SeekFrom;
use std::io;

#[derive(Fail, Debug)]
pub enum ModelError {
    #[fail(display = "{}", _0)]
    IO(#[fail(cause)] io::Error),
    #[fail(display = "{}", _0)]
    AsciiParse(#[fail(cause)] ascii_stl::StlError),
    #[fail(display = "{}", _0)]
    BinaryParse(#[fail(cause)] binary_stl::StlError),
    #[fail(display = "Unknown file format")]
    Unknown
}

enum FileType {
    Unknown,
    AsciiStl,
    BinaryStl
}

fn identify(fh : &mut File) -> io::Result<FileType> {
    let mut buffer = [0u8; 6];
    let num = fh.read(&mut buffer)?;
    
    fh.seek(SeekFrom::Start(0))?;
    
    if num != 6 {
        return Ok(FileType::Unknown)
    }
    
    if buffer.iter().zip(b"solid".iter()).all(|(a,b)| a == b) {
        return Ok(FileType::AsciiStl)
    }
    else {
        return Ok(FileType::BinaryStl)
    }
}

type ModelResult<T> = Result<T, ModelError>;
    
pub fn load(mut fh : File) -> ModelResult<data::Surfaces> {
    let file_type = identify(&mut fh).map_err(ModelError::IO)?;

    let result = match file_type {
        FileType::AsciiStl => ascii_stl::load(fh).map_err(ModelError::AsciiParse)?,
        FileType::BinaryStl => binary_stl::load(fh).map_err(ModelError::BinaryParse)?,
        FileType::Unknown => return Err(ModelError::Unknown)
    };
    Ok(result)
} 
