use std::fs;
use country_boundaries;
use country_boundaries::{CountryBoundaries, LatLon};

#[test]
fn issue_12() {
    assert_eq!(
        vec!["HR"],
        boundaries().ids(latlon(45.8, 16.0))
    );
}

fn latlon(latitude: f64, longitude: f64) -> LatLon {
    LatLon::new(latitude, longitude).unwrap()
}

fn boundaries() -> CountryBoundaries {
    CountryBoundaries::from_reader(fs::read("./data/boundaries360x180.ser").unwrap().as_slice()).unwrap()
}
