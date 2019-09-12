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
use std::f64;
use std::io::Write;

use svg::Document;
use svg::node::element::Path as svgPath;
use svg::node::element::Group as svgGroup;
use svg::node::element::path;

use hedge::Mesh;

use expression::*;
use std::path::Path as filePath;

mod model_file;
mod slicer;
mod error;

use slicer::LayerStack;
use crate::error::NarsilError;

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

    pub fn name(&self) -> String {
        let path = filePath::new(self.input_filename.as_str());
        path.file_name().unwrap().to_str().unwrap().to_string()
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

fn write_html(name : String,
              mut fh : File,
              slices : &LayerStack,
              bounds : Bounds3D,
              factor: f64) -> Result<(), std::io::Error> {
    let mut document = Document::new()
        .set("viewbox", (0, 0,
                         (bounds.x.max - bounds.x.min) * factor,
                         (bounds.y.max - bounds.y.min) * factor))
        .set("id", "layers");
    for (id, slice) in slices.iter().enumerate() {
        let mut group = svgGroup::new()
            .set("id", format!("layer_{}", id))
            .set("display", "none");

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

    fh.write(format!(r#"
<!DOCTYPE html><html><head><title>{}</title>
<meta name="viewport" content="width=device-width, initial-scale=1">
<style>
.slidecontainer {{
    width: 100%;
}}

.slider {{
    -webkit-appearance: none;
    width: 100%;
    height: 25px;
    background: #d3d3d3;
    outline: none;
    opacity: 0.7;
    -webkit-transition: .2s;
    transition: opacity .2s;
}}

.slider:hover {{
    opacity: 1;
}}

.slider::-webkit-slider-thumb {{
    -webkit-appearance: none;
    appearance: none;
    width: 25px;
    height: 25px;
    background: #4CAF50;
    cursor: pointer;
}}

.slider::-moz-range-thumb {{
    width: 25px;
    height: 25px;
    background: #4CAF50;
    cursor: pointer;
}}
</style>
</head><body>
<div class="slidecontainer">
  <input type="range" min="0" max="{}" value="0" class="slider" id="layerSlider">
    <p>Value: <span id="layerId"></span></p>
</div>
"#, name, slices.len() - 1).as_bytes())?;
    svg::write(&fh, &document)?;
    fh.write(format!(r#"
<script>
var slider = document.getElementById("layerSlider");
var layerSvg = document.getElementById("layers");
var output = document.getElementById("layerId");
var curLayerGroup = layerSvg.getElementById("layer_0");
curLayerGroup.setAttributeNS(null, 'display', "true");
var numLayers = {};

output.innerHTML = slider.value;

slider.oninput = function() {{
    output.innerHTML = this.value;

    for (i = 0; i < numLayers; ++i) {{
       var thisLayerGroup = layerSvg.getElementById("layer_" + i);
       thisLayerGroup.setAttributeNS(null, 'display', 'none');
    }}

    var newLayerGroup = layerSvg.getElementById("layer_" + this.value);
    newLayerGroup.setAttributeNS(null, 'display', 'true');
}}
</script>

</body></html>
"#, slices.len()).as_bytes())?;
    Ok(())
}

pub fn run(config : Config) -> Result<(), ExpressionError<NarsilError>> {
    let mut engine = Engine::<NarsilError>::new();

    let mesh_term = engine.term(model_file::LoadModel{ fh: config.input_fh().map_err(|e| NarsilError::IO(e))? });

    let mesh = engine.eval(&mesh_term)?;

    println!("slice");
    let slices = slicer::slice(&mesh).map_err(|e| ExpressionError::<NarsilError>::Eval(NarsilError::Slicer(e)))?;

    println!("svg");
    write_html(config.name(),
               config.output_fh()
                 .map_err(|e| ExpressionError::<NarsilError>::Eval(NarsilError::IO(e)))?,
               &slices, Bounds3D::new(&mesh),
               10.0).map_err(|e| ExpressionError::<NarsilError>::Eval(NarsilError::IO(e)))?;

    Ok(())
}
