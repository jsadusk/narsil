use geo;

use crate::id_factory::*;
use geo_clipper::{ClosedPoly, OwnedPolygon, PolyType, ToOwnedPolygonInt};
use iter;
use std::cmp;

pub type MultiPolygon = geo::MultiPolygon<i64>;
pub type Polygon = geo::Polygon<i64>;
pub type LineString = geo::LineString<i64>;
pub type MultiLineString = geo::MultiLineString<i64>;
pub type Point = geo::Point<i64>;
pub type Coordinate = geo::Coordinate<i64>;
pub type Line = geo::Line<i64>;
pub type Rect = geo::Rect<i64>;

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

pub struct Region {
    pub poly: Polygon,
    pub id: u64,
}

impl IntoLineStrings for Region {
    type Iter = impl Iterator<Item = LineString>;
    fn into_line_strings(self) -> Self::Iter {
        self.poly.into_line_strings()
    }
}

impl From<Polygon> for Region {
    fn from(poly: Polygon) -> Region {
        Region {
            poly,
            id: get_next_region_id(),
        }
    }
}

pub trait CenterInt {
    fn center(&self) -> Point;
}

impl CenterInt for Rect {
    fn center(&self) -> Point {
        (
            (self.max().x + self.min().x) / 2,
            (self.max().y + self.min().y) / 2,
        )
            .into()
    }
}

#[derive(PartialEq, Eq)]
pub enum PathTag {
    Region,
    Shell,
    Interior,
    Solid,
    Sparse,
    Unknown,
}

pub trait RegionTag {
    const PATHTAG: PathTag;
}

pub struct OutlineRegionTag {}
pub struct InteriorRegionTag {}
pub struct SolidRegionTag {}
pub struct SparseRegionTag {}
pub struct UnknownRegionTag {}

impl RegionTag for OutlineRegionTag {
    const PATHTAG: PathTag = PathTag::Region;
}
impl RegionTag for InteriorRegionTag {
    const PATHTAG: PathTag = PathTag::Interior;
}
impl RegionTag for SolidRegionTag {
    const PATHTAG: PathTag = PathTag::Solid;
}
impl RegionTag for SparseRegionTag {
    const PATHTAG: PathTag = PathTag::Sparse;
}

pub struct TaggedRegions<Tag: RegionTag>(pub Vec<Region>, std::marker::PhantomData<Tag>);

impl<Tag: RegionTag> TaggedRegions<Tag> {
    pub fn new(regions: Vec<Region>) -> Self {
        Self(regions, std::marker::PhantomData)
    }
}

pub type LayerRegions = TaggedRegions<OutlineRegionTag>;
pub type InteriorRegions = TaggedRegions<InteriorRegionTag>;
pub type SolidRegions = TaggedRegions<SolidRegionTag>;
pub type SparseRegions = TaggedRegions<SparseRegionTag>;

/*impl LayerRegions {
    fn new(regions: Vec<Region>) -> Self {
        Self(regions)
    }
}*/

impl<Tag: RegionTag> From<MultiPolygon> for TaggedRegions<Tag> {
    fn from(polys: MultiPolygon) -> Self {
        Self::new(polys.into_iter().map(|p| p.into()).collect())
    }
}

/*impl From<MultiPolygon> for LayerRegions {
    fn from(polys: MultiPolygon) -> LayerRegions {
        LayerRegions(polys.into_iter().map(|p| p.into()).collect())
    }
}*/

pub struct Shells {
    pub shells: Vec<MultiLineString>,
    pub region_id: u64,
}

pub struct LayerShells(pub Vec<Shells>);

impl<Tag: RegionTag> ToOwnedPolygonInt for TaggedRegions<Tag> {
    fn to_polygon_owned(&self, poly_type: geo_clipper::PolyType) -> geo_clipper::OwnedPolygon {
        let mut owned = OwnedPolygon {
            polygons: Vec::with_capacity(self.0.len()),
            paths: Vec::with_capacity(self.0.len()),
            vertices: Vec::with_capacity(self.0.len()),
        };

        for region in self.0.iter() {
            owned = owned.add_polygon_int(&region.poly, poly_type);
        }
        owned
    }
}

impl<Tag: RegionTag> ClosedPoly for TaggedRegions<Tag> {}

pub struct TaggedPath {
    pub tag: PathTag,
    pub path: LineString,
}

impl From<Region> for Vec<TaggedPath> {
    fn from(region: Region) -> Self {
        region
            .into_line_strings()
            .map(|path| TaggedPath {
                tag: PathTag::Region,
                path,
            })
            .collect()
    }
}

impl<Tag: RegionTag> From<TaggedRegions<Tag>> for Vec<TaggedPath> {
    fn from(regions: TaggedRegions<Tag>) -> Self {
        regions
            .0
            .into_iter()
            .map(|region| {
                region.into_line_strings().map(|path| TaggedPath {
                    tag: Tag::PATHTAG,
                    path,
                })
            })
            .flatten()
            .collect()
    }
}

impl From<LayerRegions> for InteriorRegions {
    fn from(layer_regions: LayerRegions) -> Self {
        InteriorRegions::new(layer_regions.0)
    }
}

impl From<Shells> for Vec<TaggedPath> {
    fn from(shells: Shells) -> Self {
        shells
            .shells
            .into_iter()
            .map(|paths| {
                paths.into_iter().map(|path| TaggedPath {
                    tag: PathTag::Shell,
                    path,
                })
            })
            .flatten()
            .collect()
    }
}

impl From<LayerShells> for Vec<TaggedPath> {
    fn from(layer_shells: LayerShells) -> Self {
        layer_shells
            .0
            .into_iter()
            .map(|shells| -> Vec<TaggedPath> { shells.into() })
            .flatten()
            .collect()
    }
}

pub trait RegionPolyOp {
    fn apply_mult(&self, func: impl Fn(&Polygon) -> MultiPolygon) -> Self;
    //fn apply_mult_into(self, func: impl Fn(Polygon) -> MultiPolygon) -> Self;
}

impl<Tag: RegionTag> RegionPolyOp for TaggedRegions<Tag> {
    fn apply_mult(&self, func: impl Fn(&Polygon) -> MultiPolygon) -> Self {
        TaggedRegions::<Tag>::new(
            self.0
                .iter()
                .map(|region| {
                    let id = region.id;
                    func(&region.poly)
                        .0
                        .into_iter()
                        .map(move |poly| -> Region { Region { poly, id } })
                })
                .flatten()
                .collect(),
        )
    }
}

pub trait MyDefault {
    fn default() -> Self;
}

impl MyDefault for MultiPolygon {
    fn default() -> MultiPolygon {
        geo::MultiPolygon::<i64>(Vec::new())
    }
}

impl MyDefault for MultiLineString {
    fn default() -> MultiLineString {
        geo::MultiLineString::<i64>(Vec::new())
    }
}

pub trait BoundOps {
    fn bound_sum(&self, other: &Self) -> Self;
}

impl BoundOps for Rect {
    fn bound_sum(&self, other: &Rect) -> Rect {
        let amin = self.min();
        let bmin = other.min();
        let amax = self.max();
        let bmax = other.max();
        Rect::new(
            Coordinate {
                x: cmp::min(amin.x, bmin.x),
                y: cmp::min(amin.y, bmin.y),
            },
            Coordinate {
                x: cmp::max(amax.x, bmax.x),
                y: cmp::max(amax.y, bmax.y),
            },
        )
    }
}

pub enum FillPattern {
    Linear,
}

pub struct PathGroup<Tag: RegionTag> {
    pub lines: MultiLineString,
    pub region_id: u64,
    tag: std::marker::PhantomData<Tag>,
}
