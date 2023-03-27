use std::fmt;

#[derive(Debug, Copy, Clone)]
pub struct LatLon {
    latitude: f64,
    longitude: f64
}

impl LatLon {
    pub fn latitude(&self) -> f64 { self.latitude }
    pub fn longitude(&self) -> f64 { self.longitude }

    /// Creates a new `LatLon`.
    ///
    /// # Panics
    /// Panics if the `latitude` is not between -90.0 and +90.0 or any is not finite (NaN, Infinite)
    pub fn new(latitude: f64, longitude: f64) -> LatLon {
        if !(-90.0..=90.0).contains(&latitude) {
            panic!("latitude {latitude} is out of bounds, must be within -90.0 and +90.0");
        }
        if !longitude.is_finite() {
            panic!("longitude {longitude} must be finite");
        }
        LatLon { latitude, longitude }
    }
}

impl fmt::Display for LatLon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.latitude, self.longitude)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn latitude_smaller_than_minus_90_panics() { LatLon::new(-90.0001, 0.0); }

    #[test]
    #[should_panic]
    fn latitude_greater_than_plus_90_panics() { LatLon::new(90.0001, 0.0); }

    #[test]
    fn longitude_can_be_anything() {
        LatLon::new(0.0, 0.0);
        LatLon::new(0.0, 180.1);
        LatLon::new(0.0, 999999.0);
        LatLon::new(0.0, -180.1);
        LatLon::new(0.0, -99999.0);
    }

    #[test]
    #[should_panic]
    fn latitude_is_nan_panics() { LatLon::new(f64::NAN, 0.0); }

    #[test]
    #[should_panic]
    fn latitude_is_infinite_panics() { LatLon::new(f64::INFINITY, 0.0); }

    #[test]
    #[should_panic]
    fn latitude_is_neg_infinite_panics() { LatLon::new(f64::NEG_INFINITY, 0.0); }

    #[test]
    #[should_panic]
    fn longitude_is_nan_panics() { LatLon::new(0.0, f64::NAN); }

    #[test]
    #[should_panic]
    fn longitude_is_infinite_panics() { LatLon::new(0.0, f64::INFINITY); }

    #[test]
    #[should_panic]
    fn longitude_is_neg_infinite_panics() { LatLon::new(0.0, f64::NEG_INFINITY); }
}
