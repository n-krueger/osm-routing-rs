extern crate quick_xml;

use std::error::Error;
use std::collections::HashMap;
use std::collections::HashSet;

use quick_xml::Reader;
use quick_xml::events::Event;
use quick_xml::events::BytesStart;

pub struct Config {
    pub filename: String,
    pub start_point: String,
    pub end_point: String,
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

#[derive(Copy, Clone)]
struct NodeElement {
    id: i64,
    lat: f64,
    lon:f64,
}

impl NodeElement {
    fn new(e: &BytesStart) -> NodeElement {
        let fields = e.attributes()
        .fold(
            (None, None, None),
            |acc, result| {
                let attribute = result.unwrap();
                match attribute.key {
                    b"id" => {
                        let id: i64 = std::str::from_utf8(&*attribute.value)
                            .expect("NodeElement id is not UTF-8")
                            .parse()
                            .expect("NodeElement id is not an integer value");
                        (Some(id), acc.1, acc.2)
                    },
                    b"lat" => {
                        let lat:f64 = std::str::from_utf8(&*attribute.value)
                            .expect("NodeElement lat is not UTF-8")
                            .parse()
                            .expect("NodeElement lat is not a decimal value");
                        (acc.0, Some(lat), acc.2)
                    },
                    b"lon" => {
                        let lon:f64 = std::str::from_utf8(&*attribute.value)
                            .expect("NodeElement lon is not UTF-8")
                            .parse()
                            .expect("NodeElement lon is not a decimal value");
                        (acc.0, acc.1, Some(lon))
                    }
                    _ => acc
                }
            }
        );
        match fields {
            (Some(id), Some(lat), Some(lon)) => NodeElement { id, lat, lon, },
            _ => panic!("NodeElement is missing id, lat or lon"),
        }
    }

    fn distance(u: &NodeElement, v: &NodeElement) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;

        let u_lat = u.lat.to_radians();
        let v_lat = v.lat.to_radians();

        let delta_lat = (v.lat - u.lat).to_radians();
        let delta_lon = (v.lon - u.lon).to_radians();

        let central_angle_inner = (delta_lat / 2.0).sin().powi(2)
            + u_lat.cos() * v_lat.cos() * (delta_lon / 2.0).sin().powi(2);
        let central_angle = 2.0 * central_angle_inner.sqrt().asin();

        EARTH_RADIUS_KM * central_angle
    }
}

#[derive(Copy, Clone)]
struct DirectedEdge {
    from: i64,
    to: i64,
    distance: f64,
}

impl DirectedEdge {
    fn new(from: &NodeElement, to: &NodeElement) -> DirectedEdge {
        let distance = NodeElement::distance(from, to);
        DirectedEdge { from: from.id, to: to.id, distance }
    }

    fn reversed(&self) -> DirectedEdge {
        DirectedEdge { from: self.to, to: self.from, distance: self.distance }
    }
}

#[derive(Clone)]
struct WayElement {
    id: i64,
    is_highway: bool,
    is_oneway: bool,
    nodes: Vec<i64>,
}

impl WayElement {
    fn new(e: &BytesStart) -> WayElement {
        let id = e
            .attributes()
            .filter_map(
                |result| {
                let attribute = result.unwrap();
                    if attribute.key == b"id" {
                        let id: i64 = std::str::from_utf8(&*attribute.value)
                            .expect("WayElement id is not UTF-8")
                            .parse()
                            .expect("WayElement id is not an integer value");
                        Some(id)
                    }
                    else {
                        None
                    }
                }
            )
            .next()
            .expect("WayElement has no id attribute");
        let is_highway = false;
        let is_oneway = false;
        let nodes = Vec::new();

        WayElement { id, is_highway, is_oneway, nodes }
    }

    fn handle_nd(&mut self, e: &BytesStart) {
        let nd_id = e.attributes()
            .filter_map(
                |result| {
                    let attribute = result.unwrap();
                    if attribute.key == b"ref" {
                        let id: i64 = std::str::from_utf8(&*attribute.value)
                            .expect("nd ref is not UTF-8")
                            .parse()
                            .expect("nd ref is not an integer value");
                        Some(id)
                    } else {
                        None
                    }
                }
            )
            .next()
            .expect("nd has no ref attribute");
        self.nodes.push(nd_id);
    }

    fn handle_tag(&mut self, e: &BytesStart) {
        if !self.is_highway {
            self.is_highway = e.attributes()
                .filter_map(
                    |result| {
                        let attribute = result.unwrap();
                        if attribute.key == b"k" {
                            if &*attribute.value == b"highway" {
                                Some(true)
                            } else {
                                Some(false)
                            }
                        } else {
                            None
                        }
                    }
                )
                .any(|is_highway_tag| is_highway_tag);
        }
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut node_count: u64 = 0;
    let mut way_count: u64 = 0;

    let mut way_element: Option<WayElement> = None;

    let mut highway_nodes: HashSet<i64> = HashSet::new();
    let mut highway_map: HashMap<i64, WayElement> = HashMap::new();
    let mut node_map: HashMap<i64, NodeElement> = HashMap::new();

    let mut reader = Reader::from_file(&config.filename).unwrap();
    let mut buf = Vec::new();

    println!("Reading OSM file... (this may take a while)");

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if let b"way" = e.name() {
                    match way_element {
                        Some(_) => panic!("Found nested WayElement"),
                        None => way_element = Some(WayElement::new(e)),
                    }
                }
            },
            Ok(Event::Empty(ref e)) => {
                match way_element {
                    Some(ref mut we) => {
                        match e.name() {
                            b"nd" => {
                                we.handle_nd(&e);
                            },
                            b"tag" => {
                                we.handle_tag(&e);
                            },
                            _ => (),
                        }
                    },
                    None => {
                        if let b"node" = e.name() {
                            let node_element = NodeElement::new(&e);
                            node_map.insert(node_element.id, node_element);

                            node_count += 1;
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                if let b"way" = e.name() {
                    match way_element {
                        Some(ref we) => {
                            if we.is_highway {
                                highway_nodes.extend(&we.nodes);
                                highway_map.insert(we.id, we.clone());
                            }

                            way_element = None;
                            way_count += 1;
                        },
                        None => panic!("WayElement closed without being opened"),
                    }
                }
            },
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }

        buf.clear();
    }

    node_map.retain(|k, _| highway_nodes.contains(k));

    println!("Building directed graph...");
    let mut edge_map: HashMap<i64, DirectedEdge> = HashMap::new();
    for highway in highway_map.values() {
        for i in 0..highway.nodes.len() - 1 {
            let from_opt = node_map.get(&highway.nodes[i]);
            let to_opt = node_map.get(&highway.nodes[i + 1]);

            if let (Some(from), Some(to)) = (from_opt, to_opt) {
                let directed_edge = DirectedEdge::new(from, to);
                edge_map.insert(from.id, directed_edge);

                if !highway.is_oneway {
                    edge_map.insert(to.id, directed_edge.reversed());
                }
            }
        }
    }

    println!(
        "Found {node_count} nodes\n\
        Found {way_count} ways\n\
        Number of WayElement structs in highway_map: {highway_map_len}\n\
        Number of NodeElement structs in highway_nodes: {highway_nodes_len}\n\
        Number of NodeElement structs in nodes_map: {node_map_len}\n\
        Number of DirectedEdge structs in edge_map: {edge_map_len}",
        node_count = node_count,
        way_count = way_count,
        highway_map_len = highway_map.len(),
        highway_nodes_len = highway_nodes.len(),
        node_map_len = node_map.len(),
        edge_map_len = edge_map.len(),
    );

    Ok(())
}
