[![CI build](https://github.com/westnordost/country-boundaries-rust/workflows/CI/badge.svg)](https://github.com/westnordost/country-boundaries-rust/actions)
[![crates.io version](https://img.shields.io/crates/v/country-boundaries.svg)](https://crates.io/crates/country-boundaries)
[![docs.rs docs](https://docs.rs/country-boundaries/badge.svg)](https://docs.rs/country-boundaries)

`country-boundaries` is a fast offline reverse geocoder:
Find the area in which a geo position is located.

It is a port of the [Java library of the same name](https://github.com/westnordost/countryboundaries/),
has pretty much the same API and uses the same file format.

# Copyright and License

© 2023 Tobias Zwick. This library is released under the terms of the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0).

# Example usage

Add to your `Cargo.toml`.  When using ODBL features, you must adhere to the ODbL license.
```toml
[dependencies]
country-boundaries = { version = "1", features = ["with_ODBL_licensed_OSM_data_big"] }
```

```rust
use std::collections::HashSet;
use country_boundaries::{BoundingBox, COUNTRY_BOUNDARIES, LatLon};

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // get country id(s) for Dallas¹
    assert_eq!(
        vec!["US-TX", "US"],
        COUNTRY_BOUNDARIES.ids(LatLon::new(33.0, -97.0)?)
    );

    // check that German exclave in Switzerland² is in Germany
    assert!(COUNTRY_BOUNDARIES.is_in(LatLon::new(47.6973, 8.6910)?, "DE"));

    // check if position is in any country where the first day of the workweek is Saturday. It is
    // more efficient than calling `is_in` for every id in a row.
    assert!(
        !COUNTRY_BOUNDARIES.is_in_any(
            LatLon::new(21.0, 96.0)?,
            &HashSet::from(["BD", "DJ", "IR", "PS"])
        )
    );

    // get which country ids can be found within the cell(s) that contain a bounding box around the Vaalserberg³
    assert_eq!(
        HashSet::from(["NL", "LU", "DE", "BE", "BE-VLG", "BE-WAL"]),
        COUNTRY_BOUNDARIES.intersecting_ids(BoundingBox::new(50.6, 5.9, 50.8, 6.1)?)
    );

    // get which country ids completely cover a bounding box around the Vaalserberg³
    assert_eq!(
        HashSet::new(),
        COUNTRY_BOUNDARIES.containing_ids(BoundingBox::new(50.6, 5.9, 50.8, 6.1)?)
    );

    Ok(())
}
```

¹ [Dallas](https://www.openstreetmap.org?mlat=32.7816&mlon=-96.7954) —
² [German exclave in Switzerland](https://www.openstreetmap.org?mlat=47.6973&mlon=8.6803) —
³ [Vaalserberg](https://www.openstreetmap.org/?mlat=50.754722&mlon=6.020833)

How the ids are named and what areas are available depends on the data used. The data used in
the examples is the default data (see below).

# Data

You need to feed the `CountryBoundaries` with data for it to do anything useful.
You can generate an own (country) boundaries file from a GeoJson or an
[OSM XML](https://wiki.openstreetmap.org/wiki/OSM_XML), using the Java shell application in the
`/generator/` folder of the [Java project](https://github.com/westnordost/countryboundaries).

## Default data

A default boundaries dataset generated from
[this file in the JOSM project](https://josm.openstreetmap.de/export/HEAD/josm/trunk/resources/data/boundaries.osm)
is available in the `/data` directory, it is licensed under the
[Open Data Commons Open Database License](https://opendatacommons.org/licenses/odbl/) (ODbL),
© OpenStreetMap contributors.

The dataset can only be as small as it is because the actual country- and state boundaries have
been simplified somewhat from their actual boundaries. Generally, it is made to meet the
requirements for OpenStreetMap editing:

- In respect to its precision, it strives to have at least every settlement and major road on
  the right side of the border, in populated areas the precision may be higher. However, it is
  oblivious of sea borders and will only return correct results for geo positions on land.

- As ids, it uses ISO 3166-1 alpha-2 country codes where available and otherwise ISO 3166-2 for
  subdivision codes. The dataset currently includes all subdivisions only for the
   🇺🇸 United States, 🇨🇦 Canada, 🇦🇺 Australia, 🇨🇳 China, 🇮🇳 India, 🇫🇲 Micronesia and 🇧🇪 Belgium plus
  a few subdivisions of other countries.

See the source file for details (you can open it in [JOSM](https://josm.openstreetmap.de/)).
