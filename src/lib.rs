#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate byteorder;
extern crate hedge;
extern crate svg;
extern crate quickersort;
extern crate rayon;

#[macro_use]
extern crate failure;

use failure::Error;
use std::fs::File;
use std::f64;

use svg::Document;
use svg::node::element::Path as svgPath;
use svg::node::element::Group as svgGroup;
use svg::node::element::path;

use hedge::Mesh;

use std::path::Path as filePath;

mod model_file;
mod slicer;

use slicer::LayerStack;

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

    pub fn input_fh(&self) -> Result<File, std::io::Error> {
        File::open(self.input_filename.clone())
    }

    pub fn output_fh(&self) -> Result<File, std::io::Error> {
        File::create(self.output_filename.clone())
    }
}

struct Range {
    min : f64,
    max : f64
}

impl Range {
    fn new() -> Range {
        Range { min : f64::INFINITY,
                max : f64::NEG_INFINITY }
    }
}

struct Bounds3D {
    x : Range,
    y : Range,
    z : Range
}

impl Bounds3D {
    fn new(mesh : &Mesh) -> Bounds3D {
        let mut bounds = Bounds3D { x : Range::new(),
                                    y : Range::new(),
                                    z : Range::new() };
        for face in mesh.faces().map(|fi| mesh.face(fi)) {
            for point in mesh.vertices(face).map(|vi| mesh.vertex(vi).point) {
                if point[0] < bounds.x.min {
                    bounds.x.min = point[0];
                }
                if point[0] > bounds.x.max {
                    bounds.x.max = point[0];
                }
                if point[1] < bounds.y.min {
                    bounds.y.min = point[1];
                }
                if point[1] > bounds.y.max {
                    bounds.y.max = point[1];
                }
                if point[2] < bounds.z.min {
                    bounds.z.min = point[2];
                }
                if point[2] > bounds.z.max {
                    bounds.z.max = point[2];
                }
            }
        }

        bounds
    }
}

fn write_svg(fh : File, slices : &LayerStack, bounds : Bounds3D, factor: f64) -> Result<(), std::io::Error> {
    let mut document = Document::new()
        .set("viewbox", (0, 0,
                         (bounds.x.max - bounds.x.min) * factor,
                         (bounds.y.max - bounds.y.min) * factor));
    for (id, slice) in slices.iter().enumerate() {
        let mut group = svgGroup::new()
            .set("id", format!("layer_{}", id))
            .set("display", "true");

        for poly in slice.iter() {
            let mut data = path::Data::new()
                .move_to(((poly[0][0] - bounds.x.min) * factor,
                          (poly[0][1] - bounds.y.min) * factor));

            for point in poly.iter().skip(1) {
                data = data.line_to(((point[0] - bounds.x.min) * factor,
                                     (point[1] - bounds.y.min) * factor));
            }
        
            data = data.close();

            let path = svgPath::new()
                .set("fill", "none")
                .set("stroke", "black")
                .set("stroke-width", 2)
                .set("d", data);

            group = group.add(path);
        }

        document = document.add(group);
    }
    
    svg::write(fh, &document)
}

pub fn run(config : Config) -> Result<(), Error> {
    let mesh = model_file::load(config.input_fh()?)?;

    println!("slice");
    let slices = slicer::slice(&mesh)?;

    println!("svg");
    write_svg(config.output_fh()?, &slices, Bounds3D::new(&mesh), 10.0)?;
    
    Ok(())
}
