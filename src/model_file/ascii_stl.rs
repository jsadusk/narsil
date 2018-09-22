use std::fmt;
use std::fs::File;
use model_file::data::*;
use std::error;
use std::io;
use std::io::BufRead;
use std::num;
use regex::Regex;

pub struct ExpectedSolidError;
pub struct ExpectedFacetError;
pub struct ExpectedLoopError;
pub struct TriangleNot3VerticesError;
pub struct ExpectedVertexError;

impl error::Error for ExpectedSolidError {
    fn description(&self) -> &str {
        "Expected solid"
    }
}

impl fmt::Debug for ExpectedSolidError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl fmt::Display for ExpectedSolidError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl error::Error for ExpectedFacetError {
    fn description(&self) -> &str {
        "Expected facet or endsolid"
    }
}

impl fmt::Debug for ExpectedFacetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl fmt::Display for ExpectedFacetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl error::Error for ExpectedLoopError {
    fn description(&self) -> &str {
        "Expected loop or endfacet"
    }
}


impl fmt::Debug for ExpectedLoopError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl fmt::Display for ExpectedLoopError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl error::Error for ExpectedVertexError {
    fn description(&self) -> &str {
        "Expected vertex or endloop"
    }
}

impl fmt::Debug for ExpectedVertexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl fmt::Display for ExpectedVertexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl error::Error for TriangleNot3VerticesError {
    fn description(&self) -> &str {
        "Triangle is not three vertices"
    }
}

impl fmt::Debug for TriangleNot3VerticesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl fmt::Display for TriangleNot3VerticesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

pub enum StlError {
    Solid(ExpectedSolidError),
    Facet(ExpectedFacetError),
    Loop(ExpectedLoopError),
    Vertex(ExpectedVertexError),
    Triangle(TriangleNot3VerticesError),
    Float(num::ParseFloatError),
    IO(io::Error)
}

impl fmt::Debug for StlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl fmt::Display for StlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl error::Error for StlError {
    fn description(&self) -> &str {
        match *self {
            StlError::Solid(ref e) => e.description(),
            StlError::Facet(ref e) => e.description(),
            StlError::Loop(ref e) => e.description(),
            StlError::Vertex(ref e) => e.description(),
            StlError::Triangle(ref e) => e.description(),
            StlError::Float(ref e) => e.description(),
            StlError::IO(ref e) => e.description()
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            StlError::Solid(ref e) => Some(e),
            StlError::Facet(ref e) => Some(e),
            StlError::Loop(ref e) => Some(e),
            StlError::Vertex(ref e) => Some(e),
            StlError::Triangle(ref e) => Some(e),
            StlError::Float(ref e) => Some(e),
            StlError::IO(ref e) => Some(e)
        }
    }
}

impl From<ExpectedSolidError> for StlError {
    fn from(error: ExpectedSolidError) -> Self {
        StlError::Solid(error)
    }
}

impl From<num::ParseFloatError> for StlError {
    fn from(error: num::ParseFloatError) -> Self {
        StlError::Float(error)
    }
}

type StlResult<T> = Result<T, StlError>;

enum STLParseState {
    Top,
    Solid,
    Facet,
    Loop
}

pub fn load(fh : File) -> StlResult<Surfaces> {
    lazy_static! {
        static ref SOLID_RE : Regex =
            Regex::new(r"solid ([^\s]+)$").unwrap();
        static ref FACET_RE : Regex =
            Regex::new(r"\s*facet normal ([0-9.-]+) ([0-9.-]+) ([0-9.-]+)$").unwrap();
        static ref LOOP_RE : Regex
            = Regex::new(r"\s*outer loop$").unwrap();
        static ref VERTEX_RE : Regex =
            Regex::new(r"\s*vertex ([0-9.-]+) ([0-9.-]+) ([0-9.-]+)$").unwrap();
        static ref ENDLOOP_RE :Regex
            = Regex::new(r"\s*endloop$").unwrap();
        static ref ENDFACET_RE : Regex
            = Regex::new(r"\s*endfacet$").unwrap();
        static ref ENDSOLID_RE : Regex
            = Regex::new(r"\s*endsolid ([^/s]+)$").unwrap();
    }
    
    let reader = io::BufReader::new(&fh);

    let mut surfaces = Surfaces::new();
    let mut surface = Surface::new();
    let mut surface_names : Vec<String> = Vec::new();
    let mut triangle = Points::new();
    
    let mut state = STLParseState::Top;

    for line in reader.lines() {
        let line = line.map_err(StlError::IO)?;
        let line_str = line.as_str();

        match state {
            STLParseState::Top => {
                let cap = SOLID_RE.captures(line_str)
                    .ok_or(ExpectedSolidError)?;
                
                surface_names.push(cap[1].to_string());
                state = STLParseState::Solid;
            },
            STLParseState::Solid => {
                match FACET_RE.find(line_str) {
                    Some(_mat) => state = STLParseState::Facet,
                    None => match ENDSOLID_RE.find(line_str) {
                        Some(_mat) => {
                            surfaces.push(surface);
                            surface = Surface::new();
                            state = STLParseState::Top;
                        },
                        None => return Err(StlError::Facet(ExpectedFacetError))
                    }
                };

            },
            STLParseState::Facet => {
                match LOOP_RE.captures(line_str) {
                    Some(_cap) => {
                        state = STLParseState::Loop;
                        triangle = Vec::new();
                    },
                    None => match ENDFACET_RE.find(line_str) {
                        Some(_mat) => state = STLParseState::Solid,
                        None => return Err(StlError::Loop(ExpectedLoopError))
                    }
                };

            },
            STLParseState::Loop => {
                match VERTEX_RE.captures(line_str) {
                    Some(cap) => {
                        let x = cap[1].parse::<f64>()?;
                        let y = cap[2].parse::<f64>()?;
                        let z = cap[3].parse::<f64>()?;

                        let point = [x, y, z];
                        triangle.push(point);
                    },
                    None => match ENDLOOP_RE.find(line_str) {
                        Some(_mat) => {
                            if triangle.len() != 3 {
                                return Err(StlError::Triangle(TriangleNot3VerticesError));
                            }

                            let mut real_triangle = Triangle::new();
                            
                            for (i, point) in triangle.iter().enumerate() {
                                real_triangle[i] = *point;
                            }
                            
                            surface.push(real_triangle);
                            state = STLParseState::Facet;
                        },
                        None => return Err(StlError::Vertex(ExpectedVertexError))
                    }
                };
            }
        }
    }

    Ok(surfaces)
}
