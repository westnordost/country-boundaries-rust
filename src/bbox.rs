use std::fmt;

#[derive(Debug, Copy, Clone)]
pub struct BoundingBox {
    min_latitude: f64,
    min_longitude: f64,
    max_latitude: f64,
    max_longitude: f64
}

impl BoundingBox {
    pub fn min_latitude(&self) -> f64 { self.min_latitude }
    pub fn min_longitude(&self) -> f64 { self.min_longitude }
    pub fn max_latitude(&self) -> f64 { self.max_latitude }
    pub fn max_longitude(&self) -> f64 { self.max_longitude }

    /// Creates a new `BoundingBox`
    ///
    /// # Panics
    /// - if `min_latitude` is not smaller or equal than `max_latitude`
    /// - any of `min_latitude` and `max_latitude` is not between -90.0 and +90.0
    /// - any of parameter is not finite (NaN, Infinite)
    pub fn new(min_latitude: f64, min_longitude: f64, max_latitude: f64, max_longitude: f64) -> BoundingBox {
        if !(-90.0..=90.0).contains(&min_latitude) {
            panic!("min_latitude {min_latitude} is out of bounds, must be within -90.0 and +90.0");
        }
        if !(-90.0..=90.0).contains(&max_latitude) {
            panic!("max_latitude {max_latitude} is out of bounds, must be within -90.0 and +90.0");
        }
        if min_latitude > max_latitude {
            panic!("min_latitude {} must be smaller or equal than max_latitude {}", min_latitude, max_latitude);
        }
        if !min_longitude.is_finite() {
            panic!("min_longitude {min_longitude} must be finite");
        }
        if !max_longitude.is_finite() {
            panic!("max_longitude {max_longitude} must be finite");
        }
        BoundingBox { min_latitude, min_longitude, max_latitude, max_longitude }
    }
}

impl fmt::Display for BoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
               "min: {}, {}, max: {}, {}",
               self.min_latitude, self.min_longitude, self.max_latitude, self.max_longitude
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn min_latitude_smaller_than_minus_90_panics() { BoundingBox::new(-90.0001, 0.0, 0.0, 0.0); }

    #[test]
    #[should_panic]
    fn min_latitude_greater_than_plus_90_panics() { BoundingBox::new(90.0001, 0.0, 0.0, 0.0); }

    #[test]
    #[should_panic]
    fn max_latitude_smaller_than_minus_90_panics() { BoundingBox::new(0.0, 0.0, -90.1, 0.0); }

    #[test]
    #[should_panic]
    fn max_latitude_greater_than_plus_90_panics() { BoundingBox::new(0.0, 0.0, 90.1, 0.0); }

    #[test]
    #[should_panic]
    fn min_latitude_greater_than_max_latitude_panics() { BoundingBox::new(1.1, 0.0, 1.0, 0.0); }

    #[test]
    #[should_panic]
    fn min_latitude_is_nan_panics() { BoundingBox::new(f64::NAN, 0.0, 1.0, 0.0); }

    #[test]
    #[should_panic]
    fn min_latitude_is_infinite_panics() { BoundingBox::new(f64::INFINITY, 0.0, 1.0, 0.0); }

    #[test]
    #[should_panic]
    fn min_latitude_is_neg_infinite_panics() { BoundingBox::new(f64::NEG_INFINITY, 0.0, 1.0, 0.0); }

    #[test]
    #[should_panic]
    fn max_latitude_is_nan_panics() { BoundingBox::new(0.0, 0.0, f64::NAN, 0.0); }

    #[test]
    #[should_panic]
    fn max_latitude_is_infinite_panics() { BoundingBox::new(0.0, 0.0, f64::INFINITY, 0.0); }

    #[test]
    #[should_panic]
    fn max_latitude_is_neg_infinite_panics() { BoundingBox::new(0.0, 0.0, f64::NEG_INFINITY, 0.0); }

    #[test]
    #[should_panic]
    fn min_longitude_is_nan_panics() { BoundingBox::new(0.0, f64::NAN, 0.0, 0.0); }

    #[test]
    #[should_panic]
    fn min_longitude_is_infinite_panics() { BoundingBox::new(0.0, f64::INFINITY, 0.0, 0.0); }

    #[test]
    #[should_panic]
    fn min_longitude_is_neg_infinite_panics() { BoundingBox::new(0.0, f64::NEG_INFINITY, 0.0, 0.0); }

    #[test]
    #[should_panic]
    fn max_longitude_is_nan_panics() { BoundingBox::new(0.0, 0.0, 0.0, f64::NAN); }

    #[test]
    #[should_panic]
    fn max_longitude_is_infinite_panics() { BoundingBox::new(0.0, 0.0, 0.0, f64::INFINITY); }

    #[test]
    #[should_panic]
    fn max_longitude_is_neg_infinite_panics() { BoundingBox::new(0.0, 0.0, 0.0, f64::NEG_INFINITY); }
}
