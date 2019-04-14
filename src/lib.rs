extern crate quick_xml;

pub mod config;
mod parser;

use std::error::Error;

use config::Config;
use parser::graph::Graph;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    println!(
        "Filename: {}\n\
        Start point: {}\n\
        End point: {}",
        config.filepath,
        config.start_point,
        config.end_point,
    );

    let graph = Graph::from_file(config.filepath);

    Ok(())
}
