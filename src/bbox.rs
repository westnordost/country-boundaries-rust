use crate::error::Error;

#[derive(Debug, Copy, Clone)]
pub struct BoundingBox {
    min_latitude: f64,
    min_longitude: f64,
    max_latitude: f64,
    max_longitude: f64,
}

impl BoundingBox {
    pub fn min_latitude(&self) -> f64 {
        self.min_latitude
    }
    pub fn min_longitude(&self) -> f64 {
        self.min_longitude
    }
    pub fn max_latitude(&self) -> f64 {
        self.max_latitude
    }
    pub fn max_longitude(&self) -> f64 {
        self.max_longitude
    }

    /// Creates a new `BoundingBox` or an error if any of the parameters is invalid or out of range:
    ///
    /// # Errors
    /// - `min_latitude` must be smaller or equal than `max_latitude`
    /// - `min_latitude` and `max_latitude` must be between -90.0 and +90.0
    /// - all parameters must be not finite (neither `NaN` nor `Infinite`)
    pub fn new(
        min_latitude: f64,
        min_longitude: f64,
        max_latitude: f64,
        max_longitude: f64,
    ) -> Result<Self, Error> {
        if !(-90.0..=90.0).contains(&min_latitude) {
            return Err(Error::LatitudeOutOfBounds {
                param: "min_latitude",
                latitude: min_latitude,
            });
        }
        if !(-90.0..=90.0).contains(&max_latitude) {
            return Err(Error::LatitudeOutOfBounds {
                param: "max_latitude",
                latitude: max_latitude,
            });
        }
        if min_latitude > max_latitude {
            return Err(Error::MinLatitudeGreaterThanMaxLatitude {
                min_latitude,
                max_latitude,
            });
        }
        if !min_longitude.is_finite() {
            return Err(Error::LongitudeNotFinite {
                param: "min_longitude",
                longitude: min_longitude,
            });
        }
        if !max_longitude.is_finite() {
            return Err(Error::LongitudeNotFinite {
                param: "max_longitude",
                longitude: max_longitude,
            });
        }
        Ok(Self {
            min_latitude,
            min_longitude,
            max_latitude,
            max_longitude,
        })
    }
}

impl std::fmt::Display for BoundingBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "min: {}, {}, max: {}, {}",
            self.min_latitude, self.min_longitude, self.max_latitude, self.max_longitude
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn return_errors() {
        assert!(BoundingBox::new(-90.0001, 0.0, 0.0, 0.0).is_err());
        assert!(BoundingBox::new(90.0001, 0.0, 0.0, 0.0).is_err());
        assert!(BoundingBox::new(0.0, 0.0, -90.0001, 0.0).is_err());
        assert!(BoundingBox::new(0.0, 0.0, 90.0001, 0.0).is_err());

        assert!(BoundingBox::new(1.1, 0.0, 1.0, 0.0).is_err());

        assert!(BoundingBox::new(f64::NAN, 0.0, 0.0, 0.0).is_err());
        assert!(BoundingBox::new(0.0, f64::NAN, 0.0, 0.0).is_err());
        assert!(BoundingBox::new(0.0, 0.0, f64::NAN, 0.0).is_err());
        assert!(BoundingBox::new(0.0, 0.0, 0.0, f64::NAN).is_err());

        assert!(BoundingBox::new(f64::INFINITY, 0.0, 0.0, 0.0).is_err());
        assert!(BoundingBox::new(0.0, f64::INFINITY, 0.0, 0.0).is_err());
        assert!(BoundingBox::new(0.0, 0.0, f64::INFINITY, 0.0).is_err());
        assert!(BoundingBox::new(0.0, 0.0, 0.0, f64::INFINITY).is_err());

        assert!(BoundingBox::new(f64::NEG_INFINITY, 0.0, 0.0, 0.0).is_err());
        assert!(BoundingBox::new(0.0, f64::NEG_INFINITY, 0.0, 0.0).is_err());
        assert!(BoundingBox::new(0.0, 0.0, f64::NEG_INFINITY, 0.0).is_err());
        assert!(BoundingBox::new(0.0, 0.0, 0.0, f64::NEG_INFINITY).is_err());
    }

    #[test]
    fn return_bbox() {
        assert!(BoundingBox::new(1.0, 0.0, 1.1, 0.0).is_ok());
        assert!(BoundingBox::new(-90.0, 0.0, 90.0, 0.0).is_ok());
    }

    #[test]
    fn bbox_may_have_0_size() {
        assert!(BoundingBox::new(0.0, 0.0, 0.0, 0.0).is_ok());
    }

    #[test]
    fn bbox_may_have_wrap_around_180_meridian_size() {
        assert!(BoundingBox::new(0.0, 90.0, 0.0, -90.0).is_ok());
    }

    #[test]
    fn longitude_can_be_anything() {
        assert!(BoundingBox::new(0.0, -180.0, 0.0, 0.0).is_ok());
        assert!(BoundingBox::new(0.0, -180.0, 0.0, 180.0).is_ok());
        assert!(BoundingBox::new(0.0, -720.0, 0.0, 999.0).is_ok());
    }
}
