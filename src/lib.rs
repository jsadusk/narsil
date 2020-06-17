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

use std::fs::File;
use std::io::BufReader;
use std::path::Path as filePath;

mod error;
mod generator;
mod mesh;
mod model_file;
mod slicer;
mod types;
mod writers;

use crate::error::NarsilError;

use crate::generator::*;
use crate::mesh::*;
use crate::model_file::*;
use geo::prelude::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_yaml::Result as SerdeResult;

pub type NarsilResult<T> = Result<T, NarsilError>;

pub struct Args {
    config_filename: String,
    input_filename: String,
    output_filename: String,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    layer_height: f64,
    resolution: f64,
}

impl Args {
    pub fn new(cmdline: &Vec<String>) -> Result<Args, String> {
        if cmdline.len() < 4 {
            Err(format!(
                "Usage: {} <config_file> <input_file> <output_file>",
                cmdline[0]
            ))
        } else {
            Ok(Args {
                config_filename: cmdline[1].clone(),
                input_filename: cmdline[2].clone(),
                output_filename: cmdline[3].clone(),
            })
        }
    }

    pub fn config_fh(&self) -> NarsilResult<File> {
        Ok(File::open(self.config_filename.clone())?)
    }

    pub fn config(&self) -> NarsilResult<Config> {
        Ok(serde_yaml::from_reader(BufReader::new(self.config_fh()?))?)
    }

    pub fn input_fh(&self) -> NarsilResult<File> {
        Ok(File::open(self.input_filename.clone())?)
    }

    pub fn output_fh(&self) -> NarsilResult<File> {
        Ok(File::create(self.output_filename.clone())?)
    }

    pub fn name(&self) -> String {
        let path = filePath::new(self.input_filename.as_str());
        path.file_name().unwrap().to_str().unwrap().to_string()
    }
}

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
