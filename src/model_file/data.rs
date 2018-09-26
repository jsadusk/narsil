pub type Vertex = [f64; 3];
pub type Vertices = Vec<Vertex>;
pub type FreeTriangle = [Vertex; 3];
pub type FreeSurface = Vec<FreeTriangle>;
pub type IndexTriangle = [usize; 3];
pub type Surface = Vec<IndexTriangle>;

pub trait New<T> {
    fn new() -> T;
}

impl New<FreeTriangle> for FreeTriangle {
    fn new() -> FreeTriangle {
        [[0.0; 3]; 3]
    }
}

impl New<IndexTriangle> for IndexTriangle {
    fn new() -> IndexTriangle {
        [0; 3]
    }
}
