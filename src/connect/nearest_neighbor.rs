
use crate::types::*;

use geo::coordinate_position::CoordPos;
use geo::prelude::BoundingRect;
use rstar;
use rstar::RTree;
use rstar::RTreeObject;
use rstar::primitives;
use std::rc::Rc;

type RPoint = [i64; 2];
type AABB = rstar::AABB<RPoint>;
type RRectangle = rstar::primitives::Rectangle<RPoint>;

trait ToRPoint {
    fn rpoint(self) -> RPoint;
}

impl ToRPoint for Coordinate {
    fn rpoint(self) -> RPoint {
        [self.x, self.y]
    }
}

trait ToRBox {
    fn rbox(self) -> AABB;
}

impl ToRBox for Rect {
    fn rbox(self) -> AABB {
        AABB::from_corners(self.min().rpoint(), self.max().rpoint())
    }
}

impl RTreeObject for Region {
    type Envelope = AABB;
    fn envelope(&self) -> Self::Envelope {
        self.poly.bounding_rect().unwrap().rbox()
    }
}

trait Traversable {
    fn traverse_from(&self, entry: RPoint, entry_id: usize) -> LineString;
    fn entry_points(&self) -> Vec<RPoint>; //TODO: this should be an iterator
    fn region_id(&self) -> u64;
    fn rank(&self) -> usize;

}

fn traversal_entries(traversable: &Rc<dyn Traversable>) -> Vec<TraversalEntry> {
    traversable.entry_points().iter().enumerate()
        .map(|(id, point)|
             TraversalEntry {
                 traversable: traversable.clone(),
                 entry_point: *point,
                 entry_id: id
             })
        .collect()
}


impl Traversable for SingleShell {
    fn traverse_from(&self, entry: RPoint, entry_id: usize) -> LineString {
        self.shell.clone() // TODO rotate around the entry point
    }

    fn entry_points(&self) -> Vec<RPoint> {
        self.shell.0.iter().map(|point| point.rpoint()).collect()
    }

    fn region_id(&self) -> u64 {
        self.region_id
    }

    fn rank(&self) -> usize {
        self.rank
    }
}

struct TraversalEntry {
    traversable: Rc<dyn Traversable>,
    entry_point: RPoint,
    entry_id: usize
}

impl RTreeObject for TraversalEntry {
    type Envelope = AABB;
    fn envelope(&self) -> Self::Envelope {
        self.entry_point.envelope()
    }
}
   

type RegionEntry = rstar::primitives::GeomWithData<Region, usize>;

/*trait ToEntry {
    fn entry(&self, idx: usize) -> RegionEntry;
}*/

/*impl ToEntry for Region {
    fn entry(&self, idx: usize) -> RegionEntry {
        let rect = self.poly.bounding_rect().unwrap();
        RegionEntry::new(RRectangle::from_corners(rect.min().rpoint(), rect.max().rpoint()), idx)
    }
}*/

pub fn connect(outlines: &LayerRegions, _interiors: &InteriorRegions, _solid_regions: &SolidRegions, _sparse_regions: &SparseRegions, shells: &LayerShells, _solid_fill: &MultiLineString, _sparse_fill: &MultiLineString) {
    let mut outline_regtree = RTree::<RegionEntry>::bulk_load(outlines.0.iter().enumerate().map(|(i, r)| RegionEntry::new(r.clone(), i)).collect());

    let traversables: Vec<Rc<dyn Traversable>> =
        shells.to_single_shells().into_iter().map(|shell| -> Rc<dyn Traversable> { Rc::new(shell) }).collect();

    let traversal_entries: Vec<TraversalEntry> = traversables.iter().map(|traversable| traversal_entries(traversable)).flatten().collect();
    
    let mut entrypoint_tree = RTree::<TraversalEntry>::bulk_load(traversal_entries);

    
    
}
