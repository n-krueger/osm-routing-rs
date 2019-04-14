pub mod elements {
    use quick_xml::events::BytesStart;

    #[derive(Copy, Clone)]
    pub struct NodeElement {
        pub id: i64,
        pub lat: f64,
        pub lon:f64,
    }

    impl NodeElement {
        pub fn from_bytes_start(e: &BytesStart) -> NodeElement {
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

        pub fn distance(u: &NodeElement, v: &NodeElement) -> f64 {
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

    #[derive(Clone, Eq, PartialEq, Hash)]
    pub struct WayElement {
        pub id: i64,
        pub is_highway: bool,
        pub is_oneway: bool,
        pub nodes: Vec<i64>,
    }

    impl WayElement {
        pub fn from_bytes_start(e: &BytesStart) -> WayElement {
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

        pub fn handle_nd(&mut self, e: &BytesStart) {
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

        pub fn handle_tag(&mut self, e: &BytesStart) {
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
}

pub mod graph {
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::path::Path;

    use quick_xml::events::Event;
    use quick_xml::Reader;

    use super::elements::NodeElement;
    use super::elements::WayElement;

    #[derive(Copy, Clone)]
    pub struct DirectedEdge {
        pub from: i64,
        pub to: i64,
        pub distance: f64,
    }

    impl DirectedEdge {
        pub fn from_node_elements(from: &NodeElement, to: &NodeElement) -> DirectedEdge {
            let distance = NodeElement::distance(from, to);
            DirectedEdge { from: from.id, to: to.id, distance }
        }

        pub fn reversed(&self) -> DirectedEdge {
            DirectedEdge { from: self.to, to: self.from, distance: self.distance }
        }
    }

    pub struct Graph {
        pub node_map: HashMap<i64, NodeElement>,
        pub edge_map: HashMap<i64, DirectedEdge>,
    }

    impl Graph {
        pub fn from_file<P: AsRef<Path>>(path: P) -> Graph {
            let mut way_element: Option<WayElement> = None;

            let mut highway_nodes: HashSet<i64> = HashSet::new();
            let mut highways: HashSet<WayElement> = HashSet::new();
            let mut node_map: HashMap<i64, NodeElement> = HashMap::new();

            let mut reader = Reader::from_file(path).unwrap();
            let mut buf = Vec::new();

            println!("Reading OSM file... (this may take a while)");

            loop {
                match reader.read_event(&mut buf) {
                    Ok(Event::Start(ref e)) => {
                        if let b"way" = e.name() {
                            match way_element {
                                Some(_) => panic!("Found nested WayElement"),
                                None => way_element = Some(WayElement::from_bytes_start(e)),
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
                                    let node_element = NodeElement::from_bytes_start(&e);
                                    node_map.insert(node_element.id, node_element);
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

            node_map.retain(|k, _| highway_nodes.contains(k));

            println!("Building directed graph...");
            let mut edge_map: HashMap<i64, DirectedEdge> = HashMap::new();
            for highway in &highways {
                for i in 0..highway.nodes.len() - 1 {
                    let from_opt = node_map.get(&highway.nodes[i]);
                    let to_opt = node_map.get(&highway.nodes[i + 1]);

                    if let (Some(from), Some(to)) = (from_opt, to_opt) {
                        let directed_edge = DirectedEdge::from_node_elements(from, to);
                        edge_map.insert(from.id, directed_edge);

                        if !highway.is_oneway {
                            edge_map.insert(to.id, directed_edge.reversed());
                        }
                    }
                }
            }

            println!(
                "Number of NodeElement structs in nodes_map: {node_map_len}\n\
                Number of DirectedEdge structs in edge_map: {edge_map_len}",
                node_map_len = node_map.len(),
                edge_map_len = edge_map.len(),
            );

            Graph { node_map, edge_map }
        }
    }
}
