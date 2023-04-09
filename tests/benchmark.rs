use country_boundaries::{self, CountryBoundaries, LatLon};
use std::fs;
use std::time::Instant;

#[test]
fn benchmark() {
    // Yeah, this is not really a test but some custom benchmark since the #[bench] stuff is still
    // unstable. Be sure to run this with release profile to get the real speeds. Debug profile
    // seems to be about ~4 times slower in the case of this crate at least, i.e. like this
    //
    // cargo test --test benchmark -r -- -Z unstable-options --show-output

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
        _ = rng.f64() * 180.0 - 90.0 + rng.f64() * 360.0 - 180.0;
    }
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
