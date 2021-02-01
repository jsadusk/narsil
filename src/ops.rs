use crate::captures::*;
use crate::config::*;
use crate::infill::*;
use crate::types::*;
use geo::MultiLineString;
use geo_clipper::*;
use iter;
use simple_generator::Generator;
use simple_generator::*;
use std::collections::VecDeque;

pub fn adjacent_gen<'a>(
    i: impl Iterator<Item = &'a MultiPolygon> + Clone + Send,
) -> impl Generator<Item = (MultiPolygon, MultiPolygon)> {
    GeneratorFunc::new(move || {
        let mut i = i.clone();
        let mut last = i.next().unwrap().clone();
        move || {
            let cur = i.next()?;
            let result = (last.clone(), cur.clone());
            last = cur.clone();
            Some(result)
        }
    })
}

pub fn difference_op() -> impl Fn((MultiPolygon, MultiPolygon)) -> MultiPolygon {
    move |(a, b): (MultiPolygon, MultiPolygon)| a.difference(&b)
}

pub fn intersect_open_op(
) -> impl Fn((MultiPolygon, crate::MultiLineString)) -> crate::MultiLineString {
    move |(a, b): (MultiPolygon, crate::MultiLineString)| b.intersection(&a)
}

pub fn offset_op(delta: f64) -> impl Fn(MultiPolygon) -> MultiPolygon {
    move |p: MultiPolygon| p.offset(delta, JoinType::Miter(3.0), EndType::ClosedPolygon)
}

pub fn shells_op(config: &Config) -> impl Fn(&LayerRegions) -> LayerShells {
    let num_shells = config.num_shells;
    let initial = -config.nozzle_diameter_dsc() / 2;
    let per_shell = config.nozzle_diameter_dsc() - config.shell_overlap_dsc();

    move |layer_regions: &LayerRegions| {
        let mut layer_shells = Vec::new();
        for region in layer_regions.0.iter() {
            let mut region_shells = Vec::new();

            for i in 0..num_shells {
                let offset_dist = initial - i as i64 * per_shell;
                let shell_polys = region.poly.offset(
                    offset_dist as f64,
                    JoinType::Miter(3.0),
                    EndType::ClosedPolygon,
                );
                let mut rank_shells = Vec::new();
                for shell_poly in shell_polys {
                    let (exterior, interior) = shell_poly.into_inner();
                    rank_shells.extend(iter::once(exterior).chain(interior.into_iter()));
                }
                region_shells.push(MultiLineString(rank_shells));
            }

            layer_shells.push(Shells {
                shells: region_shells,
                region_id: region.id,
            });
        }

        LayerShells(layer_shells)
    }
}

pub fn interiors_op(config: &Config) -> impl Fn(&LayerRegions) -> InteriorRegions {
    let delta = config.interior_offset_dsc() as f64;

    move |layer_regions: &LayerRegions| {
        layer_regions
            .apply_mult(|poly| poly.offset(delta, JoinType::Miter(3.0), EndType::ClosedPolygon))
            .into()
    }
}

pub fn solid_grouping_gen<'a>(
    config: &Config,
    top: impl Iterator<Item = &'a MultiPolygon> + Clone + Send,
    bottom: impl Iterator<Item = &'a MultiPolygon> + Clone + Send,
) -> impl Generator<Item = MultiPolygon> + Captures<'a> {
    let num_top_layers = config.num_top_layers();
    let num_bottom_layers = config.num_top_layers();

    GeneratorFunc::new(move || {
        let mut cur_top = top.clone();
        let mut cur_bottom = bottom.clone();

        let mut top_accum = VecDeque::<&'a MultiPolygon>::with_capacity(num_top_layers);
        let mut bottom_accum = VecDeque::<&'a MultiPolygon>::with_capacity(num_bottom_layers);

        for i in 0..(num_top_layers - 1) {
            match cur_top.next() {
                Some(next_top) => top_accum.push_back(next_top),
                None => break,
            }
        }

        move || {
            match cur_top.next() {
                Some(next_top) => {
                    top_accum.push_back(next_top);
                    if top_accum.len() > num_top_layers {
                        top_accum.pop_front();
                    }
                }
                None => {}
            }

            match cur_bottom.next() {
                Some(next_bottom) => {
                    bottom_accum.push_back(next_bottom);
                    if bottom_accum.len() > num_bottom_layers {
                        bottom_accum.pop_front();
                    }

                    Some(
                        top_accum
                            .iter()
                            .chain(bottom_accum.iter())
                            .map(|v| v.0.iter())
                            .flatten()
                            .map(|p| p.clone())
                            .collect(),
                    )
                }
                None => None,
            }
        }
    })
}

pub fn solid_fill_overlay_gen<'a>(
    config: &Config,
    bounds: Rect,
) -> impl Generator<Item = MultiLineString<i64>> + 'a {
    let spacing = config.solid_fill_line_spacing_dsc();
    rotating_fill_gen(
        move |r: Rect| linear_fill_bounds(spacing, r),
        config.solid_fill_initial_angle,
        config.solid_fill_angle_increment,
        bounds,
    )
}

pub fn sparse_fill_overlay_gen<'a>(
    config: &Config,
    bounds: Rect,
) -> impl Generator<Item = MultiLineString<i64>> + 'a {
    let spacing = config.sparse_fill_line_spacing_dsc();
    rotating_fill_gen(
        move |r: Rect| linear_fill_bounds(spacing, r),
        config.sparse_fill_initial_angle,
        config.sparse_fill_angle_increment,
        bounds,
    )
}
