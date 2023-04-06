use std::collections::HashMap;
use std::io;
use std::io::{ErrorKind, Read};
use crate::cell::Cell;
use crate::cell::multipolygon::Multipolygon;
use crate::cell::point::Point;
use crate::CountryBoundaries;
use crate::error::Error;

/// Deserialize a `CountryBoundaries` from an IO stream.
///
/// The content of the IO stream is deserialized directly from the stream without being buffered in
/// memory.
///
/// When reading from a source against which short reads are not efficient, such as a [`File`],
/// you will want to apply your own buffering because this function will not buffer the input. See
/// [`io::BufReader`].
// TODO: return error on invalid data / sizes
pub fn from_reader(mut reader: impl Read) -> io::Result<CountryBoundaries> {
    let geometry_sizes_count = read_i32(&mut reader)?;
    let mut geometry_sizes = HashMap::with_capacity(geometry_sizes_count as usize);
    for _ in 0..geometry_sizes_count {
        let id = read_string(&mut reader)?;
        let size = read_f64(&mut reader)?;
        geometry_sizes.insert(id, size);
    }
    let raster_width = read_i32(&mut reader)? as usize;
    let raster_size = read_i32(&mut reader)?;
    let mut raster = Vec::with_capacity(raster_size as usize);
    for _ in 0..raster_size {
        raster.push(read_cell(&mut reader)?);
    }

    Ok(CountryBoundaries { raster: raster, raster_width, geometry_sizes })
}

fn read_cell(reader: &mut impl Read) -> io::Result<Cell> {
    let containing_ids_size = read_i32(reader)?;
    let mut containing_ids = Vec::with_capacity(containing_ids_size as usize);
    for _ in 0..containing_ids_size {
        containing_ids.push(read_string(reader)?);
    }
    let intersecting_areas_size = read_i32(reader)?;
    let mut intersecting_areas = Vec::with_capacity(intersecting_areas_size as usize);
    for _ in 0..intersecting_areas_size {
        intersecting_areas.push(read_areas(reader)?);
    }
    Ok(Cell { containing_ids, intersecting_areas })
}

fn read_areas(reader: &mut impl Read) -> io::Result<(String, Multipolygon)> {
    let id = read_string(reader)?;
    let outer = read_polygon(reader)?;
    let inner = read_polygon(reader)?;
    Ok((id, Multipolygon { outer, inner }))
}

fn read_polygon(reader: &mut impl Read) -> io::Result<Vec<Vec<Point>>> {
    let size = read_i32(reader)?;
    let mut polygons: Vec<Vec<Point>> = Vec::with_capacity(size as usize);
    for _ in 0..size {
        polygons.push(read_ring(reader)?);
    }
    Ok(polygons)
}

fn read_ring(reader: &mut impl Read) -> io::Result<Vec<Point>> {
    let size = read_i32(reader)?;
    let mut ring = Vec::with_capacity(size as usize);
    for _ in 0..size {
        ring.push(read_point(reader)?);
    }
    Ok(ring)
}

fn read_point(reader: &mut impl Read) -> io::Result<Point> {
    let x = read_i32(reader)?;
    let y = read_i32(reader)?;
    Ok(Point { x, y })
}

fn read_i16(reader: &mut impl Read) -> io::Result<i16> {
    let mut buf = [0; 2];
    reader.read_exact(&mut buf)?;
    Ok(i16::from_be_bytes(buf))
}

fn read_i32(reader: &mut impl Read) -> io::Result<i32> {
    let mut buf = [0; 4];
    reader.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

fn read_f64(reader: &mut impl Read) -> io::Result<f64> {
    let mut buf = [0; 8];
    reader.read_exact(&mut buf)?;
    Ok(f64::from_be_bytes(buf))
}

fn read_string(reader: &mut impl Read) -> io::Result<String> {
    let length = read_i16(reader)?;
    let mut vec: Vec<u8> = Vec::with_capacity(length as usize);
    reader.read_exact(vec.as_mut_slice())?;
    let result = String::from_utf8(vec);
    return match result {
        Ok(r) => Ok(r),
        Err(e) => Err(io::Error::new(ErrorKind::InvalidData, e))
    }
}
