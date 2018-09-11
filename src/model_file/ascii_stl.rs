use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;

use regex::Regex;

pub type Point = [f64; 3];
pub type Points = Vec<Point>;
pub type Triangle = [Point; 3];
pub type Surface = Vec<Triangle>;
pub type Surfaces = Vec<Surface>;

pub trait New<T> {
    fn new() -> T;
}

impl New<Triangle> for Triangle {
    fn new() -> Triangle {
        [[0.0; 3]; 3]
    }
}

enum STLParseState {
    Top,
    Solid,
    Facet,
    Loop
}

pub fn load(fh : File) -> Result<Surfaces, Box<Error>> {
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
    
    let reader = BufReader::new(&fh);

    let mut surfaces = Surfaces::new();
    let mut surface = Surface::new();
    let mut surface_names : Vec<String> = Vec::new();
    let mut triangle = Points::new();
    
    let mut state = STLParseState::Top;

    for line in reader.lines() {
        let line_unwrapped = line.unwrap();
        let line_str = line_unwrapped.as_str();

        match state {
            STLParseState::Top => {
                let cap = match SOLID_RE.captures(line_str) {
                    Some(cap) => cap,
                    None => return Err(From::from("Expected solid")),
                };
                
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
                        None => return Err(From::from("Expected facet or endsolid")),
                        
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
                        None => return Err(From::from("Expected loop or endfacet"))
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
                                return Err(From::from("Triangle without three vertices"));
                            }

                            let mut real_triangle = Triangle::new();
                            
                            for (i, point) in triangle.iter().enumerate() {
                                real_triangle[i] = *point;
                            }
                            
                            surface.push(real_triangle);
                            state = STLParseState::Facet;
                        },
                        None => return Err(From::from("Expected vertex or endfacet"))
                    }
                };
            }
        }
    }

    Ok(surfaces)
}
