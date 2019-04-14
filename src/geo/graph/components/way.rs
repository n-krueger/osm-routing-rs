use quick_xml::events::BytesStart;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Way {
    pub id: i64,
    pub is_highway: bool,
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
        let is_highway = false;
        let is_oneway = false;
        let nodes = Vec::new();

        Way { id, is_highway, is_oneway, nodes }
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
