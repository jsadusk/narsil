use crate::error::*;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs::File;
use std::io::BufReader;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub layer_height: f64,
    pub resolution: f64,
    pub simplify_factor: f64,
    pub num_shells: u64,
    pub nozzle_diameter: f64,
    pub shell_overlap: f64,
    pub shell_infill_overlap: f64,
    pub top_thickness: f64,
    pub bottom_thickness: f64,
    pub solid_fill_overlap_ratio: f64,
    pub solid_fill_initial_angle: f64,
    pub solid_fill_angle_increment: f64,
    pub sparse_fill_density: f64,
    pub sparse_fill_initial_angle: f64,
    pub sparse_fill_angle_increment: f64,
}

impl Config {
    pub fn new(fh: File) -> NarsilResult<Config> {
        Ok(serde_yaml::from_reader(BufReader::new(fh))?)
    }

    pub fn discretized(&self, value: f64) -> i64 {
        (value / self.resolution) as i64
    }

    pub fn nozzle_diameter_dsc(&self) -> i64 {
        self.discretized(self.nozzle_diameter)
    }

    pub fn shell_overlap_dsc(&self) -> i64 {
        self.discretized(self.shell_overlap)
    }

    pub fn shell_infill_overlap_dsc(&self) -> i64 {
        self.discretized(self.shell_infill_overlap)
    }

    pub fn interior_offset_dsc(&self) -> i64 {
        -(self.nozzle_diameter_dsc()
            + (self.nozzle_diameter_dsc() - self.shell_overlap_dsc())
                * (self.num_shells as i64 - 1)
            - self.shell_infill_overlap_dsc())
    }

    pub fn num_top_layers(&self) -> usize {
        (self.top_thickness / self.layer_height) as usize
    }

    pub fn num_bottom_layers(&self) -> usize {
        (self.bottom_thickness / self.layer_height) as usize
    }

    pub fn solid_fill_line_spacing_dsc(&self) -> i64 {
        self.discretized(
            self.nozzle_diameter - self.nozzle_diameter * self.solid_fill_overlap_ratio,
        )
    }

    pub fn sparse_fill_line_spacing_dsc(&self) -> i64 {
        self.discretized(
            self.nozzle_diameter * (1.0 - self.sparse_fill_density) / self.sparse_fill_density,
        )
    }
}
