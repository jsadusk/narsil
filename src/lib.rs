#[macro_use]
extern crate lazy_static;
extern crate byteorder;
extern crate geo;
extern crate geo_booleanop;
extern crate hedge;
extern crate quickersort;
extern crate rayon;
extern crate regex;
extern crate serde;
extern crate serde_yaml;
extern crate svg;

pub mod config;
mod error;
mod generator;
mod mesh;
mod model_file;
mod slicer;
mod types;
mod writers;

use crate::config::*;
use crate::error::*;

use crate::generator::*;
use crate::mesh::*;
use crate::model_file::*;
use geo::prelude::*;
use rayon::prelude::*;

pub fn run(args: Args) -> NarsilResult<()> {
    let config = args.config()?;
    let mut input_fh = args.input_fh()?;

    let ft = model_file::identify(&mut input_fh)?;

    let free_surface = model_file::load_triangles(&ft, &mut input_fh)?;

    let (surface, vertices) = model_file::unify_vertices(&free_surface);

    let connected_mesh = hedge::Mesh::from_surface(surface, vertices);

    let bounds = mesh_bounds(&connected_mesh);

    let sorted_faces = slicer::sort_faces(&connected_mesh);

    let layer_faces = slicer::layer_faces(config.layer_height, &bounds, &sorted_faces);

    let outlines = layer_faces
        .par_iter()
        .map(|l| slicer::slice_layer(l.0, &connected_mesh, &l.1))
        .collect::<slicer::SlicerResult<Vec<slicer::Layer>>>()?;

    let simplified_outlines: Vec<slicer::Layer> = outlines
        .par_iter()
        .map(|l| l.simplify(&config.resolution))
        .collect();

    writers::write_html(
        args.name(),
        &mut args.output_fh()?,
        simplified_outlines.into_iter(),
        (bounds.z.len() / config.layer_height) as i64,
        &bounds,
        7.0,
    )?;

    Ok(())
}
