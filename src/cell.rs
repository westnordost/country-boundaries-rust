use std::collections::HashSet;

use multipolygon::Point;
use multipolygon::Multipolygon;

pub mod multipolygon;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
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
        self.containing_ids.iter().any(|cid| cid == id)
        ||
        self.intersecting_areas.iter().any(|a| a.0 == id && a.1.covers(point))
    }

    /// Returns whether the given position is in any area with the given `ids`
    pub fn is_in_any(&self, point: Point, ids: &HashSet<&str>) -> bool {
        self.containing_ids
            .iter()
            .any(|containing_id| ids.contains(containing_id.as_str()))
        ||
        self.intersecting_areas
            .iter()
            .any(|a| ids.contains(a.0.as_str()) && a.1.covers(point))
    }

    /// Return all ids of areas that cover the given `position` (in no particular order)
    pub fn get_ids(&self, point: Point) -> Vec<&str> {
        self.intersecting_areas
            .iter()
            .filter(|a| a.1.covers(point))
            .map(|a| a.0.as_str())
            .chain(self.containing_ids.iter().map(String::as_str))
            .collect()
    }

    /// Return all ids of areas that completely cover or partly cover this cell
    pub fn get_all_ids(&self) -> Vec<&str> {
        self.containing_ids
            .iter()
            .map(String::as_str)
            .chain(self.intersecting_areas.iter().map(|a| a.0.as_str()))
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
            Cell { 
                containing_ids: vec![s("A"), s("C")],
                intersecting_areas: vec![]
            }.get_ids(p(0,0))
        );
    }

    #[test]
    fn get_in_geometry_ids() {
        assert_eq!(
            vec!["B"],
            Cell { containing_ids: vec![], intersecting_areas: vec![b()] }.get_ids(p(1, 1))
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
            vec!["B", "A"],
            Cell {
                containing_ids: vec![s("A")],
                intersecting_areas: vec![b()]
            }.get_ids(p(1,1))
        );
    }

    #[test]
    fn get_ally_ids() {
        assert_eq!(
            vec!["A", "B"],
            Cell {
                containing_ids: vec![s("A")],
                intersecting_areas: vec![b()]
            }.get_all_ids()
        );
    }

    #[test]
    fn is_any_definitely() {
        assert!(Cell {
            containing_ids: vec![s("A")],
            intersecting_areas: vec![]
        }.is_in_any(p(0,0), &HashSet::from(["B", "A"])));
    }

    #[test]
    fn is_any_definitely_not() {
        assert!(!Cell {
            containing_ids: vec![s("A")],
            intersecting_areas: vec![]
        }.is_in_any(p(0,0), &HashSet::from(["B"])));
    }

    #[test]
    fn is_in_any_in_geometry() {
        assert!(Cell {
            containing_ids: vec![],
            intersecting_areas: vec![b()]
        }.is_in_any(p(1,1), &HashSet::from(["B"])));
    }

    #[test]
    fn is_in_any_out_of_geometry() {
        assert!(!Cell {
            containing_ids: vec![],
            intersecting_areas: vec![b()]
        }.is_in_any(p(4,4), &HashSet::from(["B"])));
    }

    fn s(val: &str) -> String { String::from(val) }

    fn b() -> (String, Multipolygon) {
        (
            s("B"),
            Multipolygon {
                outer: vec![vec![p(0, 0), p(0, 2), p(2, 2), p(2, 0)]],
                inner: vec![]
            }
        )
    }

    fn p(x: u16, y: u16) -> Point {
        Point { x, y }
    }
}
