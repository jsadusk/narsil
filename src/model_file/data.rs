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
