use country_boundaries::{self, BoundingBox, CountryBoundaries, LatLon};
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;

#[test]
fn return_correct_results_at_cell_edges() {
    let buf = BufReader::new(File::open("./data/boundaries360x180.ser").unwrap());
    let boundaries = CountryBoundaries::from_reader(buf).unwrap();

    // in clockwise order...
    assert_eq!(vec!["HR"], boundaries.ids(latlon(45.5, 16.0)));
    assert_eq!(vec!["HR"], boundaries.ids(latlon(46.0, 16.0)));
    assert_eq!(vec!["HR"], boundaries.ids(latlon(46.0, 16.5)));
    assert_eq!(vec!["HR"], boundaries.ids(latlon(46.0, 17.0)));
    assert_eq!(vec!["HR"], boundaries.ids(latlon(45.5, 17.0)));

    assert_eq!(vec!["BA"], boundaries.ids(latlon(45.0, 17.0)));
    assert_eq!(vec!["BA"], boundaries.ids(latlon(45.0, 16.5)));
    assert_eq!(vec!["BA"], boundaries.ids(latlon(45.0, 16.0)));
}

#[test]
fn containing_ids_at_180th_meridian() {
    let buf = BufReader::new(File::open("./data/boundaries360x180.ser").unwrap());
    let boundaries = CountryBoundaries::from_reader(buf).unwrap();

    assert_eq!(
        HashSet::from(["RU"]),
        boundaries.containing_ids(BoundingBox::new(66.0, 178.0, 68.0, -178.0).unwrap())
    );
}

fn latlon(latitude: f64, longitude: f64) -> LatLon {
    LatLon::new(latitude, longitude).unwrap()
}
