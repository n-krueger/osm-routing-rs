extern crate quick_xml;

mod config;
mod geo;
mod routing;

use std::error::Error;

pub use self::config::Config;
use self::geo::graph::Graph;

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
