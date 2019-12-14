use std::fs::File;
use std::io::Write;

use expression::*;
use svg::Document;
use svg::node::element::Path as svgPath;
use svg::node::element::Group as svgGroup;
use svg::node::element::path;

use crate::slicer::LayerStack;
use crate::mesh::Bounds3D;

pub struct WriteHtml<LS, B> {
    pub name: String,
    pub fh: File,
    pub slices: TermResult<LS>,
    pub bounds: TermResult<B>,
    pub factor: f64
}

impl<LS, B> Expression for WriteHtml<LS, B>
where
    LS: TypedTerm<ValueType=LayerStack>,
    B: TypedTerm<ValueType=Bounds3D>
{
    type ValueType = ();
    type ErrorType = std::io::Error;

    fn terms(&self) -> Terms {
        vec!(self.slices.term(), self.bounds.term())
    }

    fn eval(&self) -> Result<(), std::io::Error> {
        let mut fh = self.fh.try_clone()?;

        let mut document = Document::new()
            .set("viewbox", (0, 0,
                             (self.bounds.x.max - self.bounds.x.min)
                                 * self.factor,
                             (self.bounds.y.max - self.bounds.y.min)
                                 * self.factor))
            .set("id", "layers");
        for (id, slice) in self.slices.iter().enumerate() {
            let mut group = svgGroup::new()
                .set("id", format!("layer_{}", id))
                .set("display", "none");

            for poly in slice.iter() {
                let mut data = path::Data::new()
                    .move_to(((poly[0][0] - self.bounds.x.min) * self.factor,
                              (poly[0][1] - self.bounds.y.min) * self.factor));

                for point in poly.iter().skip(1) {
                    data = data.line_to(((point[0] - self.bounds.x.min) * self.factor,
                                         (point[1] - self.bounds.y.min) * self.factor));
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
"#, self.name, self.slices.len() - 1).as_bytes())?;
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
"#, self.slices.len()).as_bytes())?;
        Ok(())

    }
}
