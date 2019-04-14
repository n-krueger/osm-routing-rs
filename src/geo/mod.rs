mod coordinates;
pub mod graph;
pub mod traits;

pub use self::coordinates::Coordinates;
use self::traits::Location;

pub fn distance<U: Location, V: Location>(u: &U, v: &V) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let u_lat = u.lat().to_radians();
    let v_lat = v.lat().to_radians();

    let delta_lat = (v.lat() - u.lat()).to_radians();
    let delta_lon = (v.lon() - u.lon()).to_radians();

    let central_angle_inner = (delta_lat / 2.0).sin().powi(2)
        + u_lat.cos() * v_lat.cos() * (delta_lon / 2.0).sin().powi(2);
    let central_angle = 2.0 * central_angle_inner.sqrt().asin();

    EARTH_RADIUS_KM * central_angle
}
