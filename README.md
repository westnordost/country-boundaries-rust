[![crates.io version](https://img.shields.io/crates/v/country-boundaries.svg)](https://crates.io/crates/country-boundaries)
[![docs.rs docs](https://docs.rs/country-boundaries/badge.svg)](https://docs.rs/country-boundaries)

`country-boundaries` is a fast offline reverse geocoder:
Find the area in which a geo position is located.

It is a port of the [Java library of the same name](https://github.com/westnordost/countryboundaries/),
has pretty much the same API and uses the same file format.

# Copyright and License

© 2023 Tobias Zwick. This library is released under the terms of the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0).

The default data used in the examples is derived from OpenStreetMap and thus licensed under the 
[Open Data Database License](https://opendatacommons.org/licenses/odbl/) (ODbL), © OpenStreetMap contributors.
If you use it, attribution is required.

# Example usage

```rust
use std::collections::HashSet;
use country_boundaries::{BoundingBox, CountryBoundaries, LatLon, BOUNDARIES_ODBL_360X180};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an instance from ODbL licensed data, (c) OpenStreetMap contributors. Treat the value as a singleton.
    let boundaries = CountryBoundaries::from_reader(BOUNDARIES_ODBL_360X180)?;
    // You can read other/own boundaries data with custom raster sizes from other sources also from file. 
    // See section "Data" below.
    
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
        !boundaries.is_in_any(
            LatLon::new(21.0, 96.0)?,
            &HashSet::from(["BD", "DJ", "IR", "PS"])
        )
    );
    
    // get which country ids can be found within the cell(s) that contain a bounding box around the Vaalserberg³
    assert_eq!(
        HashSet::from(["NL", "LU", "DE", "BE", "BE-VLG", "BE-WAL"]),
        boundaries.intersecting_ids(BoundingBox::new(50.6, 5.9, 50.8, 6.1)?)
    );
    
    // get which country ids completely cover a bounding box around the Vaalserberg³
    assert_eq!(
        HashSet::new(),
        boundaries.containing_ids(BoundingBox::new(50.6, 5.9, 50.8, 6.1)?)
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

The default data is generated from 
[this file in the JOSM project](https://josm.openstreetmap.de/export/HEAD/josm/trunk/resources/data/boundaries.osm),
it is licensed under the [Open Database License](https://opendatacommons.org/licenses/odbl/) (ODbL),
© OpenStreetMap contributors. If you use it, attribution is required.

You can also instead generate an own (country) boundaries file from a GeoJson or an
[OSM XML](https://wiki.openstreetmap.org/wiki/OSM_XML), using the Java shell application in the
`/generator/` folder of the [Java project](https://github.com/westnordost/countryboundaries) and use that. For example, 
Natural Earth data is public domain.

## Default data
For your convenience, the default data is included in the distribution as bytes which you can access via the constants
`BOUNDARIES_ODBL_360X180`, `BOUNDARIES_ODBL_180X60` or `BOUNDARIES_ODBL_60X30`. It's all the same data, only different 
raster sizes: The bigger the raster, the bigger the file size but also the faster the queries, see the next section 
about speed for details. The precision is the same.

The default dataset can only be as small as it is because the actual country- and state boundaries have
been simplified somewhat from their actual boundaries. Generally, it is made to meet the requirements for OpenStreetMap 
editing:

- In respect to its precision, it strives to have at least every settlement and major road on
  the correct side of the border, in populated areas the precision may be higher. However, it is
  oblivious of sea borders and will only return correct results for points on land.

- As ids, it uses ISO 3166-1 alpha-2 country codes where available and otherwise ISO 3166-2 for
  subdivision codes. The dataset currently includes all subdivisions only for the
   🇺🇸 United States, 🇨🇦 Canada, 🇦🇺 Australia, 🇨🇳 China, 🇮🇳 India, 🇫🇲 Micronesia and 🇧🇪 Belgium, plus
  a few subdivisions of other countries (mostly autonomous regions), such as the republics and autonomous 
  provinces of 🇷🇺 Russia.

See the source file for details (you can open it in [JOSM](https://josm.openstreetmap.de/)).

# Speed

Querying 100 million random locations on a single thread takes about 10 seconds with a Ryzen 5700X CPU. 

For above measurement, I used a raster of 360x180 (= one cell is 1° in longitude, 1° in latitude). You can choose a smaller 
raster to have a smaller file or choose a bigger raster to have faster queries. According to my tests, a file with a 
raster of 60x30 (= one cell is 6° in longitude and latitude) is about 4 times smaller but queries are about 4 times 
slower.

Files with a raster of 60x30, 180x90 and 360x180 are supplied by default (see above section), but you can also create
files with custom raster sizes.

What makes it that fast is because the boundaries of the source data are split up into a raster, so, point in polygon
checks, if any, only need to be done for the little geometry that is in the cell in which the point is located.

The reason why the library does not directly consume a GeoJSON or similar but only a file generated from it is so that 
the slicing of the source geometry into a raster does not need to be done each time the file is loaded but only once 
before putting the current version of the boundaries into the distribution.