use hedge;
use hedge::Mesh;
use std::f64;
use hedge::Face;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range {
    pub min: f64,
    pub max: f64,
}

impl Range {
    pub fn new() -> Range {
        Range {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }

    pub fn len(&self) -> f64 {
        self.max - self.min
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds3D {
    pub x: Range,
    pub y: Range,
    pub z: Range,
}

pub fn mesh_bounds(mesh: &hedge::Mesh) -> Bounds3D {
    let mut bounds = Bounds3D {
        x: Range::new(),
        y: Range::new(),
        z: Range::new(),
    };
    for face in mesh.faces().map(|fi| mesh.face(fi)) {
        for point in mesh.vertices(face).map(|vi| mesh.vertex(vi).point) {
            if point[0] < bounds.x.min {
                bounds.x.min = point[0];
            }
            if point[0] > bounds.x.max {
                bounds.x.max = point[0];
            }
            if point[1] < bounds.y.min {
                bounds.y.min = point[1];
            }
            if point[1] > bounds.y.max {
                bounds.y.max = point[1];
            }
            if point[2] < bounds.z.min {
                bounds.z.min = point[2];
            }
            if point[2] > bounds.z.max {
                bounds.z.max = point[2];
            }
        }
    }

    bounds
}

pub fn z_range(mesh: &Mesh, face: &Face) -> Range {
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
    Range { min, max }
}

