use std::env;
use std::process;

use osm_routing_rs::Config;
use osm_routing_rs::run;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(
        |err| {
            eprintln!("Error parsing arguments: {}", err);
            process::exit(1);
        }
    );

    if let Err(e) = run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}
