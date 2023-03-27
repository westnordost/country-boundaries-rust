use crate::LatLon;

#[derive(Debug)]
/** A latitude + longitude but as ints */
pub struct Point {
    /// longitude = x / 1e7
    pub x: i32,
    /// latitude = y / 1e7
    pub y: i32
}

impl Point {
    pub fn new(position: &LatLon) -> Self {
        Point { 
            x: (position.longitude() * 1e7).round() as i32,
            y: (position.latitude() * 1e7).round() as i32
        }
    }
}