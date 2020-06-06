#[macro_use]
extern crate lazy_static;
extern crate byteorder;
extern crate hedge;
extern crate quickersort;
extern crate rayon;
extern crate regex;
extern crate svg;

use std::fs::File;

use std::path::Path as filePath;

mod error;
mod generator;
mod mesh;
mod model_file;
mod slicer;
mod writers;

use crate::error::NarsilError;

use crate::generator::*;
use crate::mesh::*;
use crate::model_file::*;

pub struct Config {
    input_filename: String,
    output_filename: String,
}

impl Config {
    pub fn new(args: &Vec<String>) -> Result<Config, String> {
        if args.len() < 3 {
            Err(format!("Usage: {} <input_file> <output_file>", args[0]))
        } else {
            Ok(Config {
                input_filename: args[1].clone(),
                output_filename: args[2].clone(),
            })
        }
    }

    pub fn input_fh(&self) -> Result<File, NarsilError> {
        File::open(self.input_filename.clone()).map_err(|e| e.into())
    }

    pub fn output_fh(&self) -> Result<File, NarsilError> {
        File::create(self.output_filename.clone()).map_err(|e| e.into())
    }

    pub fn name(&self) -> String {
        let path = filePath::new(self.input_filename.as_str());
        path.file_name().unwrap().to_str().unwrap().to_string()
    }
}

pub fn run(config: Config) -> Result<(), NarsilError> {
    let mut input_fh = config.input_fh()?;

    let ft = model_file::identify(&mut input_fh)?;

    let free_surface = model_file::load_triangles(&ft, &mut input_fh)?;

    let (surface, vertices) = model_file::unify_vertices(&free_surface);

    let connected_mesh = hedge::Mesh::from_surface(surface, vertices);

    let bounds = mesh_bounds(&connected_mesh);

    let sorted_faces = slicer::sort_faces(&connected_mesh);

    let layer_faces = slicer::layer_faces(&connected_mesh, &bounds, &sorted_faces);

    let slices = slicer::slice_faces(&connected_mesh, &layer_faces)?;

    let write_html = writers::write_html(
        config.name(),
        &mut config.output_fh()?,
        &slices,
        &bounds,
        7.0,
    )?;

    Ok(())
}
