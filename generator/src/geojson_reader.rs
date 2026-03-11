use crate::NamedArea;
use geo::MultiPolygon;
use geojson::GeoJson;
use std::convert::TryInto;

/// Read named areas from a GeoJSON string.
///
/// Expects a FeatureCollection where each Feature has:
/// - A Polygon or MultiPolygon geometry
/// - A properties object with an `"id"` field (string)
pub fn read(input: &str) -> Result<Vec<NamedArea>, Box<dyn std::error::Error>> {
    let geojson: GeoJson = input.parse()?;
    let collection = match geojson {
        GeoJson::FeatureCollection(fc) => fc,
        _ => return Err("Expected a GeoJSON FeatureCollection".into()),
    };

    let mut areas = Vec::new();

    for feature in collection.features {
        let id = feature
            .properties
            .as_ref()
            .and_then(|p| p.get("id"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let id = match id {
            Some(id) => id,
            None => continue,
        };

        let geometry = match feature.geometry {
            Some(g) => g,
            None => continue,
        };

        let multi: Option<MultiPolygon<f64>> = match geometry.value {
            geojson::Value::Polygon(_) => {
                let poly: Result<geo::Polygon<f64>, _> = geometry.try_into();
                poly.ok().map(|p| MultiPolygon(vec![p]))
            }
            geojson::Value::MultiPolygon(_) => {
                let mp: Result<MultiPolygon<f64>, _> = geometry.try_into();
                mp.ok()
            }
            _ => None,
        };

        if let Some(geometry) = multi {
            areas.push(NamedArea { id, geometry });
        }
    }

    Ok(areas)
}
