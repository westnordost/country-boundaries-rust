use std::collections::HashSet;

use point::Point;
use multipolygon::Multipolygon;

pub mod multipolygon;
pub mod point;

#[derive(Debug)]
/// One cell in the country boundaries grid
pub struct Cell {
    /// Areas that completely cover this cell
    pub containing_ids: Vec<String>,
    /// Id + Areas that only partly cover this cell
    pub intersecting_areas: Vec<(String, Multipolygon)>
}

impl Cell {
    /// Returns whether the given `position` is in the area with the given `id`
    pub fn is_in(&self, point: Point, id: &str) -> bool {
        for containing_id in self.containing_ids.iter() {
            if id == containing_id { return true }
        }
        if !self.intersecting_areas.is_empty() {
            for country in self.intersecting_areas.iter() {
                if id == country.0 {
                    if country.1.covers(&point) { return true }
                }
            }
        }
        false
    }

    /// Returns whether the given position is in any area with the given `ids`
    pub fn is_in_any(&self, point: Point, ids: &HashSet<&str>) -> bool {
        for containing_id in self.containing_ids.iter() {
            if ids.contains(containing_id.as_str()) { return true }
        }
        if !self.intersecting_areas.is_empty() {
            for country in self.intersecting_areas.iter() {
                if ids.contains(country.0.as_str()) {
                    if country.1.covers(&point) { return true }
                }
            }
        }
        false
    }

    /// Return all ids of areas that cover the given `position`
    pub fn get_ids(&self, point: Point) -> Vec<&str> {
        let mut result: Vec<&str> = Vec::with_capacity(self.containing_ids.len());
        result.extend(self.containing_ids.iter().map(|s| s.as_str()));
        if !self.intersecting_areas.is_empty() {
            for country in self.intersecting_areas.iter() {
                if country.1.covers(&point) {
                    result.push(country.0.as_str());
                }
            }
        }
        result
    }

    /// Return all ids of areas that completely cover or partly cover this cell
    pub fn get_all_ids(&self) -> Vec<&str> {
        let mut result: Vec<&str> = Vec::with_capacity(self.containing_ids.len() + self.intersecting_areas.len());
        result.extend(self.containing_ids.iter().map(|s| s.as_str()));
        result.extend(self.intersecting_areas.iter().map(|s| s.0.as_str()));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_definite_ids() {
        assert_eq!(
            vec!["A", "C"],
            Cell { 
                containing_ids: vec![String::from("A"), String::from("C")],
                intersecting_areas: vec![]
            }.get_ids(p(0,0))
        );
    }

    #[test]
    fn get_in_geometry_ids() {
        assert_eq!(
            vec!["B"],
            Cell { containing_ids: vec![], intersecting_areas: vec![b()] }.get_ids(p(1,1))
        )
    }

    #[test]
    fn dont_get_out_of_geometry_ids() {
        assert!(
            Cell { containing_ids: vec![], intersecting_areas: vec![b()] }
                .get_ids(p(4,4))
                .is_empty()
        )
    }

    #[test]
    fn get_definite_and_in_geometry_ids() {
        assert_eq!(
            vec!["A", "B"],
            Cell {
                containing_ids: vec![String::from("A")],
                intersecting_areas: vec![b()]
            }.get_ids(p(1,1))
        );
    }

    #[test]
    fn get_ally_ids() {
        assert_eq!(
            vec!["A", "B"],
            Cell {
                containing_ids: vec![String::from("A")],
                intersecting_areas: vec![b()]
            }.get_all_ids()
        );
    }

    #[test]
    fn is_any_definitely() {
        assert!(
            Cell {
                containing_ids: vec![String::from("A")],
                intersecting_areas: vec![]
            }.is_in_any(p(0,0), &HashSet::from(["B", "A"]))
        );
    }

    #[test]
    fn is_any_definitely_not() {
        assert!(!
            Cell {
                containing_ids: vec![String::from("A")],
                intersecting_areas: vec![]
            }.is_in_any(p(0,0), &HashSet::from(["B"]))
        );
    }

    #[test]
    fn is_in_any_in_geometry() {
        assert!(
            Cell {
                containing_ids: vec![],
                intersecting_areas: vec![b()]
            }.is_in_any(p(1,1), &HashSet::from(["B"]))
        );
    }

    #[test]
    fn is_in_any_out_of_geometry() {
        assert!(!
            Cell {
                containing_ids: vec![],
                intersecting_areas: vec![b()]
            }.is_in_any(p(4,4), &HashSet::from(["B"]))
        );
    }

    fn b() -> (String, Multipolygon) {
        (String::from("B"), Multipolygon {
            outer: vec![vec![p(0, 0), p(0, 2), p(2, 2), p(2, 0)]],
            inner: vec![]
        })
    }

    fn p(x: u16, y: u16) -> Point {
        Point { x, y }
    }
}
