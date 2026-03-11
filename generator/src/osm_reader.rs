use crate::NamedArea;
use geo::{Coord, LineString, MultiPolygon, Polygon, Contains};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::io::BufRead;

/// Read named areas from a JOSM boundaries OSM XML file.
///
/// This is the format used by
/// <https://josm.openstreetmap.de/export/HEAD/josm/trunk/resources/data/boundaries.osm>
pub fn read(reader: impl BufRead) -> Result<Vec<NamedArea>, Box<dyn std::error::Error>> {
    let osm = parse_xml(reader)?;
    let areas = build_areas(&osm);
    Ok(areas)
}

struct OsmData {
    nodes: HashMap<i64, Coord<f64>>,
    ways: HashMap<i64, Way>,
    relations: HashMap<i64, Relation>,
}

struct Way {
    node_refs: Vec<i64>,
    name: Option<String>,
}

struct Relation {
    outer_way_refs: Vec<i64>,
    inner_way_refs: Vec<i64>,
    name: Option<String>,
}

const ISO3166_1_ALPHA2: &str = "ISO3166-1:alpha2";
const ISO3166_2: &str = "ISO3166-2";

fn parse_xml(reader: impl BufRead) -> Result<OsmData, Box<dyn std::error::Error>> {
    let mut xml = Reader::from_reader(reader);
    let mut buf = Vec::new();

    let mut osm = OsmData {
        nodes: HashMap::new(),
        ways: HashMap::new(),
        relations: HashMap::new(),
    };

    // Current element being parsed
    enum Current {
        None,
        Way(i64, Way),
        Relation(i64, Relation),
    }
    let mut current = Current::None;

    loop {
        match xml.read_event_into(&mut buf)? {
            Event::Eof => break,
            Event::Start(ref e) | Event::Empty(ref e) => {
                let name = e.name();
                match name.as_ref() {
                    b"node" => {
                        let mut id = 0i64;
                        let mut lat = 0.0f64;
                        let mut lon = 0.0f64;
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"id" => {
                                    id = std::str::from_utf8(&attr.value)?.parse()?;
                                }
                                b"lat" => {
                                    lat = std::str::from_utf8(&attr.value)?.parse()?;
                                }
                                b"lon" => {
                                    lon = std::str::from_utf8(&attr.value)?.parse()?;
                                }
                                _ => {}
                            }
                        }
                        osm.nodes.insert(id, Coord { x: lon, y: lat });
                    }
                    b"way" => {
                        let mut id = 0i64;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"id" {
                                id = std::str::from_utf8(&attr.value)?.parse()?;
                            }
                        }
                        current = Current::Way(
                            id,
                            Way {
                                node_refs: Vec::new(),
                                name: None,
                            },
                        );
                    }
                    b"relation" => {
                        let mut id = 0i64;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"id" {
                                id = std::str::from_utf8(&attr.value)?.parse()?;
                            }
                        }
                        current = Current::Relation(
                            id,
                            Relation {
                                outer_way_refs: Vec::new(),
                                inner_way_refs: Vec::new(),
                                name: None,
                            },
                        );
                    }
                    b"nd" => {
                        if let Current::Way(_, ref mut way) = current {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"ref" {
                                    let r: i64 = std::str::from_utf8(&attr.value)?.parse()?;
                                    way.node_refs.push(r);
                                }
                            }
                        }
                    }
                    b"member" => {
                        if let Current::Relation(_, ref mut rel) = current {
                            let mut member_type = Vec::new();
                            let mut member_ref = 0i64;
                            let mut role = Vec::new();
                            for attr in e.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"type" => member_type = attr.value.to_vec(),
                                    b"ref" => {
                                        member_ref =
                                            std::str::from_utf8(&attr.value)?.parse()?;
                                    }
                                    b"role" => role = attr.value.to_vec(),
                                    _ => {}
                                }
                            }
                            if member_type == b"way" {
                                if role == b"outer" {
                                    rel.outer_way_refs.push(member_ref);
                                } else if role == b"inner" {
                                    rel.inner_way_refs.push(member_ref);
                                }
                            }
                        }
                    }
                    b"tag" => {
                        let mut key = Vec::new();
                        let mut value = Vec::new();
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"k" => key = attr.value.to_vec(),
                                b"v" => value = attr.value.to_vec(),
                                _ => {}
                            }
                        }
                        let key_str = std::str::from_utf8(&key)?;
                        let value_str = std::str::from_utf8(&value)?;

                        match current {
                            Current::Way(_, ref mut way) => {
                                if key_str == ISO3166_1_ALPHA2
                                    || (way.name.is_none() && key_str == ISO3166_2)
                                {
                                    way.name = Some(value_str.to_string());
                                }
                            }
                            Current::Relation(_, ref mut rel) => {
                                if key_str == ISO3166_1_ALPHA2
                                    || (rel.name.is_none() && key_str == ISO3166_2)
                                {
                                    rel.name = Some(value_str.to_string());
                                }
                            }
                            Current::None => {}
                        }
                    }
                    _ => {}
                }
            }
            Event::End(ref e) => match e.name().as_ref() {
                b"way" => {
                    if let Current::Way(id, way) = std::mem::replace(&mut current, Current::None) {
                        osm.ways.insert(id, way);
                    }
                }
                b"relation" => {
                    if let Current::Relation(id, rel) =
                        std::mem::replace(&mut current, Current::None)
                    {
                        osm.relations.insert(id, rel);
                    }
                }
                _ => {}
            },
            _ => {}
        }
        buf.clear();
    }

    Ok(osm)
}

fn build_areas(osm: &OsmData) -> Vec<NamedArea> {
    let mut areas = Vec::new();

    // Ways that are closed polygons with a name
    for way in osm.ways.values() {
        let name = match &way.name {
            Some(n) => n.clone(),
            None => continue,
        };

        let coords = way_to_coords(way, &osm.nodes);
        if coords.len() < 4 {
            continue;
        }

        let ring = LineString::new(coords);
        let poly = Polygon::new(ring, vec![]);
        areas.push(NamedArea {
            id: name,
            geometry: MultiPolygon(vec![poly]),
        });
    }

    // Relations (multipolygons)
    for rel in osm.relations.values() {
        let name = match &rel.name {
            Some(n) => n.clone(),
            None => continue,
        };

        let outer_rings: Vec<LineString<f64>> = rel
            .outer_way_refs
            .iter()
            .filter_map(|id| osm.ways.get(id))
            .map(|w| LineString::new(way_to_coords(w, &osm.nodes)))
            .filter(|ls| ls.0.len() >= 4)
            .collect();

        let inner_rings: Vec<LineString<f64>> = rel
            .inner_way_refs
            .iter()
            .filter_map(|id| osm.ways.get(id))
            .map(|w| LineString::new(way_to_coords(w, &osm.nodes)))
            .filter(|ls| ls.0.len() >= 4)
            .collect();

        if outer_rings.is_empty() {
            continue;
        }

        let polygons = if outer_rings.len() == 1 {
            vec![Polygon::new(outer_rings.into_iter().next().unwrap(), inner_rings)]
        } else {
            // Multiple outer rings: assign inner rings to the correct outer ring
            let mut polys = Vec::new();
            let mut remaining_inners = inner_rings;

            for outer in &outer_rings {
                let temp_poly = Polygon::new(outer.clone(), vec![]);
                let mut holes = Vec::new();

                remaining_inners.retain(|inner| {
                    if let Some(first) = inner.0.first() {
                        let point = geo::Point::new(first.x, first.y);
                        if temp_poly.contains(&point) {
                            holes.push(inner.clone());
                            return false;
                        }
                    }
                    true
                });

                polys.push(Polygon::new(outer.clone(), holes));
            }
            polys
        };

        areas.push(NamedArea {
            id: name,
            geometry: MultiPolygon(polygons),
        });
    }

    areas
}

fn way_to_coords(way: &Way, nodes: &HashMap<i64, Coord<f64>>) -> Vec<Coord<f64>> {
    way.node_refs
        .iter()
        .filter_map(|id| nodes.get(id).copied())
        .collect()
}
