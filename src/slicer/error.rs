use std::error;
use std::fmt;

#[derive(Debug)]
pub enum SlicerError {
    NonManifold,
    StartingFaceNoAttributes,
    NoLastPointInSlice,
    CurrentFaceNoAttributes,
}

impl error::Error for SlicerError {}

impl fmt::Display for SlicerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonManifold => write!(f, "Model is not manifold"),
            Self::StartingFaceNoAttributes => write!(f, "Starting face is not in attributes map"),
            Self::NoLastPointInSlice => write!(f, "No last point in slice"),
            Self::CurrentFaceNoAttributes => write!(f, "Current face is not in attributes map"),
        }
    }
}

pub type SlicerResult<T> = Result<T, SlicerError>;
