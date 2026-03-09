mod compare;
mod geojson_reader;
mod osm_reader;
mod tests;

use geo::{
    Area, BooleanOps, BoundingRect, Coord, Contains, Intersects, MultiPolygon, Polygon, Rect,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::process;

/// A named geometry: an area id and its polygon(s).
pub struct NamedArea {
    pub id: String,
    pub geometry: MultiPolygon<f64>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 && args.len() != 3 {
        eprintln!(
            "Usage: {} <input.osm|input.geojson> <width> <height>",
            args[0]
        );
        eprintln!("  Generates boundaries.ser from the input file.");
        eprintln!();
        eprintln!("  input     OSM XML (.osm) or GeoJSON (.json/.geojson) file");
        eprintln!("  width     Raster width (e.g. 360)");
        eprintln!("  height    Raster height (e.g. 180)");
        process::exit(1);
    }

    let filename = &args[1];
    let width: usize = args[2].parse().unwrap_or_else(|e| {
        eprintln!("Invalid width: {e}");
        process::exit(1);
    });
    let height: usize = args[3].parse().unwrap_or_else(|e| {
        eprintln!("Invalid height: {e}");
        process::exit(1);
    });

    let areas = read_input(filename).unwrap_or_else(|e| {
        eprintln!("Error reading {filename}: {e}");
        process::exit(1);
    });

    let excluded = ["FX", "EU", "AQ"];
    let areas: Vec<NamedArea> = areas
        .into_iter()
        .filter(|a| !excluded.contains(&a.id.as_str()))
        .collect();

    eprintln!("Read {} areas, generating {width}x{height} raster...", areas.len());

    let data = generate(width, height, &areas);

    let out = File::create("boundaries.ser").unwrap_or_else(|e| {
        eprintln!("Error creating output file: {e}");
        process::exit(1);
    });
    let mut writer = BufWriter::new(out);
    writer.write_all(&data).unwrap_or_else(|e| {
        eprintln!("Error writing output: {e}");
        process::exit(1);
    });

    eprintln!("Wrote boundaries.ser");
}

fn read_input(filename: &str) -> Result<Vec<NamedArea>, Box<dyn std::error::Error>> {
    if filename.ends_with(".json") || filename.ends_with(".geojson") {
        let mut contents = String::new();
        BufReader::new(File::open(filename)?).read_to_string(&mut contents)?;
        geojson_reader::read(&contents)
    } else if filename.ends_with(".osm") {
        let reader = BufReader::new(File::open(filename)?);
        osm_reader::read(reader)
    } else {
        Err("Input file must be .osm, .json, or .geojson".into())
    }
}

pub fn generate(width: usize, height: usize, areas: &[NamedArea]) -> Vec<u8> {
    let geometry_sizes = calculate_geometry_areas(areas);

    let total_cells = width * height;
    let mut raster_data: Vec<(Vec<String>, Vec<(String, CellPolygon)>)> =
        Vec::with_capacity(total_cells);

    for y in 0..height {
        for x in 0..width {
            let lon_min = -180.0 + 360.0 * x as f64 / width as f64;
            let lat_max = 90.0 - 180.0 * y as f64 / height as f64;
            let lon_max = -180.0 + 360.0 * (x + 1) as f64 / width as f64;
            let lat_min = 90.0 - 180.0 * (y + 1) as f64 / height as f64;

            let cell = create_cell(areas, lon_min, lat_min, lon_max, lat_max);
            raster_data.push(cell);

            let done = y * width + x + 1;
            if done % 100 == 0 || done == total_cells {
                eprint!("\r  {:.1}%", 100.0 * done as f64 / total_cells as f64);
            }
        }
    }
    eprintln!();

    encode_boundaries(width, &geometry_sizes, &raster_data)
}

/// Intermediate representation of clipped polygon data for a cell.
struct CellPolygon {
    outer: Vec<Vec<(u16, u16)>>,
    inner: Vec<Vec<(u16, u16)>>,
}

fn create_cell(
    areas: &[NamedArea],
    lon_min: f64,
    lat_min: f64,
    lon_max: f64,
    lat_max: f64,
) -> (Vec<String>, Vec<(String, CellPolygon)>) {
    let bounds = Rect::new(
        Coord { x: lon_min, y: lat_min },
        Coord { x: lon_max, y: lat_max },
    );
    let bounds_poly = bounds.to_polygon();

    let mut containing_ids = Vec::new();
    let mut intersecting_areas = Vec::new();

    for area in areas {
        if let Some(bbox) = area.geometry.bounding_rect() {
            if !Rect::intersects(&bbox, &bounds) {
                continue;
            }
        }

        if area.geometry.contains(&bounds_poly) {
            containing_ids.push(area.id.clone());
        } else if area.geometry.intersects(&bounds_poly) {
            let intersection = area.geometry.intersection(&bounds_poly);
            for poly in intersection.0 {
                let cell_poly = polygon_to_cell_coords(&poly, lon_min, lat_min, lon_max, lat_max);
                intersecting_areas.push((area.id.clone(), cell_poly));
            }
        }
    }

    (containing_ids, intersecting_areas)
}

fn polygon_to_cell_coords(
    poly: &Polygon<f64>,
    lon_min: f64,
    lat_min: f64,
    lon_max: f64,
    lat_max: f64,
) -> CellPolygon {
    let to_points = |ring: &geo::LineString<f64>| -> Vec<(u16, u16)> {
        // Skip last coord (same as first, but the binary format doesn't store the closing point)
        let coords = ring.coords().collect::<Vec<_>>();
        let n = if coords.len() > 1 {
            coords.len() - 1
        } else {
            coords.len()
        };
        coords[..n]
            .iter()
            .map(|c| {
                let x = ((c.x - lon_min) * 0xffff as f64 / (lon_max - lon_min)) as u16;
                let y = ((c.y - lat_min) * 0xffff as f64 / (lat_max - lat_min)) as u16;
                (x, y)
            })
            .collect()
    };

    let outer = vec![to_points(poly.exterior())];
    let inner = poly.interiors().iter().map(|r| to_points(r)).collect();

    CellPolygon { outer, inner }
}

fn calculate_geometry_areas(areas: &[NamedArea]) -> HashMap<String, f64> {
    areas
        .iter()
        .map(|a| (a.id.clone(), a.geometry.unsigned_area()))
        .collect()
}

/// Encode the raster data into the binary boundaries format.
fn encode_boundaries(
    raster_width: usize,
    geometry_sizes: &HashMap<String, f64>,
    raster: &[(Vec<String>, Vec<(String, CellPolygon)>)],
) -> Vec<u8> {
    let mut buf = Vec::new();

    // version
    buf.extend_from_slice(&2u16.to_be_bytes());

    // geometry sizes
    buf.extend_from_slice(&(geometry_sizes.len() as i32).to_be_bytes());
    for (id, size) in geometry_sizes {
        write_string_to(&mut buf, id);
        buf.extend_from_slice(&size.to_be_bytes());
    }

    // raster width
    buf.extend_from_slice(&(raster_width as i32).to_be_bytes());

    // raster size
    buf.extend_from_slice(&(raster.len() as i32).to_be_bytes());

    for (containing_ids, intersecting_areas) in raster {
        // containing ids
        buf.push(containing_ids.len() as u8);
        for id in containing_ids {
            write_string_to(&mut buf, id);
        }

        // intersecting areas
        buf.push(intersecting_areas.len() as u8);
        for (id, cell_poly) in intersecting_areas {
            write_string_to(&mut buf, id);

            // outer rings
            buf.push(cell_poly.outer.len() as u8);
            for ring in &cell_poly.outer {
                buf.extend_from_slice(&(ring.len() as i32).to_be_bytes());
                for &(x, y) in ring {
                    buf.extend_from_slice(&x.to_be_bytes());
                    buf.extend_from_slice(&y.to_be_bytes());
                }
            }

            // inner rings
            buf.push(cell_poly.inner.len() as u8);
            for ring in &cell_poly.inner {
                buf.extend_from_slice(&(ring.len() as i32).to_be_bytes());
                for &(x, y) in ring {
                    buf.extend_from_slice(&x.to_be_bytes());
                    buf.extend_from_slice(&y.to_be_bytes());
                }
            }
        }
    }

    buf
}

fn write_string_to(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    buf.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
    buf.extend_from_slice(bytes);
}
