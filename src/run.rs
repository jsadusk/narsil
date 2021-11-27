use geo::prelude::*;
use geo_clipper::*;
use geo_collate::*;
use rayon::prelude::*;
use simple_generator::Generator;
use std::iter;
use hedge;

use crate::geometry::linestring_utils::*;
use crate::model_file;
use model_file::FromSurface;
use crate::slicer;
use crate::ops;
use crate::types::*;
use crate::writers;
use crate::config::*;
use crate::error::*;
use crate::mesh::*;
use crate::ops::*;

pub fn run(args: Args) -> NarsilResult<()> {
    let config = args.config()?;
    let mut input_fh = args.input_fh()?;

    println!("Loading");
    let ft = model_file::identify(&mut input_fh)?;

    let free_surface = model_file::load_triangles(&ft, &mut input_fh)?;

    let (surface, vertices) = model_file::unify_vertices(&free_surface);

    let connected_mesh = hedge::Mesh::from_surface(surface, vertices);

    let bounds = mesh_bounds(&connected_mesh);

    println!("Slicing");
    let sorted_faces = slicer::sort_faces(&connected_mesh);

    let layer_faces = slicer::layer_faces(config.layer_height, &bounds, &sorted_faces);

    let outlines = layer_faces
        .par_iter()
        .map(|l| slicer::slice_layer(l.0, &connected_mesh, &l.1))
        .collect::<slicer::SlicerResult<Vec<slicer::Layer>>>()?;

    let simplified_outlines: Vec<MultiLineString> = outlines
        .par_iter()
        .map(|l| l.simplify(&config.simplify_factor))
        .map(|l| integerize(&l, config.resolution))
        .collect();

    let collated_outlines: Vec<MultiPolygon> = simplified_outlines
        .par_iter()
        .map(|l| Ok(l.collate()?))
        .collect::<Result<Vec<MultiPolygon>, geo_collate::CollateError>>()?;

    let outline_regions: Vec<LayerRegions> =
        collated_outlines.iter().map(|p| p.clone().into()).collect();

    let shells: Vec<LayerShells> = outline_regions.par_iter().map(shells_op(&config)).collect();

    let interiors: Vec<InteriorRegions> = outline_regions
        .par_iter()
        .map(interiors_op(&config))
        .collect();

    let top_exposed: Vec<MultiPolygon> = ops::adjacent_gen(collated_outlines.iter())
        .into_iter()
        //.par_bridge()
        .map(ops::difference_op())
        .chain(iter::once(collated_outlines.last().unwrap().clone()))
        .collect();

    let bottom_exposed: Vec<MultiPolygon> = iter::once(collated_outlines.first().unwrap().clone())
        .chain(
            ops::adjacent_gen(collated_outlines.iter())
                .into_iter()
                .map(|(a, b)| (b, a))
                .map(ops::difference_op()),
        )
        .collect();

    let solid: Vec<SolidRegions> =
        ops::solid_grouping_gen(&config, top_exposed.iter(), bottom_exposed.iter())
            .into_iter()
            .zip(interiors.iter())
            .map(|(grouping, interior)| {
                let interior_polys: MultiPolygon =
                    interior.0.iter().map(|i| i.poly.clone()).collect();
                grouping
                    .union(&MultiPolygon::default())
                    .intersection(&interior_polys)
            })
            .map(|p| p.into())
            .collect();

    let sparse: Vec<SparseRegions> = interiors
        .iter()
        .zip(solid.iter())
        .map(|(interior, solid)| (*interior).difference(solid))
        .map(|p| p.into())
        .collect();

    /*let mut upper = Vec::<geo::MultiPolygon<f64>>::new();
    let mut last = collated.first().unwrap().clone();
    for layer in collated.iter().skip(1) {
        upper.push(last.difference(layer, 10000.0));
        last = layer.clone();
    }*/

    let layer_bounds: Vec<Rect> = collated_outlines
        .iter()
        .map(|l| l.bounding_rect())
        .collect::<Option<Vec<Rect>>>()
        .unwrap();

    let accum_layer_bounds: Rect = layer_bounds
        .iter()
        .fold(*layer_bounds.first().unwrap(), |accum, bound| {
            accum.bound_sum(&bound)
        });

    let solid_fill: Vec<MultiLineString> = solid
        .iter()
        .zip(ops::solid_fill_overlay_gen(&config, accum_layer_bounds).into_iter())
        .map(|(region, pattern)| pattern.intersection(region))
        .collect();

    let sparse_fill: Vec<MultiLineString> = sparse
        .iter()
        .zip(ops::sparse_fill_overlay_gen(&config, accum_layer_bounds).into_iter())
        .map(|(region, pattern)| pattern.intersection(region))
        .collect();

    let tagged_paths: Vec<Vec<TaggedPath>> = outline_regions
        .into_iter()
        .map(|l| l.into())
        .zip(shells.into_iter().map(|l| l.into()))
        .map(|(a, b): (Vec<TaggedPath>, Vec<TaggedPath>)| {
            a.into_iter().chain(b.into_iter()).collect()
        })
        /*.zip(interiors.into_iter().map(|l| l.into()))
        .map(|(a, b): (Vec<TaggedPath>, Vec<TaggedPath>)| {
            a.into_iter().chain(b.into_iter()).collect()
        })*/
        .zip(solid.into_iter().map(|l| l.into()))
        .map(|(a, b): (Vec<TaggedPath>, Vec<TaggedPath>)| {
            a.into_iter().chain(b.into_iter()).collect()
        })
        .zip(sparse.into_iter().map(|l| l.into()))
        .map(|(a, b): (Vec<TaggedPath>, Vec<TaggedPath>)| {
            a.into_iter().chain(b.into_iter()).collect()
        })
        .zip(solid_fill.into_iter().map(|l| {
            l.0.into_iter()
                .map(|p| TaggedPath {
                    tag: PathTag::Solid,
                    path: p,
                })
                .collect()
        }))
        .map(|(a, b): (Vec<TaggedPath>, Vec<TaggedPath>)| {
            a.into_iter().chain(b.into_iter()).collect()
        })
        .zip(sparse_fill.into_iter().map(|l| {
            l.0.into_iter()
                .map(|p| TaggedPath {
                    tag: PathTag::Solid,
                    path: p,
                })
                .collect()
        }))
        .map(|(a, b): (Vec<TaggedPath>, Vec<TaggedPath>)| {
            a.into_iter().chain(b.into_iter()).collect()
        })
        .collect();

    println!("write");
    writers::write_html(
        args.name(),
        &mut args.output_fh()?,
        tagged_paths.into_iter(),
        (bounds.z.len() / config.layer_height) as i64 - 1,
        &bounds,
        config.resolution,
        7.0,
    )?;

    Ok(())
}
