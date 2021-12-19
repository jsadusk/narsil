use crate::types::*;

pub fn integerize(
    float_polys: &geo::MultiLineString<f64>,
    resolution: f64,
) -> geo::MultiLineString<i64> {
    float_polys
        .0
        .iter()
        .map(|float_poly| {
            float_poly
                .0
                .iter()
                .map(|c| geo::Coordinate::<i64> {
                    x: (c.x / resolution) as i64,
                    y: (c.y / resolution) as i64,
                })
                .collect::<geo::LineString<i64>>()
        })
        .collect()
}

#[inline]
fn rotate_inner_int(x: i64, y: i64, x0: i64, y0: i64, sin_theta: f64, cos_theta: f64) -> Point {
    let x = x - x0;
    let y = y - y0;
    Point::new(
        (x as f64 * cos_theta - y as f64 * sin_theta + x0 as f64) as i64,
        (x as f64 * sin_theta + y as f64 * cos_theta + y0 as f64) as i64,
    )
}
fn rotate_many_int(
    angle: f64,
    origin: Point,
    points: impl Iterator<Item = Point>,
) -> impl Iterator<Item = Point> {
    let (sin_theta, cos_theta) = angle.to_radians().sin_cos();
    let (x0, y0) = origin.x_y();
    points.map(move |point| rotate_inner_int(point.x(), point.y(), x0, y0, sin_theta, cos_theta))
}

pub trait IntRotatePoint {
    fn rotate_around_point(&self, angle: f64, point: Point) -> Self;
}

impl IntRotatePoint for LineString {
    fn rotate_around_point(&self, angle: f64, point: Point) -> Self {
        rotate_many_int(angle, point, self.points_iter()).collect()
    }
}

impl IntRotatePoint for MultiLineString {
    fn rotate_around_point(&self, angle: f64, point: Point) -> Self {
        self.into_iter()
            .map(|ls: &LineString| -> LineString {
                rotate_many_int(angle, point, ls.points_iter()).collect()
            })
            .collect()
    }
}
