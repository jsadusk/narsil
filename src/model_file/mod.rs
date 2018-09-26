pub mod ascii_stl;
pub mod binary_stl;
pub mod data;

use std::result::Result;
use std::io::prelude::*;
use std::fs::File;
use std::io::SeekFrom;
use std::io;
use std::collections::BTreeMap;
use std::cmp::Ordering;
use model_file::data::*;

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


fn dist_sq(a : &Vertex, b : &Vertex) -> f64 {
    (b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - b[2]).powi(2)
}

const EPS_FACTOR : f64 = 0.000001;

struct SortableVertex {
    data : Vertex,
    epsilon : f64
}

impl SortableVertex {
    fn new(data: &Vertex, epsilon: f64) -> SortableVertex {
        SortableVertex { data: [data[0], data[1], data[2]], epsilon }
    }
}

impl Ord for SortableVertex {
    fn cmp(&self, other: &SortableVertex) -> Ordering {
        for i in 0..2 {
            if (self.data[i] - other.data[i]).abs() > self.epsilon {
                if self.data[i] < other.data[i] {
                    return Ordering::Less;
                }
                else {
                    return Ordering::Greater;
                }
            }
        }
        Ordering::Equal
    }
}

impl PartialOrd for SortableVertex {
    fn partial_cmp(&self, other: &SortableVertex) -> Option<Ordering> {
        for i in 0..2 {
            if (self.data[i] - other.data[i]).abs() > self.epsilon {
                if self.data[i] < other.data[i] {
                    return Some(Ordering::Less);
                }
                else {
                    return Some(Ordering::Greater);
                }
            }
        }
        Some(Ordering::Equal)
    }
}

impl PartialEq for SortableVertex {
    fn eq(&self, other: &SortableVertex) -> bool {
        for i in 0..2 {
            if (self.data[i] - other.data[i]).abs() > self.epsilon {
                return false;
            }
        }
        true
    }
}

impl Eq for SortableVertex {}

fn unify_vertices(orig : FreeSurface) -> (Surface, Vertices) {
    //something to start
    let mut min_edge_len_sq = dist_sq(&orig[0][0], &orig[0][1]);

    for triangle in orig.iter() {
        let edge_len_sq = dist_sq(&triangle[0], &triangle[1]);
        if edge_len_sq < min_edge_len_sq {
            min_edge_len_sq = edge_len_sq;
        }

        let edge_len_sq = dist_sq(&triangle[1], &triangle[2]);
        if edge_len_sq < min_edge_len_sq {
            min_edge_len_sq = edge_len_sq;
        }

        let edge_len_sq = dist_sq(&triangle[2], &triangle[0]);
        if edge_len_sq < min_edge_len_sq {
            min_edge_len_sq = edge_len_sq;
        }
    }

    let epsilon = min_edge_len_sq.sqrt() / EPS_FACTOR;

    let mut vertices = Vertices::new();
    let mut surface = Surface::new();
    let mut vertex_map = BTreeMap::new();
    
    for free_triangle in orig.iter() {
        let mut indexed_triangle = IndexTriangle::new();
        for (i, vertex) in free_triangle.iter().enumerate() {
            let sortable = SortableVertex::new(vertex, epsilon);
            
            let index = vertex_map.entry(sortable).or_insert_with(|| {
                vertices.push(*vertex);
                vertices.len() - 1
            });
            indexed_triangle[i] = *index;
        }
        surface.push(indexed_triangle);
    }

    (surface, vertices)
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
    
pub fn load(mut fh : File) -> ModelResult<(Surface, Vertices)> {
    let file_type = identify(&mut fh).map_err(ModelError::IO)?;

    let result = match file_type {
        FileType::AsciiStl => unify_vertices(ascii_stl::load(fh).map_err(ModelError::AsciiParse)?),
        FileType::BinaryStl => unify_vertices(binary_stl::load(fh).map_err(ModelError::BinaryParse)?),
        FileType::Unknown => return Err(ModelError::Unknown)
    };
    Ok(result)
} 
