use std::fs::File;
use model_file::data::*;
use std::io;
use std::io::BufRead;
use regex::Regex;
use std::num;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum StlError {
    Solid(String),
    Facet(String),
    Loop(String),
    Vertex(String),
    Triangle(usize, String),
    Float(num::ParseFloatError, String),
    IO(io::Error)
}

impl error::Error for StlError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::IO(e) => Some(e),
            Self::Float(e, _) => Some(e),
            _ => None
        }
    }
}

impl From<io::Error> for StlError {
    fn from(error : io::Error) -> Self {
        StlError::IO(error)
    }
}

impl fmt::Display for StlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Solid(s) => write!(f, "Expected solid; {}", s),
            Self::Facet(s) => write!(f, "Expected facet or endsolid: {}", s),
            Self::Loop(s) => write!(f, "Expected loop or endfacet: {}", s),
            Self::Vertex(s) => write!(f, "Expected vertex or endloop: {}", s),
            Self::Triangle(n, s) => write!(f, "Triangle has {} vertices, expected 3: {}", n, s),
            Self::Float(e, s) => write!(f, "{}: {}", e, s),
            Self::IO(e) => write!(f, "{}", e)
        }
    }
}

type StlResult<T> = Result<T, StlError>;

enum STLParseState {
    Top,
    Solid,
    Facet,
    Loop
}

pub fn load(fh : File) -> StlResult<FreeSurface> {
    lazy_static! {
        static ref SOLID_RE : Regex =
            Regex::new(r"solid (.+)$").unwrap();
        static ref FACET_RE : Regex =
            Regex::new(r"\s*facet\s+normal\s+([0-9e.+-]+)\s+([0-9e.+-]+)\s+([0-9e.+-]+)$")
            .unwrap();
        static ref LOOP_RE : Regex
            = Regex::new(r"\s*outer loop$").unwrap();
        static ref VERTEX_RE : Regex =
            Regex::new(r"\s*vertex\s+([0-9e.+-]+)\s+([0-9e.+-]+)\s+([0-9e.+-]+)$").unwrap();
        static ref ENDLOOP_RE :Regex
            = Regex::new(r"\s*endloop$").unwrap();
        static ref ENDFACET_RE : Regex
            = Regex::new(r"\s*endfacet$").unwrap();
        static ref ENDSOLID_RE : Regex
            = Regex::new(r"\s*endsolid ([^/s]+)$").unwrap();
    }

    let reader = io::BufReader::new(fh);

    let mut surface = FreeSurface::new();
    let mut _surface_names : Vec<String> = Vec::new();
    let mut triangle = Vertices::new();

    let mut state = STLParseState::Top;

    for line in reader.lines() {
        let line = line?;
        let line_str = line.as_str();

        match state {
            STLParseState::Top => {
                let _cap = SOLID_RE.captures(line_str)
                    .ok_or(StlError::Solid(line_str.to_string()))?;

                state = STLParseState::Solid;
            },
            STLParseState::Solid => {
                match FACET_RE.find(line_str) {
                    Some(_mat) => state = STLParseState::Facet,
                    None => match ENDSOLID_RE.find(line_str) {
                        Some(_mat) => {
                            state = STLParseState::Top;
                        },
                        None => return Err(StlError::Facet(line_str.to_string()))
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
                        None => return Err(StlError::Loop(line_str.to_string()))
                    }
                };

            },
            STLParseState::Loop => {
                match VERTEX_RE.captures(line_str) {
                    Some(cap) => {
                        let x = cap[1].parse::<f64>()
                            .map_err(|e| StlError::Float(e,
                                                         line_str.to_string()))?;
                        let y = cap[2].parse::<f64>()
                            .map_err(|e| StlError::Float(e,
                                                         line_str.to_string()))?;
                        let z = cap[3].parse::<f64>()
                            .map_err(|e| StlError::Float(e,
                                                         line_str.to_string()))?;

                        let vertex = [x, y, z];
                        triangle.push(vertex);
                    },
                    None => match ENDLOOP_RE.find(line_str) {
                        Some(_mat) => {
                            if triangle.len() != 3 {
                                return Err(StlError::Triangle(triangle.len(),
                                                          line_str.to_string()));
                            }

                            let mut real_triangle = FreeTriangle::new();

                            for (i, vertex) in triangle.iter().enumerate() {
                                real_triangle[i] = *vertex;
                            }

                            surface.push(real_triangle);
                            state = STLParseState::Facet;
                        },
                        None =>
                            return Err(StlError::Vertex(line_str.to_string()))
                    }
                };
            }
        }
    }

    Ok(surface)
}
