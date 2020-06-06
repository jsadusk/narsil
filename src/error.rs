use model_file;
use slicer;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum NarsilError {
    Model(model_file::ModelError),
    Slicer(slicer::SlicerError),
    IO(std::io::Error),
    Unknown,
}

impl error::Error for NarsilError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Model(e) => Some(e),
            Self::Slicer(e) => Some(e),
            Self::IO(e) => Some(e),
            Self::Unknown => None,
        }
    }
}

impl fmt::Display for NarsilError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Model(e) => write!(f, "Error loading model: {}", e),
            Self::Slicer(e) => write!(f, "Error generating slice outlines: {}", e),
            Self::IO(e) => write!(f, "{}", e),
            Self::Unknown => write!(f, "Unknown error"),
        }
    }
}

impl From<model_file::ModelError> for NarsilError {
    fn from(other: model_file::ModelError) -> Self {
        Self::Model(other)
    }
}

impl From<slicer::SlicerError> for NarsilError {
    fn from(other: slicer::SlicerError) -> Self {
        Self::Slicer(other)
    }
}

impl From<std::io::Error> for NarsilError {
    fn from(other: std::io::Error) -> Self {
        Self::IO(other)
    }
}

impl From<()> for NarsilError {
    fn from(_other: ()) -> Self {
        Self::Unknown
    }
}
