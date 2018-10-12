use hedge::Mesh;
use hedge::Face;
use hedge::FaceIndex;
use hedge::Point;
use std::collections::HashMap;
use std::f64;

#[derive(Fail, Debug)]
pub enum SlicerError {
    #[fail(display = "Model is not manifold")]
    NonManifold
}

type SlicerResult<T> = Result<T, SlicerError>;

struct FaceAttrib {
    seen : bool,
    index : FaceIndex
}

struct Range {
    lower: f64,
    upper: f64
}

fn z_range(mesh : &Mesh, face : &Face) -> Range {
    let mut upper = f64::NEG_INFINITY;
    let mut lower = f64::INFINITY;
    for index in mesh.vertices(face) {
        let point = &mesh.vertex(index).point;
        if point[2] < lower {
            lower = point[2];
        }
        if point[2] > upper {
            upper = point[2];
        }
    }
    Range {lower, upper}
}

type Polygon = Vec<Point>;
struct Segment(Point, Point);

impl Segment {
    fn new() -> Self {
        Segment(Point { x: 0.0, y: 0.0, z: 0.0 },
                Point { x: 0.0, y: 0.0, z: 0.0 })
    }
}

fn slice_face(position : f64, mesh : &Mesh, face_index : FaceIndex) -> (Segment, FaceIndex) {
    let face = &mesh.face(face_index);
    let mut seg = Segment::new();
    let mut next = face_index;
    
    for edge_index in mesh.edges(face) {
        let edge = &mesh.edge(edge_index);
        let point1 = mesh.vertex(edge.vertex_index).point;
        let point2 = mesh.vertex(mesh.edge(edge.next_index).vertex_index).point;

        let (bottom, top) = if point1[2] < point2[2]
            { (point1, point2) } else { (point2, point1) };

        if position < bottom[2] || position > top[2] {
            continue;
        }

        let fraction = position - bottom[2] / top[2] - bottom[2];
        let intersect = Point { x: bottom[0] + (top[0] - bottom[0]) * fraction,
                                y: bottom[1] + (top[1] - bottom[1]) * fraction,
                                z: position };

        if point1[2] > point2[2] {
            seg.0 = intersect;
        }
        else {
            seg.1 = intersect;
            next = mesh.edge(edge.twin_index).face_index;
        }
            
    }

    (seg, next)
}

fn slice_layer(position : f64, mesh : &Mesh, starting_face : FaceIndex) -> Polygon {
    let mut slice = Polygon::new();

    let (mut seg, mut next_face) = slice_face(position, &mesh, starting_face);
    slice.push(seg.0);
    slice.push(seg.1);

    while next_face != starting_face {
        let cur_face = next_face;
        let (new_seg, new_next_face) = slice_face(position, &mesh, cur_face);
        seg = new_seg;
        next_face = new_next_face;
        if seg.0 == *slice.last().unwrap() {
            slice.push(seg.1);
        }
    }

    slice
}

pub fn slice(mesh : Mesh) -> SlicerResult<Polygon>{
    let mut attribs = HashMap::new();

    for index in mesh.faces() {
        attribs.insert(index, FaceAttrib {seen: false, index: index});
    }

    let layer_position = 0.1;

    for index in mesh.faces() {
        let face = &mesh.face(index);
        let range =  z_range(&mesh, face);
        if range.lower < layer_position && range.upper > layer_position {
            return Ok(slice_layer(layer_position, &mesh, index));
        }
    }

    Ok(Polygon::new())
}
