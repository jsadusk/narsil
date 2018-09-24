extern crate narsil;

use std::process;
use std::env;
use narsil::Config;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(
        |err| {
            println!("Args error: {}", err);
            process::exit(1);
        });

    if let Err(e) = narsil::run(config) {
        println!("Error: {}", e);
        process::exit(1);
    }
}
