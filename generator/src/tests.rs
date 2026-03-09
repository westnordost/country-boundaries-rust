#[cfg(test)]
mod tests {
    #[test]
    fn geojson_reader_ignores_features_without_id() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "properties": { "name": "No ID" },
                    "geometry": {
                        "type": "Polygon",
                        "coordinates": [[[-10, -10], [10, -10], [10, 10], [-10, 10], [-10, -10]]]
                    }
                },
                {
                    "type": "Feature",
                    "properties": {},
                    "geometry": {
                        "type": "Polygon",
                        "coordinates": [[[-10, -10], [10, -10], [10, 10], [-10, 10], [-10, -10]]]
                    }
                }
            ]
        }"#;
        let areas = crate::geojson_reader::read(geojson).unwrap();
        assert!(areas.is_empty());
    }

    #[test]
    fn geojson_reader_ignores_non_polygon_features() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "properties": { "id": "PT" },
                    "geometry": {
                        "type": "Point",
                        "coordinates": [0, 0]
                    }
                },
                {
                    "type": "Feature",
                    "properties": { "id": "LN" },
                    "geometry": {
                        "type": "LineString",
                        "coordinates": [[0, 0], [1, 1]]
                    }
                }
            ]
        }"#;
        let areas = crate::geojson_reader::read(geojson).unwrap();
        assert!(areas.is_empty());
    }

    #[test]
    fn osm_reader_parses_simple_way() {
        let osm_xml = r#"<?xml version='1.0' encoding='UTF-8'?>
<osm version='0.6'>
  <node id='1' lat='0' lon='0' />
  <node id='2' lat='10' lon='0' />
  <node id='3' lat='10' lon='10' />
  <node id='4' lat='0' lon='10' />
  <way id='10'>
    <nd ref='1'/><nd ref='2'/><nd ref='3'/><nd ref='4'/><nd ref='1'/>
    <tag k='ISO3166-1:alpha2' v='XX'/>
  </way>
</osm>"#;
        let areas = crate::osm_reader::read(osm_xml.as_bytes()).unwrap();
        assert_eq!(1, areas.len());
        assert_eq!("XX", areas[0].id);
    }

    #[test]
    fn osm_reader_parses_relation() {
        let osm_xml = r#"<?xml version='1.0' encoding='UTF-8'?>
<osm version='0.6'>
  <node id='1' lat='0' lon='0' />
  <node id='2' lat='10' lon='0' />
  <node id='3' lat='10' lon='10' />
  <node id='4' lat='0' lon='10' />
  <way id='10'>
    <nd ref='1'/><nd ref='2'/><nd ref='3'/><nd ref='4'/><nd ref='1'/>
  </way>
  <relation id='100'>
    <member type='way' ref='10' role='outer'/>
    <tag k='ISO3166-1:alpha2' v='YY'/>
  </relation>
</osm>"#;
        let areas = crate::osm_reader::read(osm_xml.as_bytes()).unwrap();
        // Should have the relation but not the way (way has no name)
        assert_eq!(1, areas.len());
        assert_eq!("YY", areas[0].id);
    }

    #[test]
    fn osm_reader_prefers_alpha2_over_iso3166_2() {
        let osm_xml = r#"<?xml version='1.0' encoding='UTF-8'?>
<osm version='0.6'>
  <node id='1' lat='0' lon='0' />
  <node id='2' lat='10' lon='0' />
  <node id='3' lat='10' lon='10' />
  <node id='4' lat='0' lon='10' />
  <way id='10'>
    <nd ref='1'/><nd ref='2'/><nd ref='3'/><nd ref='4'/><nd ref='1'/>
    <tag k='ISO3166-2' v='US-TX'/>
    <tag k='ISO3166-1:alpha2' v='US'/>
  </way>
</osm>"#;
        let areas = crate::osm_reader::read(osm_xml.as_bytes()).unwrap();
        assert_eq!(1, areas.len());
        assert_eq!("US", areas[0].id);
    }

    #[test]
    fn osm_reader_uses_iso3166_2_as_fallback() {
        let osm_xml = r#"<?xml version='1.0' encoding='UTF-8'?>
<osm version='0.6'>
  <node id='1' lat='0' lon='0' />
  <node id='2' lat='10' lon='0' />
  <node id='3' lat='10' lon='10' />
  <node id='4' lat='0' lon='10' />
  <way id='10'>
    <nd ref='1'/><nd ref='2'/><nd ref='3'/><nd ref='4'/><nd ref='1'/>
    <tag k='ISO3166-2' v='US-TX'/>
  </way>
</osm>"#;
        let areas = crate::osm_reader::read(osm_xml.as_bytes()).unwrap();
        assert_eq!(1, areas.len());
        assert_eq!("US-TX", areas[0].id);
    }
}
