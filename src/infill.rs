use crate::captures::*;
//use crate::config::*;
use crate::types::*;

use geo::bounding_rect::*;
use simple_generator::Generator;
use simple_generator::*;
use std::collections::HashMap;

pub fn rotated_fill(
    fill_func: &impl Fn(Rect) -> MultiLineString,
    angle: f64,
    bounds: &Rect,
) -> MultiLineString {
    fill_func(
        bounds
            .to_polygon()
            .exterior()
            .rotate_around_point(-angle, bounds.center())
            .bounding_rect()
            .unwrap(),
    )
    .rotate_around_point(angle, bounds.center())
}

pub fn linear_fill_bounds(spacing: i64, bounds: Rect) -> MultiLineString {
    let mut lines = MultiLineString::default();
    let mut line_pos = bounds.min().x;

    while line_pos <= bounds.max().x {
        lines.push(geo::LineString(vec![
            Coordinate {
                x: line_pos,
                y: bounds.min().y,
            },
            Coordinate {
                x: line_pos,
                y: bounds.max().y,
            },
        ]));
        line_pos += spacing;
    }

    lines
}

pub fn rotating_fill_gen<'a>(
    fill_func: impl Fn(Rect) -> MultiLineString + 'a,
    init_angle: f64,
    rotate_angle: f64,
    bounds: Rect,
) -> impl Generator<Item = MultiLineString> + Captures<'a> {
    GeneratorFunc::new(move || {
        let mut angle = init_angle;
        let mut cache = HashMap::<i64, MultiLineString>::new();
        //let fill_func = &fill_func;
        move || {
            let approx_angle = (angle * 10000.0) as i64;
            let result = match cache.get(&approx_angle) {
                Some(fill) => fill.clone(),
                None => {
                    let fill = rotated_fill(&fill_func, angle, &bounds);
                    cache.insert(approx_angle, fill.clone());
                    fill
                }
            };
            angle += rotate_angle;
            Some(result)
        }
    })
}
