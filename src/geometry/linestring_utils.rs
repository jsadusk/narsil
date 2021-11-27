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
