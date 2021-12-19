use std::iter;
use geom::*;
use aliases::*;

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

impl Push for MultiLineString {
    type Item = LineString;

    fn push(&mut self, item: Self::Item) {
        self.0.push(item);
    }
}

pub trait IntoLineStrings {
    type Iter: Iterator<Item = LineString>;
    fn into_line_strings(self) -> Self::Iter;
}

impl IntoLineStrings for Polygon {
    type Iter = std::iter::Chain<
        std::iter::Once<geo::LineString<i64>>,
        std::vec::IntoIter<geo::LineString<i64>>,
    >;
    fn into_line_strings(self) -> Self::Iter {
        let (exterior, interior) = self.into_inner();
        iter::once(exterior).chain(interior.into_iter())
    }
}

impl IntoLineStrings for MultiPolygon {
    type Iter = impl Iterator<Item = LineString>;
    fn into_line_strings(self) -> Self::Iter {
        self.0
            .into_iter()
            .map(|poly| poly.into_line_strings())
            .flatten()
    }
}

impl IntoLineStrings for Region {
    type Iter = impl Iterator<Item = LineString>;
    fn into_line_strings(self) -> Self::Iter {
        self.poly.into_line_strings()
    }
}

