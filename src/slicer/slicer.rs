use hedge::FaceIndex;
use hedge::Mesh;
use quickersort;
use std::cmp::Eq;
use std::cmp::Ord;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::f64;

use crate::mesh::*;
use crate::slicer::error::*;

struct FaceAttrib {
    seen: bool,
}

impl FaceAttrib {
    fn new() -> FaceAttrib {
        FaceAttrib { seen: false }
    }
}

#[derive(PartialEq)]
pub struct FaceRange {
    face: FaceIndex,
    range: Range,
}

pub type Layer = geo::MultiLineString<f64>;
pub type LayerStack = Vec<Layer>;
struct Segment(geo::Coordinate<f64>, geo::Coordinate<f64>);
type FaceList = Vec<FaceIndex>;

impl Segment {
    fn new() -> Self {
        Segment(
            geo::Coordinate { x: 0.0, y: 0.0 },
            geo::Coordinate { x: 0.0, y: 0.0 },
        )
    }
}

const EPSILON: f64 = 0.0000001;

fn slice_face(position: f64, mesh: &Mesh, face_index: &FaceIndex) -> (Segment, FaceIndex) {
    let face = &mesh.face(*face_index);
    let mut seg = Segment::new();
    let mut next = *face_index;
    let mut oneset = false;
    let mut zeroset = false;

    for edge_index in mesh.edges(face) {
        let edge = &mesh.edge(edge_index);
        let mut point1 = mesh.vertex(edge.vertex_index).point.clone();
        let mut point2 = mesh
            .vertex(mesh.edge(edge.next_index).vertex_index)
            .point
            .clone();

        if point1[2] == position {
            point1[2] = point1[2] + EPSILON * 2.0;
        }

        if point2[2] == position {
            point2[2] = point2[2] + EPSILON * 2.0;
        }

        let (bottom, top) = if point1[2] < point2[2] {
            (point1, point2)
        } else {
            (point2, point1)
        };

        if position < bottom[2] || position > top[2] {
            continue;
        }

        let fraction = (position - bottom[2]) / (top[2] - bottom[2]);
        let intersect = geo::Coordinate::<f64> {
            x: bottom[0] + (top[0] - bottom[0]) * fraction,
            y: bottom[1] + (top[1] - bottom[1]) * fraction,
        };

        if point1[2] < point2[2] {
            assert!(!zeroset);
            zeroset = true;

            seg.0 = intersect;
        } else {
            let Range { min, max } =
                z_range(mesh, mesh.face(mesh.edge(edge.twin_index).face_index));
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

pub fn slice_layer(position: f64, mesh: &Mesh, starting_faces: &FaceList) -> SlicerResult<Layer> {
    //println!("Starting layer {}", position);
    let mut attrib = HashMap::new();
    for face in starting_faces.iter() {
        attrib.insert(face.clone(), FaceAttrib::new());
    }

    let mut starting_index: usize = 0;
    let mut layer = Vec::<geo::LineString<f64>>::new();

    while starting_index < starting_faces.len() {
        while starting_index < starting_faces.len()
            && attrib
                .get(&starting_faces[starting_index])
                .ok_or(SlicerError::CurrentFaceNoAttributes)?
                .seen
        {
            starting_index += 1;
        }
        if starting_index == starting_faces.len() {
            break;
        }

        let mut points = Vec::new();
        let starting_face = &starting_faces[starting_index];

        let (mut seg, mut next_face) = slice_face(position, &mesh, &starting_face);
        points.push(seg.0);
        points.push(seg.1);
        attrib.get_mut(starting_face).unwrap().seen = true;

        while next_face != *starting_face {
            let cur_face = next_face;
            let (new_seg, new_next_face) = slice_face(position, &mesh, &cur_face);
            seg = new_seg;
            next_face = new_next_face;
            if seg.0 == *points.last().ok_or(SlicerError::NoLastPointInSlice)? {
                points.push(seg.1);
            }
            attrib
                .get_mut(&cur_face)
                .ok_or(SlicerError::CurrentFaceNoAttributes)?
                .seen = true;
        }

        assert!(points.first() == points.last());
        layer.push(points.into());
    }

    Ok(layer.into_iter().collect())
}

#[derive(PartialEq)]
struct TopSortedFace {
    top: f64,
    face: FaceIndex,
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

pub fn sort_faces(mesh: &hedge::Mesh) -> Vec<FaceRange> {
    let mut bottom_sorted: Vec<FaceRange> = mesh
        .faces()
        .map(|fi| FaceRange {
            face: fi,
            range: z_range(&*mesh, mesh.face(fi)),
        })
        .collect();
    quickersort::sort_by(&mut bottom_sorted, &|a, b| {
        a.range.min.partial_cmp(&b.range.min).unwrap()
    });

    bottom_sorted
}

pub fn layer_faces(
    layer_height: f64,
    bounds: &Bounds3D,
    sorted_faces: &Vec<FaceRange>,
) -> Vec<(f64, Vec<FaceIndex>)> {
    let max_z = bounds.z.max;
    let num_layers: usize = (max_z / layer_height).round() as usize;
    let mut cur_face_iter = sorted_faces.iter().peekable();
    let mut valid_faces = BinaryHeap::new();
    let mut layers = Vec::new();
    for layer_id in 0..num_layers {
        let layer_position = (layer_id as f64) * layer_height + layer_height / 2.0;

        while cur_face_iter.peek() != None
            && cur_face_iter.peek().unwrap().range.min < layer_position
        {
            let facerange = cur_face_iter.next().unwrap();
            valid_faces.push(TopSortedFace {
                top: facerange.range.max,
                face: facerange.face,
            });
        }

        while !valid_faces.is_empty() && valid_faces.peek().unwrap().top < layer_position {
            valid_faces.pop();
        }

        if !valid_faces.is_empty() {
            let collected: Vec<FaceIndex> = valid_faces.iter().map(|f| f.face).collect();
            layers.push((layer_position, collected));
        }
    }

    layers
}
