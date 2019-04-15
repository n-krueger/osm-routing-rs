use quick_xml::events::BytesStart;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Way {
    pub id: i64,
    pub is_road: bool,
    pub is_oneway: bool,
    pub nodes: Vec<i64>,
}

impl Way {
    pub fn from_bytes_start(e: &BytesStart) -> Way {
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
        let is_road = false;
        let is_oneway = false;
        let nodes = Vec::new();

        Way { id, is_road, is_oneway, nodes }
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
        if !self.is_road {
            self.is_road = e.attributes()
                .filter_map(
                    |result| {
                        let attribute = result.unwrap();
                        match attribute.key {
                            b"k" => {
                                if &*attribute.value == b"highway" {
                                    Some(true)
                                } else {
                                    Some(false)
                                }
                            },
                            b"v" => {
                                match &*attribute.value {
                                    b"motorway"
                                    | b"trunk"
                                    | b"primary"
                                    | b"secondary"
                                    | b"tertiary"
                                    | b"unclassified"
                                    | b"residential"
                                    | b"motorway_link"
                                    | b"trunk_link"
                                    | b"primary_link"
                                    | b"secondary_link"
                                    | b"tertiary_link"
                                    | b"road" => Some(true),
                                    _ => Some(false)
                                }
                            },
                            _ => None,
                        }
                    }
                )
                .all(|correct| correct);
        }
    }
}
