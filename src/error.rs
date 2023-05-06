use std::fmt;

#[derive(Debug)]
pub enum Error {
    LongitudeNotFinite {
        param: &'static str,
        longitude: f64,
    },
    LatitudeNotFinite {
        param: &'static str,
        latitude: f64,
    },
    LatitudeOutOfBounds {
        param: &'static str,
        latitude: f64,
    },
    MinLatitudeGreaterThanMaxLatitude {
        min_latitude: f64,
        max_latitude: f64,
    },
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::LongitudeNotFinite { param, longitude } => {
                write!(f, "{param} {longitude} must be finite")
            }
            Error::LatitudeNotFinite { param, latitude } => {
                write!(f, "{param} {latitude} must be finite")
            }
            Error::LatitudeOutOfBounds { param, latitude } => {
                write!(
                    f,
                    "{param} {latitude} is out of bounds, must be within -90.0 and +90.0"
                )
            }
            Error::MinLatitudeGreaterThanMaxLatitude {
                min_latitude,
                max_latitude,
            } => {
                write!(f, "min_latitude {min_latitude} must not be greater than max_latitude {max_latitude}")
            }
        }
    }
}
