pub mod components;

use std::collections::{HashMap, HashSet};
use std::path::Path;

use quick_xml::Reader;
use quick_xml::events::Event;

use self::components::{Edge, Node, Way};

pub struct Graph {
    pub id_to_node: HashMap<i64, Node>,
    pub id_to_edges: HashMap<i64, Edge>,
}

impl Graph {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Graph {
        let mut way_element: Option<Way> = None;

        let mut highway_nodes: HashSet<i64> = HashSet::new();
        let mut highways: HashSet<Way> = HashSet::new();
        let mut id_to_node: HashMap<i64, Node> = HashMap::new();

        let mut reader = Reader::from_file(path).unwrap();
        let mut buf = Vec::new();

        println!("Reading OSM file... (this may take a while)");

        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if let b"way" = e.name() {
                        match way_element {
                            Some(_) => panic!("Found nested WayElement"),
                            None => way_element = Some(Way::from_bytes_start(e)),
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
                                let node_element = Node::from_bytes_start(&e);
                                id_to_node.insert(node_element.id, node_element);
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
                                    highways.insert(we.clone());
                                }

                                way_element = None;
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

        id_to_node.retain(|k, _| highway_nodes.contains(k));

        println!("Building directed graph...");
        let mut id_to_edges: HashMap<i64, Edge> = HashMap::new();
        for highway in &highways {
            for i in 0..highway.nodes.len() - 1 {
                let from_opt = id_to_node.get(&highway.nodes[i]);
                let to_opt = id_to_node.get(&highway.nodes[i + 1]);

                if let (Some(from), Some(to)) = (from_opt, to_opt) {
                    let directed_edge = Edge::from_node_elements(from, to);
                    id_to_edges.insert(from.id, directed_edge);

                    if !highway.is_oneway {
                        id_to_edges.insert(to.id, directed_edge.reversed());
                    }
                }
            }
        }

        println!(
            "Number of NodeElement structs in id_to_node: {id_to_node_len}\n\
                Number of DirectedEdge structs in id_to_edges: {id_to_edges_len}",
            id_to_node_len = id_to_node.len(),
            id_to_edges_len = id_to_edges.len(),
        );

        Graph { id_to_node, id_to_edges }
    }
}
