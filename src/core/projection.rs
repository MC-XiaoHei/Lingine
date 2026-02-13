use geo::Coord;
use std::f64::consts::PI;

const EARTH_RADIUS: f64 = 6378137.0;

pub struct AdaptiveLtm {
    center_lon: f64,
    center_lat: f64,
    cos_lat: f64,
}

impl AdaptiveLtm {
    pub fn new(center: Coord<f64>) -> Self {
        Self {
            center_lon: center.x,
            center_lat: center.y,
            cos_lat: (center.y * PI / 180.0).cos(),
        }
    }

    pub fn project(&self, lon: f64, lat: f64) -> Coord<f64> {
        let d_lon = (lon - self.center_lon).to_radians();
        let d_lat = (lat - self.center_lat).to_radians();

        let x = d_lon * EARTH_RADIUS * self.cos_lat;
        let y = d_lat * EARTH_RADIUS;

        Coord { x, y }
    }

    pub fn unproject(&self, x: f64, y: f64) -> Coord<f64> {
        let x_raw = x / self.cos_lat;
        let d_lon = (x_raw / EARTH_RADIUS).to_degrees();
        let d_lat = (y / EARTH_RADIUS).to_degrees();

        Coord {
            x: self.center_lon + d_lon,
            y: self.center_lat + d_lat,
        }
    }

    pub fn convergence_angle(&self, lon: f64, lat: f64) -> f64 {
        let d_lon = (lon - self.center_lon).to_radians();
        let phi = lat.to_radians();

        d_lon * phi.sin()
    }
}
