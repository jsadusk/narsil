use std::fs::File;
use std::io::Write;

use svg::node::element::path;
use svg::node::element::Group as svgGroup;
use svg::node::element::Path as svgPath;
use svg::Document;

use crate::mesh::Bounds3D;
use types::*;

pub fn write_html(
    name: String,
    fh: &mut File,
    slices: impl Iterator<Item = Vec<TaggedPath>>,
    num_slices: i64,
    bounds: &Bounds3D,
    resolution: f64,
    factor: f64,
) -> Result<(), std::io::Error> {
    let mut document = Document::new()
        .set(
            "viewbox",
            (
                0,
                0,
                (bounds.x.max - bounds.x.min) * factor,
                (bounds.y.max - bounds.y.min) * factor,
            ),
        )
        .set("id", "layers");
    for (id, slice) in slices.enumerate() {
        let mut group = svgGroup::new()
            .set("id", format!("layer_{}", id))
            .set("display", "none");

        for path in slice.into_iter() {
            let poly = path.path;

            let mut data = path::Data::new().move_to((
                (poly.0[0].x as f64 * resolution - bounds.x.min) * factor,
                (poly.0[0].y as f64 * resolution - bounds.y.min) * factor,
            ));

            for point in poly.0.iter().skip(1) {
                data = data.line_to((
                    (point.x as f64 * resolution - bounds.x.min) * factor,
                    (point.y as f64 * resolution - bounds.y.min) * factor,
                ));
            }

            data = data.close();

            let color = match path.tag {
                PathTag::Region => "black",
                PathTag::Shell => "red",
                PathTag::Interior => "yellow",
                PathTag::Solid => "green",
                PathTag::Sparse => "blue",
                PathTag::Unknown => "grey",
            };

            let path = svgPath::new()
                .set("fill", "none")
                .set("stroke", color)
                .set("stroke-width", 0.2)
                .set("d", data);

            group = group.add(path);
        }
        document = document.add(group);
    }

    fh.write(
        format!(
            r#"
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
"#,
            name,
            num_slices - 1
        )
        .as_bytes(),
    )?;
    svg::write(&*fh, &document)?;
    fh.write(
        format!(
            r#"
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
"#,
            num_slices
        )
        .as_bytes(),
    )?;
    Ok(())
}
