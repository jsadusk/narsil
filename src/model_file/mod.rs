pub mod ascii_stl;
pub mod data;

use std::result::Result;
use std::io::prelude::*;
use std::fs::File;
use std::io::SeekFrom;
use std::io;
use std::error;
use std::fmt;

pub struct UnknownFileError;

impl error::Error for UnknownFileError {
    fn description(&self) -> &str {
        "Unknown file format"
    }
}

impl fmt::Debug for UnknownFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl fmt::Display for UnknownFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

pub struct BinaryParseError;

impl fmt::Debug for BinaryParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl fmt::Display for BinaryParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl error::Error for BinaryParseError {
    fn description(&self) -> &str {
        "Binary STL not supported yet"
    }
}

pub enum ModelError {
    IO(io::Error),
    AsciiParse(ascii_stl::StlError),
    BinaryParse(BinaryParseError),
    Unknown(UnknownFileError)
}

impl From<BinaryParseError> for  ModelError {
    fn from(error: BinaryParseError) -> Self {
        ModelError::BinaryParse(error)
    }
}


impl fmt::Debug for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl error::Error for ModelError {
    fn description(&self) -> &str {
        match *self {
            ModelError::IO(ref e) => e.description(),
            ModelError::AsciiParse(ref e) => e.description(),
            ModelError::BinaryParse(ref e) => e.description(),
            ModelError::Unknown(ref e) => e.description()
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ModelError::IO(ref e) => Some(e),
            ModelError::AsciiParse(ref stl_e) => match stl_e {
                ascii_stl::StlError::IO(ref e) => Some(e),
                _ => Some(stl_e)
            },
            ModelError::BinaryParse(ref e) => Some(e),
            ModelError::Unknown(ref e) => Some(e)
        }
    }
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
        FileType::BinaryStl => return Err(ModelError::BinaryParse(BinaryParseError)),
        FileType::Unknown => return Err(ModelError::Unknown(UnknownFileError))
    };
    Ok(result)
} 
