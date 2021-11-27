#![feature(type_alias_impl_trait)]

extern crate byteorder;
extern crate geo;
extern crate geo_clipper;
extern crate geo_collate;
extern crate geo_svg;
extern crate hedge;
extern crate quickersort;
extern crate rayon;
extern crate regex;
extern crate rstar;
extern crate serde;
extern crate serde_yaml;
extern crate simple_generator;
extern crate svg;

#[macro_use]
extern crate lazy_static;

mod captures;
pub mod config;
mod connect;
mod error;
mod generator;
mod id_factory;
mod infill;
mod mesh;
mod model_file;
mod ops;
mod slicer;
mod types;
mod writers;
pub mod run;
mod geometry;

use crate::types::*;
pub use crate::run::*;


