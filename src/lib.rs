// Use README.md in a documentation on github, crates.io, and docs site, as well as unit test the examples in it.
// The readme is only included if ran with `cargo doc --all-features` (or a specific feature). Otherwise may not compile.
#![cfg_attr(feature = "lazy_static", doc = include_str!("../README.md"))]

use crate::deserializer::from_reader;
use cell::Cell;
use multipolygon::Point;
use std::{cmp::min, collections::HashMap, collections::HashSet, io, vec::Vec};

pub use self::bbox::BoundingBox;
pub use self::deserializer::ReadError;
pub use self::error::Error;
pub use self::latlon::LatLon;

mod bbox;
mod cell;
mod deserializer;
mod error;
mod latlon;
mod multipolygon;

#[cfg(feature = "lazy_static")]
lazy_static::lazy_static! {
    /// The country boundaries data for the whole world under ODBl license.
    pub static ref COUNTRY_BOUNDARIES: CountryBoundaries = {
        let data = if cfg!(feature = "with_ODBL_licensed_OSM_data_high") {
            include_bytes!("../data/boundaries360x180.ser").as_slice()
        } else if cfg!(feature = "with_ODBL_licensed_OSM_data_mid") {
            include_bytes!("../data/boundaries180x90.ser").as_slice()
        } else if cfg!(feature = "with_ODBL_licensed_OSM_data_low") {
            include_bytes!("../data/boundaries60x30.ser").as_slice()
        } else {
            unreachable!()
        };
        #[cfg(not(any(
            feature = "with_ODBL_licensed_OSM_data_high",
            feature = "with_ODBL_licensed_OSM_data_mid",
            feature = "with_ODBL_licensed_OSM_data_low"
        )))]
        compile_error!("Do not use `lazy_static` feature directly. See README");

        CountryBoundaries::from_reader(data).unwrap()
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryBoundaries {
    /// 2-dimensional array of cells
    raster: Vec<Cell>,
    /// width of the raster
    raster_width: usize,
    /// the sizes of the different countries contained
    geometry_sizes: HashMap<String, f64>,
}

impl CountryBoundaries {
    /// Create a `CountryBoundaries` from a stream of bytes.
    ///
    /// # Errors
    /// Returns an error if the given data is not a valid country boundaries file.
    pub fn from_reader(reader: impl io::Read) -> Result<Self, ReadError> {
        from_reader(reader)
    }

    /// Returns whether the given `position` is in the region with the given `id`
    ///
    /// # Example
    /// ```
    /// # use country_boundaries::{CountryBoundaries, LatLon};
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let buf = std::fs::read("./data/boundaries360x180.ser")?;
    /// # let boundaries = CountryBoundaries::from_reader(buf.as_slice())?;
    /// assert!(boundaries.is_in(LatLon::new(47.6973, 8.6910)?, "DE"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_in(&self, position: LatLon, id: &str) -> bool {
        let (cell, point) = self.cell_and_local_point(position);
        cell.is_in(point, id)
    }

    /// Returns whether the given `position` is in any of the regions with the given `ids`.
    ///
    /// # Example
    /// ```
    /// # use country_boundaries::{CountryBoundaries, LatLon};
    /// # use std::collections::HashSet;
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let buf = std::fs::read("./data/boundaries360x180.ser")?;
    /// # let boundaries = CountryBoundaries::from_reader(buf.as_slice())?;
    /// // check if position is in any country where the first day of the workweek is Saturday. It is
    /// // more efficient than calling `is_in` for every id in a row.
    /// assert!(
    ///     !boundaries.is_in_any(
    ///         LatLon::new(21.0, 96.0)?,
    ///         &HashSet::from(["BD", "DJ", "IR", "PS"])
    ///     )
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_in_any(&self, position: LatLon, ids: &HashSet<&str>) -> bool {
        let (cell, point) = self.cell_and_local_point(position);
        cell.is_in_any(point, ids)
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
    /// # let buf = std::fs::read("./data/boundaries360x180.ser")?;
    /// # let boundaries = CountryBoundaries::from_reader(buf.as_slice())?;
    /// assert_eq!(
    ///     vec!["US-TX", "US"],
    ///     boundaries.ids(LatLon::new(33.0, -97.0)?)
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn ids(&self, position: LatLon) -> Vec<&str> {
        let (cell, point) = self.cell_and_local_point(position);
        let mut result = cell.get_ids(point);
        let zero = 0.0;
        result.sort_by(|&a, &b| {
            let a = self.geometry_sizes.get(a).unwrap_or(&zero);
            let b = self.geometry_sizes.get(b).unwrap_or(&zero);
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
    /// # let buf = std::fs::read("./data/boundaries360x180.ser")?;
    /// # let boundaries = CountryBoundaries::from_reader(buf.as_slice())?;
    /// assert_eq!(
    ///     HashSet::from(["RU"]),
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
                ids.extend(cell.containing_ids.iter().map(String::as_str));
                first_cell = false;
            } else {
                ids.retain(|&id| {
                    cell.containing_ids
                        .iter()
                        .any(|containing_id| containing_id == id)
                });
                if ids.is_empty() {
                    return ids;
                }
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
    /// # let buf = std::fs::read("./data/boundaries360x180.ser")?;
    /// # let boundaries = CountryBoundaries::from_reader(buf.as_slice())?;
    /// assert_eq!(
    ///     HashSet::from(["RU", "US-AK", "US"]),
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

    fn cell_and_local_point(&self, position: LatLon) -> (&Cell, Point) {
        let normalized_longitude = normalize(position.longitude(), -180.0, 360.0);
        let cell_x = self.longitude_to_cell_x(normalized_longitude);
        let cell_y = self.latitude_to_cell_y(position.latitude());

        (
            self.cell(cell_x, cell_y),
            Point {
                x: self.longitude_to_local_x(cell_x, normalized_longitude),
                y: self.latitude_to_local_y(cell_y, position.latitude()),
            },
        )
    }

    fn cell(&self, x: usize, y: usize) -> &Cell {
        &self.raster[y * self.raster_width + x]
    }

    fn longitude_to_cell_x(&self, longitude: f64) -> usize {
        let raster_width = self.raster_width as f64;
        min(
            self.raster_width.saturating_sub(1),
            (raster_width * (180.0 + longitude) / 360.0).floor() as usize,
        )
    }

    fn latitude_to_cell_y(&self, latitude: f64) -> usize {
        let raster_height = (self.raster.len() / self.raster_width) as f64;
        ((raster_height * (90.0 - latitude) / 180.0).ceil() as usize).saturating_sub(1)
    }

    fn longitude_to_local_x(&self, cell_x: usize, longitude: f64) -> u16 {
        let raster_width = self.raster_width as f64;
        let cell_x = cell_x as f64;
        let cell_longitude = -180.0 + 360.0 * cell_x / raster_width;
        ((longitude - cell_longitude) * 360.0 * 0xffff as f64 / raster_width) as u16
    }

    fn latitude_to_local_y(&self, cell_y: usize, latitude: f64) -> u16 {
        let raster_height = (self.raster.len() / self.raster_width) as f64;
        let cell_y = cell_y as f64;
        let cell_latitude = 90.0 - 180.0 * (cell_y + 1.0) / raster_height;
        ((latitude - cell_latitude) * 180.0 * 0xffff as f64 / raster_height) as u16
    }

    fn cells(&self, bounds: &BoundingBox) -> impl Iterator<Item = &Cell> {
        let normalized_min_longitude = normalize(bounds.min_longitude(), -180.0, 360.0);
        let normalized_max_longitude = normalize(bounds.max_longitude(), -180.0, 360.0);

        let min_x = self.longitude_to_cell_x(normalized_min_longitude);
        let max_y = self.latitude_to_cell_y(bounds.min_latitude());
        let max_x = self.longitude_to_cell_x(normalized_max_longitude);
        let min_y = self.latitude_to_cell_y(bounds.max_latitude());

        let steps_y = max_y - min_y;
        // might wrap around
        let steps_x = if min_x > max_x {
            self.raster_width - min_x + max_x
        } else {
            max_x - min_x
        };

        let mut x_step = 0;
        let mut y_step = 0;

        std::iter::from_fn(move || {
            let result = if x_step <= steps_x && y_step <= steps_y {
                let x = (min_x + x_step) % self.raster_width;
                let y = min_y + y_step;
                Some(self.cell(x, y))
            } else {
                None
            };

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
                intersecting_areas: vec![],
            }
        };
        ($containing_ids: expr, $intersecting_areas: expr) => {
            Cell {
                containing_ids: $containing_ids.iter().map(|&s| String::from(s)).collect(),
                intersecting_areas: $intersecting_areas,
            }
        };
    }

    fn latlon(latitude: f64, longitude: f64) -> LatLon {
        LatLon::new(latitude, longitude).unwrap()
    }

    fn bbox(
        min_latitude: f64,
        min_longitude: f64,
        max_latitude: f64,
        max_longitude: f64,
    ) -> BoundingBox {
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
            geometry_sizes: HashMap::new(),
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
            geometry_sizes: HashMap::new(),
        };

        boundaries.ids(latlon(-90.0, -180.0));
        boundaries.ids(latlon(90.0, 180.0));
        boundaries.ids(latlon(90.0, -180.0));
        boundaries.ids(latlon(-90.0, 180.0));
    }

    #[test]
    fn get_containing_ids_sorted_by_size_ascending() {
        let boundaries = CountryBoundaries {
            raster: vec![cell!(&["D", "B", "C", "A"])],
            raster_width: 1,
            geometry_sizes: HashMap::from([
                (String::from("A"), 10.0),
                (String::from("B"), 15.0),
                (String::from("C"), 100.0),
                (String::from("D"), 800.0),
            ]),
        };
        assert_eq!(vec!["A", "B", "C", "D"], boundaries.ids(latlon(1.0, 1.0)));
    }

    #[test]
    fn get_intersecting_ids_in_bbox_is_merged_correctly() {
        let boundaries = CountryBoundaries {
            raster: vec![
                cell!(&["A"]),
                cell!(&["B"]),
                cell!(&["C"]),
                cell!(&["D", "E"]),
            ],
            raster_width: 2,
            geometry_sizes: HashMap::new(),
        };
        assert_eq!(
            HashSet::from(["A", "B", "C", "D", "E"]),
            boundaries.intersecting_ids(bbox(-10.0, -10.0, 10.0, 10.0))
        )
    }

    #[test]
    fn get_intersecting_ids_in_bbox_wraps_longitude_correctly() {
        let boundaries = CountryBoundaries {
            raster: vec![cell!(&["A"]), cell!(&["B"]), cell!(&["C"])],
            raster_width: 3,
            geometry_sizes: HashMap::new(),
        };
        assert_eq!(
            HashSet::from(["A", "C"]),
            boundaries.intersecting_ids(bbox(0.0, 170.0, 1.0, -170.0))
        )
    }

    #[test]
    fn get_containing_ids_in_bbox_wraps_longitude_correctly() {
        let boundaries = CountryBoundaries {
            raster: vec![cell!(&["A", "B", "C"]), cell!(&["X"]), cell!(&["A", "B"])],
            raster_width: 3,
            geometry_sizes: HashMap::new(),
        };
        assert_eq!(
            HashSet::from(["A", "B"]),
            boundaries.containing_ids(bbox(0.0, 170.0, 1.0, -170.0))
        )
    }

    #[test]
    fn get_containing_ids_in_bbox_returns_correct_result_when_one_cell_is_empty() {
        let boundaries = CountryBoundaries {
            raster: vec![
                cell!(&[] as &[&str; 0]),
                cell!(&["A"]),
                cell!(&["A"]),
                cell!(&["A"]),
            ],
            raster_width: 2,
            geometry_sizes: HashMap::new(),
        };
        assert!(boundaries
            .containing_ids(bbox(-10.0, -10.0, 10.0, 10.0))
            .is_empty())
    }

    #[test]
    fn get_containing_ids_in_bbox_is_merged_correctly() {
        let boundaries = CountryBoundaries {
            raster: vec![
                cell!(&["A", "B"]),
                cell!(&["B", "A"]),
                cell!(&["C", "B", "A"]),
                cell!(&["D", "A"]),
            ],
            raster_width: 2,
            geometry_sizes: HashMap::new(),
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
            geometry_sizes: HashMap::new(),
        };

        assert!(boundaries
            .containing_ids(bbox(-10.0, -10.0, 10.0, 10.0))
            .is_empty())
    }
}
