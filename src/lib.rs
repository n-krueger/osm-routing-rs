extern crate quick_xml;

mod config;
mod geo;
mod routing;

use std::error::Error;

pub use self::config::Config;
use self::geo::graph::Graph;
use crate::geo::Coordinates;

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
    let start_coords = Coordinates::from_string(&config.start_point);
    let end_coords = Coordinates::from_string(&config.end_point);
    let route = routing::route(&graph, &start_coords, &end_coords);

    for id in route {
        let node = graph.id_to_node[&id];
        println!("{:?}", node);
    }

    Ok(())
}
