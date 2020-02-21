#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate byteorder;
extern crate hedge;
extern crate svg;
extern crate quickersort;
extern crate rayon;
extern crate expression;

use std::fs::File;

use expression::*;
use expression::simple_engine::*;
use std::path::Path as filePath;

mod model_file;
mod mesh;
mod slicer;
mod error;
mod writers;

use crate::error::NarsilError;

use crate::mesh::*;

pub struct Config {
    input_filename : String,
    output_filename : String
}

impl Config {
    pub fn new(args: &Vec<String>) -> Result<Config, String> {
        if args.len() < 3 {
            Err(format!("Usage: {} <input_file> <output_file>", args[0]))
        }
        else {
            Ok(Config { input_filename: args[1].clone(),
                        output_filename: args[2].clone()})
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


pub fn run(config : Config) -> Result<(), ExpressionError<NarsilError>> {
    let mut engine = SimpleEngine::<NarsilError>::new();

    let input_fh = config.input_fh()?;

    let ft = engine.term(model_file::IdentifyModelType { fh: input_fh.try_clone().map_err(|e| ExpressionError::<NarsilError>::Eval(NarsilError::IO(e)))? });
    let free_surface = engine.list_term(model_file::LoadTriangles{ fh: input_fh.try_clone().map_err(|e| ExpressionError::<NarsilError>::Eval(NarsilError::IO(e)))?,
                                                         ft: ft.into() });
    let unified_triangles = engine.term(model_file::UnifyVertices { free_mesh: free_surface.into() });
    let connected_mesh = engine.term(model_file::ConnectedMesh{ unified_triangles: unified_triangles.into() });

    let bounds = engine.term(mesh::MeshBounds { mesh: connected_mesh.clone().into() });

    let sorted_faces = engine.list_term(slicer::SortedFaces { mesh: connected_mesh.clone().into() });

    let layer_faces = engine.list_term(slicer::LayerFaces {
        mesh: connected_mesh.clone().into(),
        bounds: bounds.clone().into(),
        sorted_faces: sorted_faces.into()
    });

    let slicer = engine.list_term(slicer::SliceFaces { mesh: connected_mesh.into(), layer_faces: layer_faces.into() });

    let write_html = engine.term(writers::WriteHtml {
        name: config.name(),
        fh: config.output_fh()?,
        slices: slicer.into(),
        bounds: bounds.into(),
        factor: 7.0,
    });

    engine.eval(&write_html)?;

    Ok(())
}
