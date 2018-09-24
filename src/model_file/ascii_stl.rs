use std::fs::File;
use model_file::data::*;
use std::io;
use std::io::BufRead;
use regex::Regex;
use std::num;

#[derive(Fail, Debug)]
pub enum StlError {
    #[fail(display = "Expected solid: {}", _0)]
    Solid(String),
    #[fail(display = "Expected facet or endsolid: {}", _0)]
    Facet(String),
    #[fail(display = "Expected loop or endfacet: {}", _0)]
    Loop(String),
    #[fail(display = "Expected vertex or endloop: {}", _0)]
    Vertex(String),
    #[fail(display = "Triangle has {} vertices, expected 3: {}", _0, _1)]
    Triangle(usize, String),
    #[fail(display = "{}: {}", _0, _1)]
    Float(#[fail(cause)] num::ParseFloatError, String),
    #[fail(display = "{}", _0)]
    IO(#[fail(cause)] io::Error)
}

impl From<io::Error> for StlError {
    fn from(error : io::Error) -> Self {
        StlError::IO(error)
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
            Regex::new(r"\s*facet normal ([0-9e.-]+) ([0-9e.-]+) ([0-9e.-]+)$")
            .unwrap();
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
        let line = line?;
        let line_str = line.as_str();

        match state {
            STLParseState::Top => {
                let cap = SOLID_RE.captures(line_str)
                    .ok_or(StlError::Solid(line_str.to_string()))?;
                
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

                        let point = [x, y, z];
                        triangle.push(point);
                    },
                    None => match ENDLOOP_RE.find(line_str) {
                        Some(_mat) => {
                            if triangle.len() != 3 {
                                return Err(StlError::Triangle(triangle.len(),
                                                          line_str.to_string()));
                            }

                            let mut real_triangle = Triangle::new();
                            
                            for (i, point) in triangle.iter().enumerate() {
                                real_triangle[i] = *point;
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

    Ok(surfaces)
}
