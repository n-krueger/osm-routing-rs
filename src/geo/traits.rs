pub trait Location {
    fn lat(&self) -> f64;
    fn lon(&self) -> f64;
}

pub trait Distance {
    fn distance<T: Location>(&self, other: &T) -> f64;
}
