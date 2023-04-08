[![CI build](https://github.com/westnordost/country-boundaries-rust/workflows/CI/badge.svg)](https://github.com/westnordost/country-boundaries-rust/actions)
[![crates.io version](https://img.shields.io/crates/v/country-boundaries.svg)](https://crates.io/crates/country-boundaries)
[![docs.rs docs](https://docs.rs/country-boundaries/badge.svg)](https://docs.rs/country-boundaries)

`country-boundaries` is a fast offline reverse geocoder:
Find the area in which a geo position is located.

It is a port of the [Java library of the same name](https://github.com/westnordost/countryboundaries/),
has pretty much the same API and uses the same file format.

# Example usage

```rust
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use country_boundaries::{BoundingBox, CountryBoundaries, LatLon};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let buf = BufReader::new(File::open("./data/boundaries360x180.ser")?);
    let boundaries = CountryBoundaries::from_reader(buf)?;
    
    // get country id(s) for Dallas¹
    assert_eq!(
        vec!["US-TX", "US"],
        boundaries.ids(LatLon::new(33.0, -97.0)?)
    );
    
        // check that German exclave in Switzerland² is in Germany
    assert!(
        boundaries.is_in(LatLon::new(47.6973, 8.6910)?, "DE")
    );
    
    // check if position is in any country where the first day of the workweek is Saturday. It is
    // more efficient than calling `is_in` for every id in a row.
    assert!(
        boundaries.is_in_any(
            LatLon::new(21.0, 96.0)?,
            &HashSet::from(["BD", "DJ", "IR", "PS"])
        )
    );
    
    // get which country ids can be found within a bounding box around the Vaalserberg³
    assert_eq!(
        HashSet::from(["DE", "BE", "NL"]),
        boundaries.intersecting_ids(BoundingBox::new(50.7679, 5.9865, 50.7358, 6.0599)?)
    );
    
    // get which country ids completely cover a bounding box around the Vaalserberg³
    assert_eq!(
        HashSet::new(),
        boundaries.containing_ids(BoundingBox::new(50.7679, 5.9865, 50.7358, 6.0599)?)
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
