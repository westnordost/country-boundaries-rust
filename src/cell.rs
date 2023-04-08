use crate::utils::{Multipolygon, Point};
use std::collections::HashSet;

/// One cell in the country boundaries grid
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Cell {
    /// Areas that completely cover this cell
    pub containing_ids: Vec<String>,
    /// Id + Areas that only partly cover this cell
    pub intersecting_areas: Vec<(String, Multipolygon)>,
}

impl Cell {
    pub fn new(
        containing_ids: Vec<String>,
        intersecting_areas: Vec<(String, Multipolygon)>,
    ) -> Self {
        Self {
            containing_ids,
            intersecting_areas,
        }
    }

    /// Returns whether the given `position` is in the area with the given `id`
    pub fn is_in(&self, point: Point, id: &str) -> bool {
        self.containing_ids.iter().any(|v| v == id)
            || self
                .intersecting_areas
                .iter()
                .any(|v| v.0 == id && v.1.covers(point))
    }

    /// Returns whether the given position is in any area with the given `ids`
    pub fn is_in_any(&self, point: Point, ids: &HashSet<&str>) -> bool {
        self.containing_ids.iter().any(|v| ids.contains(v.as_str()))
            || self
                .intersecting_areas
                .iter()
                .any(|v| ids.contains(v.0.as_str()) && v.1.covers(point))
    }

    /// Return all ids of areas that cover the given `position`
    pub fn get_ids(&self, point: Point) -> Vec<&str> {
        // FIXME: capacity should be containing_ids.len() + the count expected from the intersecting_areas
        let mut result: Vec<&str> = Vec::with_capacity(self.containing_ids.len());
        result.extend(self.containing_ids.iter().map(String::as_str));
        for country in &self.intersecting_areas {
            if country.1.covers(point) {
                result.push(country.0.as_str());
            }
        }
        result
    }

    /// Return all ids of areas that completely cover or partly cover this cell
    pub fn get_all_ids(&self) -> Vec<&str> {
        self.containing_ids
            .iter()
            .map(String::as_str)
            .chain(self.intersecting_areas.iter().map(|s| s.0.as_str()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_definite_ids() {
        assert_eq!(
            vec!["A", "C"],
            Cell::new(vec![s("A"), s("C")], vec![]).get_ids(p(0, 0))
        );
    }

    #[test]
    fn get_in_geometry_ids() {
        assert_eq!(vec!["B"], Cell::new(vec![], vec![b()]).get_ids(p(1, 1)))
    }

    #[test]
    fn dont_get_out_of_geometry_ids() {
        assert!(Cell {
            containing_ids: vec![],
            intersecting_areas: vec![b()]
        }
        .get_ids(p(4, 4))
        .is_empty())
    }

    #[test]
    fn get_definite_and_in_geometry_ids() {
        assert_eq!(
            vec!["A", "B"],
            Cell::new(vec![s("A")], vec![b()]).get_ids(p(1, 1))
        );
    }

    #[test]
    fn get_ally_ids() {
        assert_eq!(
            vec!["A", "B"],
            Cell::new(vec![s("A")], vec![b()]).get_all_ids()
        );
    }

    #[test]
    fn is_any_definitely() {
        assert!(Cell {
            containing_ids: vec![s("A")],
            intersecting_areas: vec![]
        }
        .is_in_any(p(0, 0), &HashSet::from(["B", "A"])));
    }

    #[test]
    fn is_any_definitely_not() {
        assert!(!Cell {
            containing_ids: vec![s("A")],
            intersecting_areas: vec![]
        }
        .is_in_any(p(0, 0), &HashSet::from(["B"])));
    }

    #[test]
    fn is_in_any_in_geometry() {
        assert!(Cell {
            containing_ids: vec![],
            intersecting_areas: vec![b()]
        }
        .is_in_any(p(1, 1), &HashSet::from(["B"])));
    }

    #[test]
    fn is_in_any_out_of_geometry() {
        assert!(!Cell {
            containing_ids: vec![],
            intersecting_areas: vec![b()]
        }
        .is_in_any(p(4, 4), &HashSet::from(["B"])));
    }

    fn s(val: &str) -> String {
        // Helper method to keep string creation more concise
        String::from(val)
    }

    fn b() -> (String, Multipolygon) {
        (
            s("B"),
            Multipolygon {
                outer: vec![vec![p(0, 0), p(0, 2), p(2, 2), p(2, 0)]],
                inner: vec![],
            },
        )
    }

    fn p(x: u16, y: u16) -> Point {
        Point { x, y }
    }
}
