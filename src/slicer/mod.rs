use hedge::Mesh;
use hedge::Face;
use hedge::FaceIndex;
use hedge::Point;
use std::collections::HashMap;
use std::f64;
use quickersort;
use std::collections::BinaryHeap;
use std::cmp::Ord;
use std::cmp::Ordering;
use std::cmp::Eq;
use std::error;
use std::fmt;
use rayon::prelude::*;

use expression::*;

use crate::mesh::{Bounds3D, Range};

#[derive(Debug)]
pub enum SlicerError {
    NonManifold,
    StartingFaceNoAttributes,
    NoLastPointInSlice,
    CurrentFaceNoAttributes
}

impl error::Error for SlicerError {}

impl fmt::Display for SlicerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonManifold => write!(f, "Model is not manifold"),
            Self::StartingFaceNoAttributes =>
                write!(f, "Starting face is not in attributes map"),
            Self::NoLastPointInSlice => write!(f, "No last point in slice"),
            Self::CurrentFaceNoAttributes =>
                write!(f, "Current face is not in attributes map")
        }
    }
}

type SlicerResult<T> = Result<T, SlicerError>;

struct FaceAttrib {
    seen : bool,
}

impl FaceAttrib {
    fn new() -> FaceAttrib {
        FaceAttrib { seen : false }
    }
}

#[derive(PartialEq)]
pub struct FaceRange {
    face : FaceIndex,
    range: Range
}

fn z_range(mesh : &Mesh, face : &Face) -> Range {
    let mut max = f64::NEG_INFINITY;
    let mut min = f64::INFINITY;
    for index in mesh.vertices(face) {
        let point = &mesh.vertex(index).point;
        if point[2] < min {
            min = point[2];
        }
        if point[2] > max {
            max = point[2];
        }
    }
    Range {min, max}
}

pub type Polygon = Vec<Point>;
pub type Layer = Vec<Polygon>;
pub type LayerStack = Vec<Layer>;
struct Segment(Point, Point);
type FaceList = Vec<FaceIndex>;

impl Segment {
    fn new() -> Self {
        Segment(Point { x: 0.0, y: 0.0, z: 0.0 },
                Point { x: 0.0, y: 0.0, z: 0.0 })
    }
}

const EPSILON : f64 = 0.0000001;

fn slice_face(position : f64, mesh : &Mesh, face_index : &FaceIndex) -> (Segment, FaceIndex) {
    let face = &mesh.face(*face_index);
    let mut seg = Segment::new();
    let mut next = *face_index;
    let mut oneset = false;
    let mut zeroset = false;

    for edge_index in mesh.edges(face) {
        let edge = &mesh.edge(edge_index);
        let mut point1 = mesh.vertex(edge.vertex_index).point.clone();
        let mut point2 = mesh.vertex(mesh.edge(edge.next_index).vertex_index).point.clone();

        if point1[2] == position {
            point1[2] = point1[2] + EPSILON * 2.0;
        }

        if point2[2] == position {
            point2[2] = point2[2] + EPSILON * 2.0;
        }

        let (bottom, top) = if point1[2] < point2[2]
            { (point1, point2) } else { (point2, point1) };

        if position < bottom[2] || position > top[2] {
            continue;
        }

        let fraction = (position - bottom[2]) / (top[2] - bottom[2]);
        let intersect = Point { x: bottom[0] + (top[0] - bottom[0]) * fraction,
                                y: bottom[1] + (top[1] - bottom[1]) * fraction,
                                z: position };

        if point1[2] < point2[2] {
            assert!(!zeroset);
            zeroset = true;

            seg.0 = intersect;
        }
        else {
            let Range {min, max} = z_range(mesh, mesh.face(mesh.edge(edge.twin_index).face_index));
            if position > min && position <= max {
                if oneset {
                    println!("seg.1 was already set");
                }
                oneset = true;
                seg.1 = intersect;
                next = mesh.edge(edge.twin_index).face_index;
            }
        }
    }

    assert!(oneset);
    assert!(zeroset);

    (seg, next)
}

fn slice_layer(position : f64, mesh : &Mesh, starting_faces : &FaceList) -> SlicerResult<Layer> {
    //println!("Starting layer {}", position);
    let mut attrib = HashMap::new();
    for face in starting_faces.iter() {
        attrib.insert(face.clone(), FaceAttrib::new());
    }

    let mut starting_index : usize = 0;
    let mut layer = Layer::new();

    while starting_index < starting_faces.len() {
        while starting_index < starting_faces.len() &&
              attrib.get(&starting_faces[starting_index]).ok_or(SlicerError::CurrentFaceNoAttributes)?.seen {
                  starting_index += 1;
              }
        if starting_index == starting_faces.len() {
            break;
        }

        let mut slice = Polygon::new();
        let starting_face = &starting_faces[starting_index];

        let (mut seg, mut next_face) = slice_face(position, &mesh, &starting_face);
        slice.push(seg.0);
        slice.push(seg.1);
        attrib.get_mut(starting_face).unwrap().seen = true;

        while next_face != *starting_face {
            let cur_face = next_face;
            let (new_seg, new_next_face) = slice_face(position, &mesh, &cur_face);
            seg = new_seg;
            next_face = new_next_face;
            if seg.0 == *slice.last().ok_or(SlicerError::NoLastPointInSlice)? {
                slice.push(seg.1);
            }
            attrib.get_mut(&cur_face).ok_or(SlicerError::CurrentFaceNoAttributes)?.seen = true;
        }

        assert!(slice.first() == slice.last());
        layer.push(slice);
    }

    Ok(layer)
}

#[derive(PartialEq)]
struct TopSortedFace {
    top : f64,
    face : FaceIndex
}

impl Eq for TopSortedFace {}

impl Ord for TopSortedFace {
    fn cmp(&self, other: &TopSortedFace) -> Ordering {
        return other.top.partial_cmp(&self.top).unwrap();
    }
}

impl PartialOrd for TopSortedFace {
    fn partial_cmp(&self, other: &TopSortedFace) -> Option<Ordering> {
        return other.top.partial_cmp(&self.top);
    }
}

pub struct SliceMesh<M, B> {
    pub mesh: TermResult<M>,
    pub bounds: TermResult<B>
}

impl<M, B> Expression for SliceMesh<M, B>
where
    M: TypedTerm<ValueType=Mesh>,
    B: TypedTerm<ValueType=Bounds3D>
{
    type ValueType = LayerStack;
    type ErrorType = SlicerError;

    fn terms(&self) -> Terms {
        vec!(self.mesh.term(), self.bounds.term())
    }

    fn eval(&self) -> SlicerResult<LayerStack> {
        let layer_height = 0.2;

        let max_z = self.bounds.z.max;
        let mesh = &*self.mesh;

        let num_layers : usize = (max_z / layer_height).round() as usize;
        println!("Sort");
        let mut bottom_sorted : Vec<FaceRange> =
            mesh.faces().
            map(|fi| FaceRange{face: fi, range: z_range(&mesh, mesh.face(fi))}).
            collect();
        quickersort::sort_by(&mut bottom_sorted,
                             &|a, b| a.range.min.partial_cmp(&b.range.min).unwrap());

        let mut cur_face_iter = bottom_sorted.iter().peekable();
        let mut valid_faces = BinaryHeap::new();

        println!("Valid faces");
        let mut layers = Vec::new();
        for layer_id in 0..num_layers {
            let layer_position = (layer_id as f64) * layer_height + layer_height / 2.0;

            while cur_face_iter.peek() != None &&
                cur_face_iter.peek().unwrap().range.min < layer_position {
                    let facerange = cur_face_iter.next().unwrap();
                    valid_faces.push(TopSortedFace{top : facerange.range.max,
                                                   face : facerange.face});
                }

            while !valid_faces.is_empty() && valid_faces.peek().unwrap().top < layer_position {
                valid_faces.pop();
            }

            if !valid_faces.is_empty() {
                let collected : Vec<FaceIndex> =
                    valid_faces.iter().map(|f|f.face).collect();
                layers.push((layer_position, collected));
            }
        }

        println!("Parallel slice");
        let layer_results : Vec<SlicerResult<Layer>> =
            layers.par_iter().map(|l| slice_layer(l.0, &mesh, &l.1)).collect();

        let mut layers = Vec::new();

        for layer_result in layer_results {
            layers.push(layer_result?);
        }

        Ok(layers)
    }
}

pub struct SortedFaces<M> {
    pub mesh: TermResult<M>
}

impl<M> Expression for SortedFaces<M>
where M: TypedTerm<ValueType=Mesh>
{
    type ValueType = Vec<FaceRange>;
    type ErrorType = SlicerError;

    fn terms(&self) -> Terms {
        vec!(self.mesh.term())
    }

    fn eval(&self) -> SlicerResult<Vec<FaceRange>> {
        let mut bottom_sorted : Vec<FaceRange> =
            self.mesh.faces().
            map(|fi| FaceRange{face: fi, range: z_range(&*self.mesh, self.mesh.face(fi))}).
            collect();
        quickersort::sort_by(&mut bottom_sorted,
                             &|a, b| a.range.min.partial_cmp(&b.range.min).unwrap());
        Ok(bottom_sorted)
    }
}

pub struct LayerFaces<M, B, FR> {
    pub mesh: TermResult<M>,
    pub bounds: TermResult<B>,
    pub sorted_faces: TermResult<FR>
}

impl<M, B, FR> Expression for LayerFaces<M, B, FR>
where
    M: TypedTerm<ValueType=Mesh>,
    B: TypedTerm<ValueType=Bounds3D>,
    FR: TypedTerm<ValueType=Vec<FaceRange>>
{
    type ValueType = Vec<(f64, Vec<FaceIndex>)>;
    type ErrorType = SlicerError;

    fn terms(&self) -> Terms {
        vec!(self.mesh.term(), self.bounds.term(), self.sorted_faces.term())
    }

    fn eval(&self) -> SlicerResult<Vec<(f64, Vec<FaceIndex>)>> {
        let layer_height = 0.2;

        let max_z = self.bounds.z.max;
        let num_layers : usize = (max_z / layer_height).round() as usize;
        let mut cur_face_iter = self.sorted_faces.iter().peekable();
        let mut valid_faces = BinaryHeap::new();
        let mut layers = Vec::new();
        for layer_id in 0..num_layers {
            let layer_position = (layer_id as f64) * layer_height + layer_height / 2.0;

            while cur_face_iter.peek() != None &&
                cur_face_iter.peek().unwrap().range.min < layer_position {
                    let facerange = cur_face_iter.next().unwrap();
                    valid_faces.push(TopSortedFace{top : facerange.range.max,
                                                   face : facerange.face});
                }

            while !valid_faces.is_empty() && valid_faces.peek().unwrap().top < layer_position {
                valid_faces.pop();
            }

            if !valid_faces.is_empty() {
                let collected : Vec<FaceIndex> =
                    valid_faces.iter().map(|f|f.face).collect();
                layers.push((layer_position, collected));
            }
        }

        Ok(layers)
    }
}

pub struct SliceFaces<M, FI> {
    pub mesh: TermResult<M>,
    pub layer_faces: TermResult<FI>
}

impl<M, FI> Expression for SliceFaces<M, FI>
where
    M: TypedTerm<ValueType=Mesh>,
    FI: TypedTerm<ValueType=Vec<(f64, Vec<FaceIndex>)>>
{
    type ValueType = LayerStack;
    type ErrorType = SlicerError;

    fn terms(&self) -> Terms {
        vec!(self.mesh.term(), self.layer_faces.term())
    }

    fn eval(&self) -> SlicerResult<LayerStack> {
        let layer_faces = &*self.layer_faces;
        let layer_results : Vec<SlicerResult<Layer>> =
            layer_faces.iter().map(|l| slice_layer(l.0, &*self.mesh, &l.1)).collect();

        let mut layers = Vec::new();

        for layer_result in layer_results {
            layers.push(layer_result?);
        }

        Ok(layers)
    }
}
