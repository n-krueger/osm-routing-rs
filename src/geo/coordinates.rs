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

impl Distance for Coordinates {
    fn distance<T: Location>(&self, other: &T) -> f64 {
        geo::distance(self, other)
    }
}
