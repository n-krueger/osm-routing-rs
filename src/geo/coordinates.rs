use super::traits::{Distance, Location};
use crate::geo;

#[derive(Copy, Clone)]
pub struct Coordinates {
    pub lat: f64,
    pub lon: f64,
}

impl Location for Coordinates {
    fn lat(&self) -> f64 {
        self.lat
    }

    fn lon(&self) -> f64 {
        self.lon
    }
}


impl Location for &Coordinates {
    fn lat(&self) -> f64 {
        self.lat
    }

    fn lon(&self) -> f64 {
        self.lon
    }
}

impl Distance for Coordinates {
    fn distance<T: Location>(&self, other: &T) -> u64 {
        geo::distance(self, other)
    }
}

impl Distance for &Coordinates {
    fn distance<T: Location>(&self, other: &T) -> u64 {
        geo::distance(self, other)
    }
}

impl Coordinates {
    pub fn from_string(s: &str) -> Coordinates {
        let split_string: Vec<&str> = s.split(',').collect();
        let lat = split_string[0].parse().expect("Invalid lat for coordinate");
        let lon = split_string[1].parse().expect("Invalid lon for coordinate");

        Coordinates { lat, lon }
    }
}
