use std::collections::HashMap;
use std::io;
use std::io::{ErrorKind, Read};
use crate::cell::Cell;
use crate::cell::multipolygon::Multipolygon;
use crate::cell::multipolygon::Point;
use crate::{CountryBoundaries, Error};

/// Deserialize a `CountryBoundaries` from an IO stream.
///
/// The content of the IO stream is deserialized directly from the stream without being buffered in
/// memory.
///
/// When reading from a source against which short reads are not efficient, such as a [`File`],
/// you will want to apply your own buffering because this function will not buffer the input. See
/// [`io::BufReader`].
pub fn from_reader(mut reader: impl Read) -> io::Result<CountryBoundaries> {
    let version = read_u16(&mut reader)?;
    if version != 2 {
        return Err(io::Error::new(ErrorKind::InvalidData,
            Error::new(format!(
                "Wrong version number '{}' of the boundaries file (expected: '2').\
                 You may need to get the current version of the data.", version
            ))
        ))
    }

    let geometry_sizes_count = read_usize32(&mut reader)?;
    let mut geometry_sizes = HashMap::with_capacity(geometry_sizes_count);
    for _ in 0..geometry_sizes_count {
        let id = read_string(&mut reader)?;
        let size = read_f64(&mut reader)?;
        geometry_sizes.insert(id, size);
    }
    let raster_width = read_usize32(&mut reader)?;
    let raster_size = read_usize32(&mut reader)?;
    let mut raster = Vec::with_capacity(raster_size);
    for _ in 0..raster_size {
        raster.push(read_cell(&mut reader)?);
    }

    Ok(CountryBoundaries { raster, raster_width, geometry_sizes })
}

fn read_cell(reader: &mut impl Read) -> io::Result<Cell> {
    let containing_ids_size = usize::from(read_u8(reader)?);
    let mut containing_ids = Vec::with_capacity(containing_ids_size);
    for _ in 0..containing_ids_size {
        containing_ids.push(read_string(reader)?);
    }
    let intersecting_areas_size = usize::from(read_u8(reader)?);
    let mut intersecting_areas = Vec::with_capacity(intersecting_areas_size);
    for _ in 0..intersecting_areas_size {
        intersecting_areas.push(read_areas(reader)?);
    }
    Ok(Cell { containing_ids, intersecting_areas })
}

fn read_areas(reader: &mut impl Read) -> io::Result<(String, Multipolygon)> {
    let id = read_string(reader)?;
    let outer = read_polygons(reader)?;
    let inner = read_polygons(reader)?;
    Ok((id, Multipolygon { outer, inner }))
}

fn read_polygons(reader: &mut impl Read) -> io::Result<Vec<Vec<Point>>> {
    let size = usize::from(read_u8(reader)?);
    let mut polygons: Vec<Vec<Point>> = Vec::with_capacity(size);
    for _ in 0..size {
        polygons.push(read_ring(reader)?);
    }
    Ok(polygons)
}

fn read_ring(reader: &mut impl Read) -> io::Result<Vec<Point>> {
    let size = read_usize32(reader)?;
    let mut ring = Vec::with_capacity(size);
    for _ in 0..size {
        ring.push(read_point(reader)?);
    }
    Ok(ring)
}

fn read_point(reader: &mut impl Read) -> io::Result<Point> {
    let x = read_u16(reader)?;
    let y = read_u16(reader)?;
    Ok(Point { x, y })
}

fn read_u8(reader: &mut impl Read) -> io::Result<u8> {
    let mut buf = [0; 1];
    reader.read_exact(&mut buf)?;
    Ok(u8::from_be_bytes(buf))
}

fn read_u16(reader: &mut impl Read) -> io::Result<u16> {
    let mut buf = [0; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

fn read_u32(reader: &mut impl Read) -> io::Result<u32> {
    let mut buf = [0; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

fn read_usize32(reader: &mut impl Read) -> io::Result<usize> {
    usize::try_from(read_u32(reader)?).map_err(|e| io::Error::new(ErrorKind::Unsupported, e))
}

fn read_f64(reader: &mut impl Read) -> io::Result<f64> {
    let mut buf = [0; 8];
    reader.read_exact(&mut buf)?;
    Ok(f64::from_be_bytes(buf))
}

fn read_string(reader: &mut impl Read) -> io::Result<String> {
    let length = usize::from(read_u16(reader)?);
    let mut vec: Vec<u8> = vec![0; length];
    reader.read_exact(vec.as_mut_slice())?;
    String::from_utf8(vec).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_string() {
        assert!(read_string(&mut [0x00].as_slice()).is_err());
        assert!(read_string(&mut [0x00, 0x01].as_slice()).is_err());
        assert!(read_string(&mut [0x00, 0x02, 0x41].as_slice()).is_err());

        assert!(read_string(&mut [0x00, 0x00].as_slice()).unwrap().is_empty());
        assert_eq!("A", read_string(&mut [0x00, 0x01, 0x41].as_slice()).unwrap());
        assert_eq!("AB", read_string(&mut [0x00, 0x02, 0x41, 0x42].as_slice()).unwrap());
    }

    #[test]
    fn read_float() {
        assert!(read_f64(&mut [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00].as_slice()).is_err());
        
        assert_eq!(
            12.5, 
            read_f64(&mut [0x40, 0x29, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00].as_slice()).unwrap()
        );
    }

    #[test]
    fn test_read_u8() {
        assert!(read_u8(&mut [].as_slice()).is_err());
        
        assert_eq!(17, read_u8(&mut [0x11].as_slice()).unwrap());
        assert_eq!(u8::MIN, read_u8(&mut [0x00].as_slice()).unwrap());
        assert_eq!(u8::MAX, read_u8(&mut [0xff].as_slice()).unwrap());
    }

    #[test]
    fn test_read_u16() {
        assert!(read_u16(&mut [0x00].as_slice()).is_err());

        assert_eq!(17, read_u16(&mut [0x00, 0x11].as_slice()).unwrap());
        assert_eq!(u16::MIN, read_u16(&mut [0x00, 0x00].as_slice()).unwrap());
        assert_eq!(u16::MAX, read_u16(&mut [0xff, 0xff].as_slice()).unwrap());
    }

    #[test]
    fn test_read_u32() {
        assert!(read_u32(&mut [0x00, 0x00, 0x00].as_slice()).is_err());

        assert_eq!(17, read_u32(&mut [0x00, 0x00, 0x00, 0x11].as_slice()).unwrap());
        assert_eq!(u32::MIN, read_u32(&mut [0x00, 0x00, 0x00, 0x00].as_slice()).unwrap());
        assert_eq!(u32::MAX, read_u32(&mut [0xff, 0xff, 0xff, 0xff].as_slice()).unwrap());
    }

    #[test]
    fn test_read_usize32() {
        assert!(read_usize32(&mut [0x00, 0x00, 0x00].as_slice()).is_err());

        assert_eq!(17, read_usize32(&mut [0x00, 0x00, 0x00, 0x11].as_slice()).unwrap());
        assert_eq!(0, read_usize32(&mut [0x00, 0x00, 0x00, 0x00].as_slice()).unwrap());
        assert_eq!(0xffff, read_usize32(&mut [0x00, 0x00, 0xff, 0xff].as_slice()).unwrap());
    }

    #[test]
    #[cfg(target_pointer_width = "16")]
    fn read_usize32_on_16_bit_machines_results_in_error_if_number_too_big() {
        assert!(read_usize32(&mut [0x00, 0xff, 0xff, 0xff].as_slice()).is_err());
    }

    #[test]
    fn test_read_point() {
        assert_eq!(
            Point {x: 1, y: 2} ,
            read_point(&mut [0x00, 0x01, 0x00, 0x02].as_slice()).unwrap()
        );
    }

    #[test]
    fn test_read_ring() {
        let empty = [0x00, 0x00, 0x00, 0x00];
        for i in 0..empty.len() - 1 { assert!(read_ring(&mut &empty[0..i]).is_err()); }
        assert!(read_ring(&mut empty.as_slice()).unwrap().is_empty());

        let two_points = [
            0x00, 0x00, 0x00, 0x02, // length
            0x00, 0x01,             // p1.x
            0x00, 0x02,             // p1.y
            0x00, 0x03,             // p2.x
            0x00, 0x04              // p2.y
        ];
        for i in 0..two_points.len() - 1 { assert!(read_ring(&mut &two_points[0..i]).is_err()); }
        assert_eq!(
            vec![Point {x: 1, y: 2}, Point {x: 3, y: 4}],
            read_ring(&mut two_points.as_slice()).unwrap()
        );
    }

    #[test]
    fn test_read_polygons() {
        assert!(read_polygons(&mut [0x00].as_slice()).unwrap().is_empty());
        
        let two_rings = [
            0x02,                   // polygons length
            0x00, 0x00, 0x00, 0x01, // ring length
            0x00, 0x01,             // p1.x
            0x00, 0x02,             // p1.y
            0x00, 0x00, 0x00, 0x01, // ring length
            0x00, 0x03,             // p2.x
            0x00, 0x04              // p2.y
        ];
        for i in 0..two_rings.len() - 1 { assert!(read_polygons(&mut &two_rings[0..i]).is_err()); }
        assert_eq!(
            vec![vec![Point {x: 1, y: 2}], vec![Point {x: 3, y: 4}]],
            read_polygons(&mut two_rings.as_slice()).unwrap()
        );
    }

    #[test]
    fn test_read_cell() {
        assert_eq!(
            Cell { containing_ids: vec![], intersecting_areas: vec![] },
            read_cell(&mut [0x00, 0x00].as_slice()).unwrap()
        );
        
        let cell = [
            0x01,             // containing ids length
            0x00, 0x01, 0x41, // "A"
            0x01,             // intersecting areas length
            0x00, 0x01, 0x42, // "B"
            0x00, 0x00        // empty multipolygon
        ];
        for i in 0..cell.len() - 1 { assert!(read_polygons(&mut &cell[0..i]).is_err()); }
        assert_eq!(
            Cell { 
                containing_ids: vec![String::from("A")],
                intersecting_areas: vec![
                    (String::from("B"), Multipolygon { inner: vec![], outer: vec![] })
                ]
            },
            read_cell(&mut cell.as_slice()).unwrap()
        );
    }

    #[test]
    fn test_read_wrong_version() {
        let minimum = [
            0x00, 0x03,                                     // version number
            0x00, 0x00, 0x00, 0x00,                         // geometry sizes map length
            0x00, 0x00, 0x00, 0x00,                         // raster width
            0x00, 0x00, 0x00, 0x00,                         // raster size
        ];
        assert!(from_reader(&mut minimum.as_slice()).is_err());
    }

    #[test]
    fn test_read_minimum() {
        let minimum = [
            0x00, 0x02,             // version number
            0x00, 0x00, 0x00, 0x00, // geometry sizes map length
            0x00, 0x00, 0x00, 0x00, // raster width
            0x00, 0x00, 0x00, 0x00, // raster size
        ];
        for i in 0..minimum.len() - 1 { assert!(from_reader(&mut &minimum[0..i]).is_err()); }
        assert_eq!(
            CountryBoundaries { raster: vec![], raster_width: 0, geometry_sizes: HashMap::new() },
            from_reader(&mut minimum.as_slice()).unwrap()
        );
    }

    #[test]
    fn test_read_basic() {
        let basic = [
            0x00, 0x02,                                     // version number
            0x00, 0x00, 0x00, 0x01,                         // geometry sizes map length
            0x00, 0x01, 0x41,                               // "A"
            0x40, 0x29, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 12.5
            0x00, 0x00, 0x00, 0x01,                         // raster width
            0x00, 0x00, 0x00, 0x01,                         // raster size
            0x01,                                           // cell containing ids length
            0x00, 0x01, 0x41,                               // "A"
            0x00,                                           // intersecting areas length
        ];
        for i in 0..basic.len() - 1 { assert!(from_reader(&mut &basic[0..i]).is_err()); }
        assert_eq!(
            CountryBoundaries { 
                raster: vec![Cell { 
                    containing_ids: vec![String::from("A")],
                    intersecting_areas: vec![]
                }],
                raster_width: 1,
                geometry_sizes: HashMap::from([(String::from("A"), 12.5)])
            },
            from_reader(&mut basic.as_slice()).unwrap()
        );
    }
}
