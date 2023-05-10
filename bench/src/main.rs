use country_boundaries::{self, CountryBoundaries, LatLon, BOUNDARIES_ODBL_360X180};
use std::fs;
use std::time::Instant;

fn main() {
    let loading_time = Instant::now();
    let boundaries = boundaries();
    println!(
        "Loading data took {:.2} seconds",
        loading_time.elapsed().as_secs_f64()
    );

    let rng = fastrand::Rng::new();
    let checks = 100_000_000;

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
    return CountryBoundaries::from_reader(BOUNDARIES_ODBL_360X180).unwrap();
}

fn latlon(latitude: f64, longitude: f64) -> LatLon {
    LatLon::new(latitude, longitude).unwrap()
}
