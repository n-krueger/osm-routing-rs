use quick_xml::events::BytesStart;

use crate::geo;
use crate::geo::traits::{Location, Distance};

#[derive(Copy, Clone, Debug)]
pub struct Node {
    pub id: i64,
    pub lat: f64,
    pub lon:f64,
}

impl Node {
    pub fn from_bytes_start(e: &BytesStart) -> Node {
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
            (Some(id), Some(lat), Some(lon)) => Node { id, lat, lon, },
            _ => panic!("NodeElement is missing id, lat or lon"),
        }
    }
}

impl Location for Node {
    fn lat(&self) -> f64 {
        self.lat
    }

    fn lon(&self) -> f64 {
        self.lon
    }
}

impl Location for &Node {
    fn lat(&self) -> f64 {
        self.lat
    }

    fn lon(&self) -> f64 {
        self.lon
    }
}

impl Distance for Node {
    fn distance<T: Location>(&self, other: &T) -> u64 {
        geo::distance(self, other)
    }
}

impl Distance for &Node {
    fn distance<T: Location>(&self, other: &T) -> u64 {
        geo::distance(self, other)
    }
}
