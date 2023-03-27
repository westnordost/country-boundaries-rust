use std::fs::File;
use std::io;
use std::io::{BufReader, Read};
use country_boundaries;
use country_boundaries::{CountryBoundaries, LatLon};

// TODO
/*
#[test]
fn issue_12() {
    assert_eq!(
        vec!["HR"],
        boundaries().get_ids(&LatLon::new(45.8, 16.0))
    );
}

fn boundaries() -> io::Result<CountryBoundaries> {
    let buf = BufReader::new(File::open("../data/boundaries360x180.ser")?);
    CountryBoundaries::from(buf.bytes())
}
*/