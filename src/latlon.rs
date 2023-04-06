use crate::error::Error;

#[derive(Debug, Copy, Clone)]
pub struct LatLon {
    latitude: f64,
    longitude: f64
}

impl LatLon {
    pub fn latitude(&self) -> f64 { self.latitude }
    pub fn longitude(&self) -> f64 { self.longitude }

    /// Creates a new `LatLon` or an error if `latitude` or `longitude` are invalid:
    ///
    /// - `latitude` must be between -90.0 and +90.0
    /// - all parameters must be finite (NaN, Infinite)
    pub fn new(latitude: f64, longitude: f64) -> Result<LatLon, Error> {
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(Error::new(format!(
                "latitude {latitude} is out of bounds, must be within -90.0 and +90.0"
            )))
        }
        if !longitude.is_finite() {
            return Err(Error::new(format!("longitude {longitude} must be finite")))
        }
        Ok(LatLon { latitude, longitude })
    }
}

impl std::fmt::Display for LatLon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.latitude, self.longitude)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn return_errors() {
        assert!(LatLon::new(-90.0001, 0.0).is_err());
        assert!(LatLon::new( 90.0001, 0.0).is_err());

        assert!(LatLon::new(f64::NAN, 0.0).is_err());
        assert!(LatLon::new(0.0, f64::NAN).is_err());

        assert!(LatLon::new(f64::INFINITY, 0.0).is_err());
        assert!(LatLon::new(0.0, f64::INFINITY).is_err());

        assert!(LatLon::new(f64::NEG_INFINITY, 0.0).is_err());
        assert!(LatLon::new(0.0, f64::NEG_INFINITY).is_err());
    }

    #[test]
    fn longitude_can_be_anything() {
        assert!(LatLon::new(0.0, 0.0).is_ok());
        assert!(LatLon::new(0.0, 180.1).is_ok());
        assert!(LatLon::new(0.0, 999999.0).is_ok());
        assert!(LatLon::new(0.0, -180.1).is_ok());
        assert!(LatLon::new(0.0, -99999.0).is_ok());
    }
}
