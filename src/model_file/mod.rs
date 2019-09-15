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
use std::error;
use std::fmt;
use model_file::data::*;
use hedge;
use hedge::Mesh;
use hedge::AddGeometry;

use expression::*;

use crate::error::NarsilError;

#[derive(Debug)]
pub enum ModelError {
    IO(io::Error),
    AsciiParse(ascii_stl::StlError),
    BinaryParse(binary_stl::StlError),
    Unknown
}

impl error::Error for ModelError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::IO(e) => Some(e),
            Self::AsciiParse(e) => Some(e),
            Self::BinaryParse(e) => Some(e),
            Self::Unknown => None
        }
    }
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(e) => write!(f, "{}", e),
            Self::AsciiParse(e) => write!(f, "{}", e),
            Self::BinaryParse(e) => write!(f, "{}", e),
            Self::Unknown => write!(f, "Unknown file format")
        }
    }
}

impl From<io::Error> for ModelError {
    fn from(e: io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<ascii_stl::StlError> for ModelError {
    fn from(e: ascii_stl::StlError) -> Self {
        Self::AsciiParse(e)
    }
}

impl From<binary_stl::StlError> for ModelError {
    fn from(e: binary_stl::StlError) -> Self {
        Self::BinaryParse(e)
    }
}

fn dist_sq(a : &Vertex, b : &Vertex) -> f64 {
    ((*b)[0] - (*a)[0]).powi(2) + ((*b)[1] - (*a)[1]).powi(2) + ((*b)[2] - (*a)[2]).powi(2)
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
        for i in 0..3 {
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

fn unify_vertices(orig : &FreeSurface) -> (Surface, Vertices) {
    //something to start
    let mut min_edge_len_sq = dist_sq(&orig[0][0], &orig[0][1]);

    for (i, triangle) in orig.iter().enumerate() {
        let edge_len_sq = dist_sq(&triangle[0], &triangle[1]);
        if edge_len_sq == 0.0 {
            panic!("Degenerate 0 {} edge {},{},{} {},{},{}", i,
                   triangle[0][0], triangle[0][1], triangle[0][2],
                   triangle[1][0], triangle[1][1], triangle[1][2]);
        }
        if edge_len_sq < min_edge_len_sq {
            min_edge_len_sq = edge_len_sq;
        }

        let edge_len_sq = dist_sq(&triangle[1], &triangle[2]);
        if edge_len_sq == 0.0 {
            panic!("Degenerate 1 {} edge {},{},{} {},{},{}", i,
                   triangle[1][0], triangle[1][1], triangle[1][2],
                   triangle[2][0], triangle[2][1], triangle[2][2]);
        }
        if edge_len_sq < min_edge_len_sq {
            min_edge_len_sq = edge_len_sq;
        }

        let edge_len_sq = dist_sq(&triangle[2], &triangle[0]);
        if edge_len_sq == 0.0 {
            panic!("Degenerate 2 {} edge {},{},{} {},{},{}", i,
                   triangle[0][0], triangle[0][1], triangle[0][2],
                   triangle[2][0], triangle[2][1], triangle[2][2]);
        }
        if edge_len_sq < min_edge_len_sq {
            min_edge_len_sq = edge_len_sq;
        }
    }

    let epsilon = min_edge_len_sq.sqrt() * EPS_FACTOR;

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

pub enum FileType {
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

trait FromSurface {
    fn from_surface(surface : Surface, vertices : Vertices) -> Self;
}

impl FromSurface for Mesh {
    fn from_surface(surface : Surface, vertices : Vertices) -> Self {
        let mut mesh = Mesh::new();

        let mut vert_indices = Vec::new();

        for mesh_vert in vertices.iter().map(
            |vert| hedge::Vertex::from_point(hedge::Point {x : vert[0],
                                                           y : vert[1],
                                                           z : vert[2]})) {
            vert_indices.push(mesh.add(mesh_vert));
        }

        for triangle in surface {
            mesh.add(hedge::triangle::FromVerts(vert_indices[triangle[0]],
                                                vert_indices[triangle[1]],
                                                vert_indices[triangle[2]]));
        }

        mesh
    }
}

type ModelResult<T> = Result<T, ModelError>;

pub fn load(mut fh: File) -> ModelResult<Mesh> {
    println!("Identify");
    let file_type = identify(&mut fh).map_err(ModelError::IO)?;

    println!("load");
    let free_mesh = match file_type {
        FileType::AsciiStl => ascii_stl::load(&fh)?,
        FileType::BinaryStl => binary_stl::load(&fh)?,
        FileType::Unknown => return Err(ModelError::Unknown)
    };

    println!("unify");
    let (surface, vertices) = unify_vertices(&free_mesh);

    println!("mesh");
    Ok(Mesh::from_surface(surface, vertices))
}

pub struct IdentifyModelType {
    pub fh: File
}

impl Expression<FileType, NarsilError> for IdentifyModelType {
    fn terms(&self) -> Terms { Terms::new() }

    fn eval(&self) -> Result<FileType, NarsilError> {
        let mut fh = self.fh.try_clone()?;
        identify(&mut fh).map_err(|e| NarsilError::Model(ModelError::IO(e)))
    }
}

pub struct LoadTriangles {
    pub fh: File,
    pub ft: TypedTerm<FileType>
}

impl Expression<FreeSurface, NarsilError> for LoadTriangles {
    fn terms(&self) -> Terms {
        vec!(self.ft.term())
    }

    fn eval(&self) -> Result<FreeSurface, NarsilError> {
        match *self.ft {
            FileType::AsciiStl => ascii_stl::load(&self.fh).map_err(|e|NarsilError::Model(ModelError::AsciiParse(e))),
            FileType::BinaryStl => binary_stl::load(&self.fh).map_err(|e|NarsilError::Model(ModelError::BinaryParse(e))),
            FileType::Unknown => return Err(NarsilError::Model(ModelError::Unknown))
        }
    }
}

pub struct UnifiedTriangles {
    surface: Surface,
    vertices: Vertices
}

pub struct UnifyVertices {
    pub free_mesh: TypedTerm<FreeSurface>
}

impl Expression<UnifiedTriangles, NarsilError> for UnifyVertices {
    fn terms(&self) -> Terms {
        vec!(self.free_mesh.term())
    }

    fn eval(&self) -> Result<UnifiedTriangles, NarsilError> {
        let (surface, vertices) = unify_vertices(&*self.free_mesh);
        Ok(UnifiedTriangles { surface: surface, vertices: vertices })
    }
}

pub struct ConnectedMesh {
    pub unified_triangles: TypedTerm<UnifiedTriangles>
}

impl Expression<Mesh, NarsilError> for ConnectedMesh {
    fn terms(&self) -> Terms {
        vec!(self.unified_triangles.term())
    }

    fn eval(&self) -> Result<Mesh, NarsilError> {
        Ok(Mesh::from_surface(self.unified_triangles.surface.clone(), self.unified_triangles.vertices.clone()))
    }
}

pub struct LoadModel {
    pub fh: File
}

impl Expression<Mesh, NarsilError> for LoadModel {
    fn terms(&self) -> Terms {Terms::new() }

    fn eval(&self) -> Result<Mesh, NarsilError> {
        let fh = self.fh.try_clone()?;
        load(fh).map_err(|e| NarsilError::Model(e))
    }
}
