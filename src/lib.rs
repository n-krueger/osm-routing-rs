extern crate quick_xml;

pub mod config;
mod parser;

use std::error::Error;
use std::collections::HashMap;
use std::collections::HashSet;

use quick_xml::Reader;
use quick_xml::events::Event;

use config::Config;
use parser::elements::NodeElement;
use parser::elements::WayElement;
use parser::graph::DirectedEdge;

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
