use std::collections::HashMap;
use std::io;
use crate::cell::Cell;
use crate::cell::multipolygon::Multipolygon;
use crate::cell::point::Point;
use crate::CountryBoundaries;

type Bytes = impl Iterator<Item=Result<u8, io::Error>>;

pub fn deserialize_from_bytes(bytes: &mut Bytes) -> io::Result<CountryBoundaries> {
    let geometry_sizes_count = read_u32(bytes)?;
    let mut geometry_sizes = HashMap::with_capacity(geometry_sizes_count as usize);
    for _ in 0..geometry_sizes_count {
        let id = read_string_fixed_length(bytes)?;
        let size = read_f64(bytes)?;
        geometry_sizes.insert(id, size);
    }
    let raster_width = read_u32(bytes)? as usize;
    let raster_size = read_u32(bytes)?;
    let mut raster = Vec::with_capacity(raster_size as usize);
    for _ in 0..raster_size {
        raster.push(read_cell(bytes)?);
    }

    Ok(CountryBoundaries { raster: raster.into_boxed_slice(), raster_width, geometry_sizes })
}

fn read_cell(bytes: &mut Bytes) -> io::Result<Cell> {
    let containing_ids_size = read_u32(bytes)?;
    let mut containing_ids = Vec::with_capacity(containing_ids_size as usize);
    for _ in 0..containing_ids_size {
        containing_ids.push(read_string_fixed_length(bytes)?);
    }
    let intersecting_areas_size = read_u32(bytes)?;
    let mut intersecting_areas = Vec::with_capacity(intersecting_areas_size as usize);
    for _ in 0..intersecting_areas_size {
        intersecting_areas.push(read_areas(bytes)?);
    }
    Ok(Cell { containing_ids, intersecting_areas })
}

fn read_areas(bytes: &mut Bytes) -> io::Result<(String, Multipolygon)> {
    let id = read_string_fixed_length(bytes)?;
    let outer = read_polygon(bytes)?;
    let inner = read_polygon(bytes)?;
    Ok((id, Multipolygon { outer, inner }))
}

fn read_polygon(bytes: &mut Bytes) -> io::Result<Vec<Vec<Point>>> {
    let size = read_u32(bytes)?;
    let mut polygons: Vec<Vec<Point>> = Vec::with_capacity(size as usize);
    for _ in 0..size {
        polygons.push(read_ring(bytes)?);
    }
    Ok(polygons)
}

fn read_ring(bytes: &mut Bytes) -> io::Result<Vec<Point>> {
    let size = read_u32(bytes)?;
    let mut ring = Vec::with_capacity(size as usize);
    for _ in 0..size {
        ring.push(read_point(bytes)?);
    }
    Ok(ring)
}

fn read_point(bytes: &mut Bytes) -> io::Result<Point> {
    let x = read_i32(bytes)?;
    let y = read_i32(bytes)?;
    Ok(Point { x, y })
}
/*
fn read_u64(bytes: &mut Bytes) -> io::Result<u64> {
    let mut buf = [0; 8];
    bytes.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf)?)
}
*/
fn read_u32(bytes: &mut Bytes) -> io::Result<u32> {
    let mut buf = [0; 4];
    bytes.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

fn read_u16(bytes: &mut Bytes) -> io::Result<u16> {
    let mut buf = [0; 2];
    bytes.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}
/*
fn read_u8(bytes: &mut Bytes) -> io::Result<u8> {
    Ok(bytes.next()?)
}
*/
fn read_i32(bytes: &mut Bytes) -> io::Result<i32> {
    let mut buf = [0; 4];
    bytes.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

fn read_f64(bytes: &mut Bytes) -> io::Result<f64> {
    let mut buf = [0; 8];
    bytes.read_exact(&mut buf)?;
    Ok(f64::from_be_bytes(buf))
}

fn read_string_fixed_length(bytes: &mut Bytes) -> io::Result<String> {
    let length = read_u16(bytes)?;
    let mut str: Vec<u8> = Vec::with_capacity(length as usize);
    for _ in 0 .. length {
        let byte = bytes.next()??; // first ? for Option, second ? for Result
        if byte == 0 { break; }
        else { str.push(byte); }
    }
    Ok(String::from_utf8(str)?)
}
/*
fn read_string_null_terminated(bytes: &mut impl Iterator<Item=u8>) -> io::Result<String> {
    let mut str: Vec<u8> = Vec::new();
    loop {
        let byte = bytes.next()?;
        if byte == 0 { break; }
        else { str.push(byte); }
    }
    Ok(String::from_utf8(str)?)
}
*/