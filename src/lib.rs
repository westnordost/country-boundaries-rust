//! `country-boundaries` is a fast offline reverse geocoder:
//! Find the area in which a geo position is located.
//!
//! It is a port of the [Java library of the same name](https://github.com/westnordost/countryboundaries/),
//! has pretty much the same API and uses the same file format.
//!
//! # Example usage
//!
//! ```
//! # use std::collections::HashSet;
//! # use country_boundaries::{BoundingBox, CountryBoundaries, LatLon};
//! #
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let buf = std::fs::read("./data/boundaries360x180.ser")?.as_slice();
//! let boundaries = CountryBoundaries::from_reader(buf)?;
//!
//! // get country id(s) for Dallas¹
//! assert_eq!(
//!     vec!["US-TX", "US"],
//!     boundaries.ids(LatLon::new(33.0, -97.0)?)
//! );
//!
//! // check that German exclave in Switzerland² is in Germany
//! assert!(
//!     boundaries.is_in(LatLon::new(47.6973, 8.6910)?, "DE")
//! );
//!
//! // check if position is in any country where the first day of the workweek is Saturday. It is
//! // more efficient than calling `is_in` for every id in a row.
//! assert!(
//!     boundaries.is_in_any(
//!         LatLon::new(21.0, 96.0)?,
//!         &HashSet::from(["BD", "DJ", "IR", "PS"])
//!     )
//! );
//!
//! // get which country ids can be found within a bounding box around the Vaalserberg³
//! assert_eq!(
//!     HashSet::from(["DE", "BE", "NL"]),
//!     boundaries.intersecting_ids(BoundingBox::new(50.7679, 5.9865, 50.7358, 6.0599)?)
//! );
//!
//! // get which country ids completely cover a bounding box around the Vaalserberg³
//! assert_eq!(
//!     HashSet::new(),
//!     boundaries.containing_ids(BoundingBox::new(50.7679, 5.9865, 50.7358, 6.0599)?)
//! );
//! #
//! # Ok(())
//! # }
//! ```
//! ¹ [Dallas](https://www.openstreetmap.org?mlat=32.7816&mlon=-96.7954) —
//! ² [German exclave in Switzerland](https://www.openstreetmap.org?mlat=47.6973&mlon=8.6803) —
//! ³ [Vaalserberg](https://www.openstreetmap.org/?mlat=50.754722&mlon=6.020833)
//!
//! How the ids are named and what areas are available depends on the data used. The data used in
//! the examples is the default data (see below).
//!
//! # Data
//!
//! You need to feed the `CountryBoundaries` with data for it to do anything useful.
//! You can generate an own (country) boundaries file from a GeoJson or an
//! [OSM XML](https://wiki.openstreetmap.org/wiki/OSM_XML), using the Java shell application in the
//! `/generator/` folder of the [Java project](https://github.com/westnordost/countryboundaries).
//!
//! ## Default data
//!
//! A default boundaries dataset generated from
//! [this file in the JOSM project](https://josm.openstreetmap.de/export/HEAD/josm/trunk/resources/data/boundaries.osm)
//! is available in the `/data` directory, it is licensed under the
//! [Open Data Commons Open Database License](https://opendatacommons.org/licenses/odbl/) (ODbL),
//! © OpenStreetMap contributors.
//!
//! The dataset can only be as small as it is because the actual country- and state boundaries have
//! been simplified somewhat from their actual boundaries. Generally, it is made to meet the
//! requirements for OpenStreetMap editing:
//!
//! - In respect to its precision, it strives to have at least every settlement and major road on
//!   the right side of the border, in populated areas the precision may be higher. However, it is
//!   oblivious of sea borders and will only return correct results for geo positions on land.
//!
//! - As ids, it uses ISO 3166-1 alpha-2 country codes where available and otherwise ISO 3166-2 for
//!   subdivision codes. The dataset currently includes all subdivisions only for the
//!    🇺🇸 United States, 🇨🇦 Canada, 🇦🇺 Australia, 🇨🇳 China, 🇮🇳 India, 🇫🇲 Micronesia and 🇧🇪 Belgium plus
//!   a few subdivisions of other countries.
//!
//! See the source file for details (you can open it in [JOSM](https://josm.openstreetmap.de/)).

use std::{cmp::min, collections::HashMap, collections::HashSet, io, vec::Vec};
use cell::Cell;
use crate::deserializer::from_reader;

pub use self::latlon::LatLon;
pub use self::bbox::BoundingBox;

mod latlon;
mod bbox;
mod cell;
mod deserializer;
mod error;

#[derive(Debug)]
pub struct CountryBoundaries {
    /// 2-dimensional array of cells
    raster: Vec<Cell>,
    /// width of the raster
    raster_width: usize,
    /// the sizes of the different countries contained
    geometry_sizes: HashMap<String, f64>
}

impl CountryBoundaries {

    /// Create a CountryBoundaries from a stream of bytes.
    pub fn from_reader(reader: impl io::Read) -> io::Result<CountryBoundaries> {
        from_reader(reader)
    }

    /// Returns whether the given `position` is in the region with the given `id`
    ///
    /// # Example
    /// ```
    /// # use country_boundaries::{CountryBoundaries, LatLon};
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let buf = std::fs::read("./data/boundaries360x180.ser")?.as_slice();
    /// # let boundaries = CountryBoundaries::from_reader(buf)?;
    /// assert!(
    ///     boundaries.is_in(LatLon::new(47.6973, 8.6910)?, "DE")
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_in(&self, position: LatLon, id: &str) -> bool {
        self.cell(&position).is_in(&position, id)
    }

    /// Returns whether the given `position` is in any of the regions with the given `ids`.
    ///
    /// # Example
    /// ```
    /// # use country_boundaries::{CountryBoundaries, LatLon};
    /// # use std::collections::HashSet;
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let buf = std::fs::read("./data/boundaries360x180.ser")?.as_slice();
    /// # let boundaries = CountryBoundaries::from_reader(buf)?;
    /// // check if position is in any country where the first day of the workweek is Saturday. It is
    /// // more efficient than calling `is_in` for every id in a row.
    /// assert!(
    ///     boundaries.is_in_any(
    ///         LatLon::new(21.0, 96.0)?,
    ///         &HashSet::from(["BD", "DJ", "IR", "PS"])
    ///     )
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_in_any(&self, position: LatLon, ids: &HashSet<&str>) -> bool {
        self.cell(&position).is_in_any(&position, ids)
    }

    /// Returns the ids of the regions the given `position` is contained in, ordered by size of
    /// the region ascending
    ///
    /// # Example
    /// ```
    /// # use country_boundaries::{CountryBoundaries, LatLon};
    /// # use std::collections::HashSet;
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let buf = std::fs::read("./data/boundaries360x180.ser")?.as_slice();
    /// # let boundaries = CountryBoundaries::from_reader(buf)?;
    /// assert_eq!(
    ///     vec!["US-TX", "US"],
    ///     boundaries.ids(LatLon::new(33.0, -97.0)?)
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn ids(&self, position: LatLon) -> Vec<&str> {
        let mut result = self.cell(&position).get_ids(&position);
        let zero = 0.0;
        result.sort_by(|&a, &b| {
            let a = if let Some(size) = self.geometry_sizes.get(a) { size } else { &zero };
            let b = if let Some(size) = self.geometry_sizes.get(b) { size } else { &zero };
            a.total_cmp(b)
        });
        result
    }

    /// Returns the ids of the regions that fully contain the given bounding box `bounds`.
    /// 
    /// The given bounding box is allowed to wrap around the 180th longitude,
    /// i.e `bounds.min_longitude` = 170 and `bounds.max_longitude` = -170 is fine.
    ///
    /// # Example
    /// ```
    /// # use country_boundaries::{CountryBoundaries, BoundingBox};
    /// # use std::collections::HashSet;
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let buf = std::fs::read("./data/boundaries360x180.ser")?.as_slice();
    /// # let boundaries = CountryBoundaries::from_reader(buf)?;
    /// assert_eq!(
    ///     HashSet::from(["RU-CHU", "RU"]),
    ///     boundaries.containing_ids(BoundingBox::new(66.0, 178.0, 68.0, -178.0)?)
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn containing_ids(&self, bounds: BoundingBox) -> HashSet<&str> {
        let mut ids: HashSet<&str> = HashSet::new();
        let mut first_cell = true;
        for cell in self.cells(&bounds) {
            if first_cell {
                ids.extend(cell.containing_ids.iter().map(|id| id.as_str()));
                first_cell = false;
            } else {
                ids.retain(|&id| cell.containing_ids.iter().any(|containing_id| containing_id == id));
            }
        }
        ids
    }

    /// Returns the ids of the regions that contain or at lest intersect with the given bounding box
    /// `bounds`. 
    /// 
    /// The given bounding box is allowed to wrap around the 180th longitude, 
    /// i.e `bounds.min_longitude` = 170 and `bounds.max_longitude` = -170 is fine.
    ///
    /// # Example
    /// ```
    /// # use country_boundaries::{CountryBoundaries, BoundingBox};
    /// # use std::collections::HashSet;
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let buf = std::fs::read("./data/boundaries360x180.ser")?.as_slice();
    /// # let boundaries = CountryBoundaries::from_reader(buf)?;
    /// assert_eq!(
    ///     HashSet::from(["RU-CHU", "RU", "US-AK", "US"]),
    ///     boundaries.intersecting_ids(BoundingBox::new(50.0, 163.0, 67.0, -150.0)?)
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn intersecting_ids(&self, bounds: BoundingBox) -> HashSet<&str> {
        let mut ids: HashSet<&str> = HashSet::new();
        for cell in self.cells(&bounds) {
            ids.extend(cell.get_all_ids());
        }
        ids
    }

    fn cell(&self, position: &LatLon) -> &Cell {
        let x = self.longitude_to_cell_x(position.longitude());
        let y = self.latitude_to_cell_y(position.latitude());

        &self.raster[y * self.raster_width + x]
    }

    fn longitude_to_cell_x(&self, longitude: f64) -> usize {
        let normalized_longitude = normalize(longitude, -180.0, 360.0);
        min(
            self.raster_width.saturating_sub(1),
            ((self.raster_width as f64) * (180.0 + normalized_longitude) / 360.0).floor() as usize
        )
    }

    fn latitude_to_cell_y(&self, latitude: f64) -> usize {
        let raster_height = self.raster.len() / self.raster_width;
        (((raster_height as f64) * (90.0 - latitude) / 180.0).ceil() as usize).saturating_sub(1)
    }

    fn cells(&self, bounds: &BoundingBox) -> impl Iterator<Item = &Cell> {
        let min_x = self.longitude_to_cell_x(bounds.min_longitude());
        let max_y = self.latitude_to_cell_y(bounds.min_latitude());
        let max_x = self.longitude_to_cell_x(bounds.max_longitude());
        let min_y = self.latitude_to_cell_y(bounds.max_latitude());

        let steps_y = max_y - min_y;
        // might wrap around
        let steps_x = if min_x > max_x { self.raster_width - min_x + max_x } else { max_x - min_x };

        let mut x_step = 0;
        let mut y_step = 0;

        std::iter::from_fn(move || {
            let result = if x_step <= steps_x && y_step <= steps_y {
                let x = (min_x + x_step) % self.raster_width;
                let y = min_y + y_step;
                Some(&self.raster[y * self.raster_width + x])
            } else { None };
            
            if y_step < steps_y {
                y_step += 1;
            } else {
                y_step = 0;
                x_step += 1;
            }

            result
        })
        /* 
        // this would be more elegant and shorter, but it is still experimental

        return std::iter::from_generator(|| {
            for x_step in 0..=steps_x {
                let x = (min_x + x_step) % self.raster_width;
                for y_step in 0..=steps_y {
                    let y = y_step + min_y;
                    yield &self.raster[y * self.raster_width + x];
                }
            }
        })
        */
    }
}

fn normalize(value: f64, start_at: f64, base: f64) -> f64 {
    let mut value = value % base;
    if value < start_at {
        value += base;
    } else if value >= start_at + base {
        value -= base;
    } 
    value
}

#[cfg(test)]
mod tests {
    use crate::LatLon;

    use super::*;

    // just a convenience macro that constructs a cell
    macro_rules! cell {
        ($containing_ids: expr) => {
            Cell {
                containing_ids: $containing_ids.iter().map(|&s| String::from(s)).collect(),
                intersecting_areas: vec![]
            }
        };
        ($containing_ids: expr, $intersecting_areas: expr) => {
            Cell {
                containing_ids: $containing_ids.iter().map(|&s| String::from(s)).collect(),
                intersecting_areas: $intersecting_areas
            }
        }
    }

    fn latlon(latitude: f64, longitude: f64) -> LatLon {
        LatLon::new(latitude, longitude).unwrap()
    }

    fn bbox(min_latitude: f64, min_longitude: f64, max_latitude: f64, max_longitude: f64) -> BoundingBox {
        BoundingBox::new(min_latitude, min_longitude, max_latitude, max_longitude).unwrap()
    }

    #[test]
    fn delegates_to_correct_cell_at_edges() {
        // the world:
        // ┌─┬─┐
        // │A│B│
        // ├─┼─┤
        // │C│D│
        // └─┴─┘
        let boundaries = CountryBoundaries {
            raster: vec![cell!(&["A"]), cell!(&["B"]), cell!(&["C"]), cell!(&["D"])],
            raster_width: 2,
            geometry_sizes: HashMap::new()
        };

        assert_eq!(vec!["C"], boundaries.ids(latlon(-90.0, -180.0)));
        assert_eq!(vec!["C"], boundaries.ids(latlon(-90.0, -90.0)));
        assert_eq!(vec!["C"], boundaries.ids(latlon(-45.0, -180.0)));
        // wrap around
        assert_eq!(vec!["C"], boundaries.ids(latlon(-45.0, 180.0)));
        assert_eq!(vec!["C"], boundaries.ids(latlon(-90.0, 180.0)));

        assert_eq!(vec!["A"], boundaries.ids(latlon(0.0, -180.0)));
        assert_eq!(vec!["A"], boundaries.ids(latlon(45.0, -180.0)));
        assert_eq!(vec!["A"], boundaries.ids(latlon(0.0, -90.0)));
        // wrap around
        assert_eq!(vec!["A"], boundaries.ids(latlon(0.0, 180.0)));
        assert_eq!(vec!["A"], boundaries.ids(latlon(45.0, 180.0)));

        assert_eq!(vec!["B"], boundaries.ids(latlon(0.0, 0.0)));
        assert_eq!(vec!["B"], boundaries.ids(latlon(45.0, 0.0)));
        assert_eq!(vec!["B"], boundaries.ids(latlon(0.0, 90.0)));

        assert_eq!(vec!["D"], boundaries.ids(latlon(-45.0, 0.0)));
        assert_eq!(vec!["D"], boundaries.ids(latlon(-90.0, 0.0)));
        assert_eq!(vec!["D"], boundaries.ids(latlon(-90.0, 90.0)));
    }


    #[test]
    fn no_array_index_out_of_bounds_at_world_edges() {
        let boundaries = CountryBoundaries {
            raster: vec![cell!(&["A"])],
            raster_width: 1,
            geometry_sizes: HashMap::new()
        };

        boundaries.ids(latlon(-90.0, -180.0));
        boundaries.ids(latlon(90.0, 180.0));
        boundaries.ids(latlon(90.0, -180.0));
        boundaries.ids(latlon(-90.0, 180.0));
    }

    #[test]
    fn get_containing_ids_sorted_by_size_ascending() {
        let boundaries = CountryBoundaries {
            raster: vec![cell!(&["D","B","C","A"])],
            raster_width: 1,
            geometry_sizes: HashMap::from([
                (String::from("A"), 10.0),
                (String::from("B"), 15.0),
                (String::from("C"), 100.0),
                (String::from("D"), 800.0),
            ])
        };
        assert_eq!(vec!["A", "B", "C", "D"], boundaries.ids(latlon(1.0, 1.0)));
    }

    #[test]
    fn get_intersecting_ids_in_bbox_is_merged_correctly() {
        let boundaries = CountryBoundaries {
            raster: vec![cell!(&["A"]), cell!(&["B"]), cell!(&["C"]), cell!(&["D","E"])],
            raster_width: 2,
            geometry_sizes: HashMap::new()
        };
        assert_eq!(
            HashSet::from(["A","B","C","D","E"]),
            boundaries.intersecting_ids(bbox(-10.0,-10.0, 10.0,10.0))
        )
    }

    #[test]
    fn get_intersecting_ids_in_bbox_wraps_longitude_correctly() {
        let boundaries = CountryBoundaries {
            raster: vec![cell!(&["A"]), cell!(&["B"]), cell!(&["C"])],
            raster_width: 3,
            geometry_sizes: HashMap::new()
        };
        assert_eq!(
            HashSet::from(["A", "C"]),
            boundaries.intersecting_ids(bbox(0.0, 170.0, 1.0, -170.0))
        )
    }

    #[test]
    fn get_containing_ids_in_bbox_wraps_longitude_correctly() {
        let boundaries = CountryBoundaries {
            raster: vec![cell!(&["A", "B", "C"]),cell!(&["X"]),cell!(&["A", "B"])],
            raster_width: 3,
            geometry_sizes: HashMap::new()
        };
        assert_eq!(
            HashSet::from(["A", "B"]),
            boundaries.containing_ids(bbox(0.0, 170.0, 1.0, -170.0))
        )
    }


    #[test]
    fn get_containing_ids_in_bbox_returns_correct_result_when_one_cell_is_empty() {
        let boundaries = CountryBoundaries {
            raster: vec![cell!(&[] as &[&str; 0]), cell!(&["A"]), cell!(&["A"]), cell!(&["A"])],
            raster_width: 2,
            geometry_sizes: HashMap::new()
        };
        assert!(boundaries.containing_ids(bbox(-10.0, -10.0, 10.0, 10.0)).is_empty())
    }

    #[test]
    fn get_containing_ids_in_bbox_is_merged_correctly() {
        let boundaries = CountryBoundaries {
            raster: vec![
                cell!(&["A","B"]),
                cell!(&["B","A"]),
                cell!(&["C","B","A"]),
                cell!(&["D","A"]),
            ],
            raster_width: 2,
            geometry_sizes: HashMap::new()
        };
        assert_eq!(
            HashSet::from(["A"]),
            boundaries.containing_ids(bbox(-10.0, -10.0, 10.0, 10.0))
        )
    }

    #[test]
    fn get_containing_ids_in_bbox_is_merged_correctly_an_nothing_is_left() {
        let boundaries = CountryBoundaries {
            raster: vec![cell!(&["A"]), cell!(&["B"]), cell!(&["C"]), cell!(&["D"])],
            raster_width: 2,
            geometry_sizes: HashMap::new()
        };

        assert!(
            boundaries.containing_ids(bbox(-10.0, -10.0, 10.0, 10.0)).is_empty()
        )
    }
}