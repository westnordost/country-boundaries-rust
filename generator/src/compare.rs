/// Compare two CountryBoundaries instances cell by cell, reporting differences.
#[cfg(test)]
pub mod tests {
    use country_boundaries::{CountryBoundaries, LatLon};
    use std::collections::HashSet;

    /// Compare the query results of two boundaries instances over a grid of test points.
    /// Returns a list of (lat, lon, ids_a, ids_b) for points where they differ.
    pub fn compare_query_results(
        a: &CountryBoundaries,
        b: &CountryBoundaries,
    ) -> Vec<(f64, f64, Vec<String>, Vec<String>)> {
        let mut diffs = Vec::new();
        // Test at 1-degree intervals
        let mut lat = -89.5;
        while lat <= 89.5 {
            let mut lon = -179.5;
            while lon <= 179.5 {
                let pos = LatLon::new(lat, lon).unwrap();
                let ids_a: Vec<String> = a.ids(pos).iter().map(|s| s.to_string()).collect();
                let ids_b: Vec<String> = b.ids(pos).iter().map(|s| s.to_string()).collect();

                // Compare as sets (order may differ due to geometry_sizes HashMap ordering)
                let set_a: HashSet<&str> = ids_a.iter().map(|s| s.as_str()).collect();
                let set_b: HashSet<&str> = ids_b.iter().map(|s| s.as_str()).collect();

                if set_a != set_b {
                    diffs.push((lat, lon, ids_a, ids_b));
                }

                lon += 1.0;
            }
            lat += 1.0;
        }
        diffs
    }

    #[test]
    fn kotlin_and_rust_generators_produce_same_results() {
        let kotlin_path = "/tmp/boundaries.ser";
        let rust_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../boundaries.ser");

        let kotlin_data = match std::fs::read(kotlin_path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Skipping comparison test: cannot read {kotlin_path}: {e}");
                eprintln!("Run the Kotlin generator first:");
                eprintln!("  java -jar generator-all.jar boundaries.osm 180 90");
                return;
            }
        };
        let rust_data = match std::fs::read(rust_path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Skipping comparison test: cannot read {rust_path}: {e}");
                eprintln!("Run the Rust generator first:");
                eprintln!("  cargo run --release -p country-boundaries-generator -- boundaries.osm 180 90");
                return;
            }
        };

        let kotlin = CountryBoundaries::from_reader(kotlin_data.as_slice()).unwrap();
        let rust = CountryBoundaries::from_reader(rust_data.as_slice()).unwrap();

        let diffs = compare_query_results(&kotlin, &rust);

        if !diffs.is_empty() {
            eprintln!("Found {} points with different results:", diffs.len());
            for (lat, lon, kotlin_ids, rust_ids) in &diffs[..diffs.len().min(20)] {
                eprintln!(
                    "  ({lat}, {lon}): kotlin={kotlin_ids:?} rust={rust_ids:?}"
                );
            }
            if diffs.len() > 20 {
                eprintln!("  ... and {} more", diffs.len() - 20);
            }
            panic!(
                "Kotlin and Rust generators produced different results at {} points",
                diffs.len()
            );
        }
    }
}
