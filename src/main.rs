extern crate narsil;

use narsil::config::Args;
use std::env;
use std::process;

fn main() {
    let cmdline: Vec<String> = env::args().collect();
    let args = Args::new(&cmdline).unwrap_or_else(|err| {
        println!("Args error: {}", err);
        process::exit(1);
    });

    if let Err(e) = narsil::run(args) {
        println!("Error: {}", e);
        process::exit(1);
    }
}
