use geo;

use crate::id_factory::*;
use geo_clipper::{ClosedPoly, OwnedPolygon, ToOwnedPolygonInt};
use std::cmp;
use types::traits::IntoLineStrings;

use aliases::*;

#[derive(Clone)]
pub struct Region {
    pub poly: Polygon,
    pub id: u64,
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


impl From<Polygon> for Region {
    fn from(poly: Polygon) -> Region {
        Region {
            poly,
            id: get_next_region_id(),
        }
    }
}

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

pub struct SingleShell {
    pub shell: LineString,
    pub region_id: u64,
    pub rank: usize,
}

impl Shells {
    fn to_single_shells(&self) -> Vec<SingleShell> {
        self.shells.iter()
            .enumerate()
            .map(|(rank, shells)|
                 shells.iter().map(|shell|
                            SingleShell {
                                shell: shell.clone(),
                                region_id: self.region_id,
                                rank: rank
                            }).collect::<Vec<SingleShell>>())
            .flatten()
            .collect()
    }
}
                

pub struct LayerShells(pub Vec<Shells>);

impl LayerShells {
    pub fn to_single_shells(&self) -> Vec<SingleShell> {
        self.0.iter().map(|shells| shells.to_single_shells()).flatten().collect()
    }
}

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

