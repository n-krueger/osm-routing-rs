pub mod elements {
    use quick_xml::events::BytesStart;

    #[derive(Copy, Clone)]
    pub struct NodeElement {
        pub id: i64,
        pub lat: f64,
        pub lon:f64,
    }

    impl NodeElement {
        pub fn new(e: &BytesStart) -> NodeElement {
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

    #[derive(Clone)]
    pub struct WayElement {
        pub id: i64,
        pub is_highway: bool,
        pub is_oneway: bool,
        pub nodes: Vec<i64>,
    }

    impl WayElement {
        pub fn new(e: &BytesStart) -> WayElement {
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
    use super::elements::NodeElement;

    #[derive(Copy, Clone)]
    pub struct DirectedEdge {
        pub from: i64,
        pub to: i64,
        pub distance: f64,
    }

    impl DirectedEdge {
        pub fn new(from: &NodeElement, to: &NodeElement) -> DirectedEdge {
            let distance = NodeElement::distance(from, to);
            DirectedEdge { from: from.id, to: to.id, distance }
        }

        pub fn reversed(&self) -> DirectedEdge {
            DirectedEdge { from: self.to, to: self.from, distance: self.distance }
        }
    }
}
