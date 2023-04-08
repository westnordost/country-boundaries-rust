use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{param} {longitude} must be finite")]
    LongitudeNotFinite { param: &'static str, longitude: f64 },

    #[error("{param} {latitude} must be finite")]
    LatitudeNotFinite { param: &'static str, latitude: f64 },

    #[error("{param} {latitude} is out of bounds, must be within -90.0 and +90.0")]
    LatitudeOutOfBounds { param: &'static str, latitude: f64 },

    #[error("min_latitude {min_latitude} must not be greater than max_latitude {max_latitude}")]
    MinLatitudeGreaterThanMaxLatitude {
        min_latitude: f64,
        max_latitude: f64,
    },

    #[error("\
    Wrong version number '{actual}' of the boundaries file (expected: '{expected}'). \
    You may need to get the current version of the data."
    )]
    WrongVersionNumber { expected: u16, actual: u16 },

    #[error("Unable to parse usize from '{0}'")]
    UnableToParseUsize(#[from] std::num::TryFromIntError),

    #[error("Unable to decode UTF-8 string from '{0}'")]
    UnableToDecodeUtf8(#[from] std::string::FromUtf8Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
