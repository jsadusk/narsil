#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate byteorder;
extern crate hedge;
extern crate svg;

#[macro_use]
extern crate failure;

use failure::Error;
use std::fs::File;
use std::f64;

use svg::Document;
use svg::node::element::Path as svgPath;
use svg::node::element::path;

use std::path::Path as filePath;

mod model_file;
mod slicer;

use slicer::Layer;

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

    pub fn output_fh(&self, id: usize) -> Result<File, std::io::Error> {
        let orig = filePath::new(&self.output_filename);
        let dir = orig.parent().unwrap_or(filePath::new(""));
        let file_stem = orig.file_stem().unwrap_or(std::ffi::OsStr::new(""));
        let ext = orig.extension().unwrap_or(std::ffi::OsStr::new(""));

        let filename = format!("{}_{}.{}",
                               file_stem.to_str().unwrap(),
                               id,
                               ext.to_str().unwrap());
        let full = dir.join(filename);
        File::create(full)
    }
}

pub fn write_svg(fh : File, slice : &Layer, factor: f64) -> Result<(), std::io::Error> {
    let mut minx = f64::INFINITY;
    let mut miny = f64::INFINITY;

    for poly in slice.iter() {
        for point in poly.iter() {
            if point[0] < minx {
                minx = point[0];
            }
            if point[1] < miny {
                miny = point[0];
            }
        }
    }

    let mut document = Document::new();
    for poly in slice.iter() {
        let mut data = path::Data::new()
            .move_to(((poly[0][0] - minx) * factor,
                      (poly[0][1] - miny) * factor));

        for point in poly.iter().skip(1) {
            data = data.line_to(((point[0] - minx) * factor,
                                 (point[1] - miny) * factor));
        }
        
        data = data.close();

        let path = svgPath::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 2)
            .set("d", data);

        document = document.add(path);
    }
    
    svg::write(fh, &document)
}

pub fn run(config : Config) -> Result<(), Error> {
    let mesh = model_file::load(config.input_fh()?)?;
    println!("num triangles {}", mesh.faces().count());

    let slices = slicer::slice(mesh)?;

    for (i, slice) in slices.iter().enumerate() {
        write_svg(config.output_fh(i)?, slice, 100.0)?;
    }
    
    Ok(())
}
