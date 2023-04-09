use std::time::Instant;
use std::collections::HashSet;
use std::fs;
use country_boundaries::{self, BoundingBox, CountryBoundaries, LatLon};



#[test]
fn return_correct_results_at_cell_edges() {
    let boundaries = boundaries();

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
    let boundaries = boundaries();
    
    assert_eq!(
        HashSet::from(["RU"]),
        boundaries.containing_ids(BoundingBox::new(66.0, 178.0, 68.0, -178.0).unwrap())
    );
}

#[test]
fn benchmark() {
    let loading_time = Instant::now();
    let boundaries = boundaries();
    println!("Loading data took {:.2} seconds", loading_time.elapsed().as_secs_f64());

    let rng = fastrand::Rng::new();
    let checks = 10_000_000;

    let time = Instant::now();
    for _ in 0..checks {
        boundaries.ids(latlon(rng.f64() * 180.0 - 90.0, rng.f64() * 360.0 - 180.0));
    }
    // minus time spent on random
    let time_spent_on_random = Instant::now();
    for _ in 0..checks {
        let _ = rng.f64() * 180.0 - 90.0 + rng.f64() * 360.0 - 180.0;
    }
    println!("(Approximately {:.2} seconds spent by random number generator)", time_spent_on_random.elapsed().as_secs_f64());
    let time_spent_on_boundaries = time.elapsed() - time_spent_on_random.elapsed();

    println!(
        "Querying {} random locations took {:.2} seconds - so on average {} nanoseconds",
        checks,
        time_spent_on_boundaries.as_secs_f64(),
        time_spent_on_boundaries.as_nanos() / checks
    )
}

fn boundaries() -> CountryBoundaries {
    let buf = fs::read("./data/boundaries360x180.ser");
    return CountryBoundaries::from_reader(buf.unwrap().as_slice()).unwrap();
}

fn latlon(latitude: f64, longitude: f64) -> LatLon {
    LatLon::new(latitude, longitude).unwrap()
}
