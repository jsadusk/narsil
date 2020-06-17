use geo;

pub type MultiPolygon = geo::MultiPolygon<f64>;
pub type Polygon = geo::Polygon<f64>;
pub type LineString = geo::LineString<f64>;
pub type MultiLineString = geo::MultiLineString<f64>;
pub type Point = geo::Point<f64>;
pub type Coordinate = geo::Coordinate<f64>;

pub trait Push {
    type Item;
    fn push(&mut self, item: Self::Item);
}

impl Push for LineString {
    type Item = Coordinate;

    fn push(&mut self, item: Self::Item) {
        self.0.push(item);
    }
}
