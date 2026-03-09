use country_boundaries::{CountryBoundaries, LatLon};
use std::collections::HashSet;
use std::process::Command;

fn latlon(lat: f64, lon: f64) -> LatLon {
    LatLon::new(lat, lon).unwrap()
}

fn generator_bin() -> String {
    // cargo test builds the binary into the deps directory; use cargo_bin_exe convention
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test binary name
    path.pop(); // remove "deps"
    path.push("country-boundaries-generator");
    path.to_str().unwrap().to_string()
}

fn run_generator(input_path: &str, width: u32, height: u32, work_dir: &std::path::Path) -> Vec<u8> {
    let output = Command::new(generator_bin())
        .args([input_path, &width.to_string(), &height.to_string()])
        .current_dir(work_dir)
        .output()
        .expect("failed to run generator");

    if !output.status.success() {
        panic!(
            "generator failed with status {}:\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    std::fs::read(work_dir.join("boundaries.ser"))
        .expect("generator did not produce boundaries.ser")
}

#[test]
fn generate_from_geojson_and_query() {
    let geojson = r#"{
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "properties": { "id": "DE" },
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[5, 47], [15, 47], [15, 55], [5, 55], [5, 47]]]
                }
            },
            {
                "type": "Feature",
                "properties": { "id": "FR" },
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[-5, 42], [8, 42], [8, 51], [-5, 51], [-5, 42]]]
                }
            }
        ]
    }"#;

    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("test.geojson");
    std::fs::write(&input, geojson).unwrap();

    let data = run_generator(input.to_str().unwrap(), 60, 30, dir.path());
    let boundaries = CountryBoundaries::from_reader(data.as_slice()).unwrap();

    // Center of Germany-only area
    assert_eq!(vec!["DE"], boundaries.ids(latlon(52.0, 13.0)));

    // Center of France-only area
    assert_eq!(vec!["FR"], boundaries.ids(latlon(45.0, 2.0)));

    // Overlap region (both DE and FR cover parts of 47-51°N, 5-8°E)
    let ids: HashSet<&str> = boundaries.ids(latlon(49.0, 7.0)).into_iter().collect();
    assert!(ids.contains("DE"));
    assert!(ids.contains("FR"));

    // Outside both
    assert!(boundaries.ids(latlon(0.0, 0.0)).is_empty());
}

#[test]
fn generate_from_osm_xml_and_query() {
    let osm_xml = r#"<?xml version='1.0' encoding='UTF-8'?>
<osm version='0.6'>
  <node id='1' lat='-10' lon='-10' />
  <node id='2' lat='10' lon='-10' />
  <node id='3' lat='10' lon='10' />
  <node id='4' lat='-10' lon='10' />
  <node id='11' lat='20' lon='20' />
  <node id='12' lat='40' lon='20' />
  <node id='13' lat='40' lon='40' />
  <node id='14' lat='20' lon='40' />
  <way id='100'>
    <nd ref='1'/><nd ref='2'/><nd ref='3'/><nd ref='4'/><nd ref='1'/>
    <tag k='ISO3166-1:alpha2' v='XX'/>
  </way>
  <way id='200'>
    <nd ref='11'/><nd ref='12'/><nd ref='13'/><nd ref='14'/><nd ref='11'/>
    <tag k='ISO3166-2' v='YY-ZZ'/>
  </way>
</osm>"#;

    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("test.osm");
    std::fs::write(&input, osm_xml).unwrap();

    let data = run_generator(input.to_str().unwrap(), 60, 30, dir.path());
    let boundaries = CountryBoundaries::from_reader(data.as_slice()).unwrap();

    assert_eq!(vec!["XX"], boundaries.ids(latlon(0.0, 0.0)));
    assert_eq!(vec!["YY-ZZ"], boundaries.ids(latlon(30.0, 30.0)));
    assert!(boundaries.ids(latlon(60.0, 60.0)).is_empty());
}

#[test]
fn generate_excludes_non_country_ids() {
    // FX, EU, AQ should be excluded by the generator
    let geojson = r#"{
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "properties": { "id": "FX" },
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[-10, -10], [10, -10], [10, 10], [-10, 10], [-10, -10]]]
                }
            },
            {
                "type": "Feature",
                "properties": { "id": "EU" },
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[-10, -10], [10, -10], [10, 10], [-10, 10], [-10, -10]]]
                }
            },
            {
                "type": "Feature",
                "properties": { "id": "AQ" },
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[-90, -90], [90, -90], [90, 90], [-90, 90], [-90, -90]]]
                }
            },
            {
                "type": "Feature",
                "properties": { "id": "DE" },
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[-10, -10], [10, -10], [10, 10], [-10, 10], [-10, -10]]]
                }
            }
        ]
    }"#;

    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("test.geojson");
    std::fs::write(&input, geojson).unwrap();

    let data = run_generator(input.to_str().unwrap(), 6, 3, dir.path());
    let boundaries = CountryBoundaries::from_reader(data.as_slice()).unwrap();

    // Only DE should be present
    assert_eq!(vec!["DE"], boundaries.ids(latlon(0.0, 0.0)));
}

#[test]
fn generate_handles_multipolygon_geojson() {
    let geojson = r#"{
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "properties": { "id": "IS" },
                "geometry": {
                    "type": "MultiPolygon",
                    "coordinates": [
                        [[[-30, -30], [-10, -30], [-10, -10], [-30, -10], [-30, -30]]],
                        [[[10, 10], [30, 10], [30, 30], [10, 30], [10, 10]]]
                    ]
                }
            }
        ]
    }"#;

    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("test.geojson");
    std::fs::write(&input, geojson).unwrap();

    let data = run_generator(input.to_str().unwrap(), 60, 30, dir.path());
    let boundaries = CountryBoundaries::from_reader(data.as_slice()).unwrap();

    // Both disjoint parts should be recognized
    assert_eq!(vec!["IS"], boundaries.ids(latlon(-20.0, -20.0)));
    assert_eq!(vec!["IS"], boundaries.ids(latlon(20.0, 20.0)));
    // Gap between them
    assert!(boundaries.ids(latlon(0.0, 0.0)).is_empty());
}

#[test]
fn generate_invalid_input_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("test.txt");
    std::fs::write(&input, "not valid").unwrap();

    let output = Command::new(generator_bin())
        .args([input.to_str().unwrap(), "6", "3"])
        .current_dir(dir.path())
        .output()
        .expect("failed to run generator");

    assert!(!output.status.success());
}

#[test]
fn generate_missing_args_exits_nonzero() {
    let output = Command::new(generator_bin())
        .output()
        .expect("failed to run generator");

    assert!(!output.status.success());
}
