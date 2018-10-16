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
}

impl FaceAttrib {
    fn new() -> FaceAttrib {
        FaceAttrib { seen : false }
    }
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

fn slice_face(position : f64, mesh : &Mesh, face_index : &FaceIndex) -> (Segment, FaceIndex) {
    let face = &mesh.face(*face_index);
    let mut seg = Segment::new();
    let mut next = *face_index;
    
    for edge_index in mesh.edges(face) {
        let edge = &mesh.edge(edge_index);
        let point1 = mesh.vertex(edge.vertex_index).point;
        let point2 = mesh.vertex(mesh.edge(edge.next_index).vertex_index).point;

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
            seg.0 = intersect;
        }
        else {
            seg.1 = intersect;
            next = mesh.edge(edge.twin_index).face_index;
        }
            
    }

    (seg, next)
}

fn slice_layer(position : f64, mesh : &Mesh, starting_faces : FaceList) -> Layer {
    let mut attrib = HashMap::new();
    for face in starting_faces.iter() {
        attrib.insert(face.clone(), FaceAttrib::new());
    }

    let mut starting_index : usize = 0;
    let mut layer = Layer::new();
    while starting_index < starting_faces.len() {
        while starting_index < starting_faces.len() &&
              attrib.get(&starting_faces[starting_index]).unwrap().seen {
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
            if seg.0 == *slice.last().unwrap() {
                slice.push(seg.1);
            }
            attrib.get_mut(&cur_face).unwrap().seen = true;
        }

        layer.push(slice);
    }

    layer
}

pub fn slice(mesh : Mesh) -> SlicerResult<LayerStack>{
    let layer_height = 0.2;

    let mut max_z = f64::NEG_INFINITY;
    let mut min_z = f64::INFINITY;

    for index in mesh.faces() {
        let face = &mesh.face(index);
        let range = z_range(&mesh, face);
        if range.lower < min_z {
            min_z = range.lower;
        }
        if range.upper > max_z {
            max_z = range.upper;
        }
    }

    let num_layers : usize = (max_z / layer_height).round() as usize;

    let mut layers = LayerStack::new();
    for layer_id in 0..num_layers {
        let layer_position = (layer_id as f64) * layer_height + layer_height / 2.0;
        let mut valid_faces = FaceList::new();

        for index in mesh.faces() {
            let face = &mesh.face(index);
            let range =  z_range(&mesh, face);
            if range.lower < layer_position && range.upper > layer_position {
                valid_faces.push(index.clone());
            }
        }

        layers.push(slice_layer(layer_position, &mesh, valid_faces));
    }

    Ok(layers)
}
