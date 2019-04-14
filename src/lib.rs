extern crate quick_xml;

use std::error::Error;
use std::collections::HashMap;
use std::collections::HashSet;

use quick_xml::Reader;
use quick_xml::events::Event;

pub struct Config {
    pub filename: String,
    pub start_point: String,
    pub end_point: String,
}

struct HighWay {
    id: i64,
    nodes :Vec<i64>,
    oneway: bool,
}

struct Node {
    id: i64,
    lat: f64,
    lon:f64,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 4 {
            Err("not enough arguments")
        } else {
            let filename = args[1].clone();
            let start_point = args[2].clone();
            let end_point = args[3].clone();

            Ok(Config { filename, start_point, end_point })
        }
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_file(&config.filename).unwrap();

    let mut highway_count: u64 = 0;

    let mut way_id = 0;
    let mut in_way = false;
    let mut is_highway = true;
    let mut oneway = false;
    let mut nodes: Vec<i64> = Vec::new();

    let mut all_nodes: Vec<i64> = Vec::new();
    let mut highway_map: HashMap<i64, HighWay> = HashMap::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"way" => {
                        in_way = true;
                        way_id = e.attributes().filter_map(
                            |result| {
                                let attribute = result.unwrap();
                                if attribute.key == b"id" {
                                    let id: i64 = std::str::from_utf8(&*attribute.value)
                                        .unwrap()
                                        .parse()
                                        .unwrap();
                                    Some(id)
                                }
                                else {
                                    None
                                }
                            }
                        ).next().unwrap();
                    },
                    _ => (),
                }
            },
            Ok(Event::Empty(ref e)) => {
                if in_way {
                    match e.name() {
                        b"nd" => {
                            e.attributes().map(
                                |result| result.unwrap()
                            ).for_each(
                                |attribute| {
                                    if attribute.key == b"ref" {
                                        let id: i64 = std::str::from_utf8(&*attribute.value)
                                            .unwrap()
                                            .parse()
                                            .unwrap();
                                        nodes.push(id);
                                    }
                                }
                            )
                        },
                        b"tag" => {
                            if !is_highway {
                                is_highway = e.attributes().map(
                                    |result| result.unwrap()
                                ).fold(
                                    false,
                                    |is_highway, attribute| {
                                        if is_highway {
                                            true
                                        } else {
                                            (attribute.key == b"k") && (&*attribute.value == b"highway")
                                        }
                                    }
                                );
                            }
                        },
                        _ => (),
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                match e.name() {
                    b"way" => {
                        if (is_highway) {
                            highway_count += 1;

                            all_nodes.extend(&nodes);
                            let highway = HighWay { id:way_id, nodes:nodes, oneway:oneway };
                            highway_map.insert(way_id, highway);
                            nodes = Vec::new();
                        }

                        nodes.clear();
                        in_way = false;
                        is_highway = false;
                    }
                    _ => (),
                }
            },
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }

        buf.clear();
    }

    println!(
        "Found {} highways\n\
        Number of HighWay structs in Map: {}\n\
        Number of Nodes in all_nodes: {}",
        highway_count,
        highway_map.len(),
        all_nodes.len(),
    );

    Ok(())
}
