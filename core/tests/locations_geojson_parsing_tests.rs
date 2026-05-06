use gapline_core::models::GeoJsonGeometry;
use gapline_core::parser::error::ParserError;
use gapline_core::parser::file_parsers::locations_geojson;

#[test]
fn parses_valid_polygons() {
    let json = br#"{
        "type": "FeatureCollection",
        "features": [
            {
                "id": "zone-a",
                "type": "Feature",
                "geometry": { "type": "Polygon", "coordinates": [[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]]] },
                "properties": {}
            },
            {
                "id": "zone-b",
                "type": "Feature",
                "geometry": { "type": "Polygon", "coordinates": [[[2.0, 2.0], [3.0, 2.0], [3.0, 3.0], [2.0, 3.0], [2.0, 2.0]]] },
                "properties": {}
            },
            {
                "id": "zone-c",
                "type": "Feature",
                "geometry": { "type": "Polygon", "coordinates": [[[4.0, 4.0], [5.0, 4.0], [5.0, 5.0], [4.0, 5.0], [4.0, 4.0]]] },
                "properties": {}
            }
        ]
    }"#;

    let (locations, errors) = locations_geojson::parse(json).unwrap();
    assert_eq!(locations.len(), 3);
    assert!(errors.is_empty());
    assert_eq!(locations[0].id, "zone-a");
    assert!(matches!(
        locations[0].geometry,
        GeoJsonGeometry::Polygon { .. }
    ));
}

#[test]
fn parses_multipolygon() {
    let json = br#"{
        "type": "FeatureCollection",
        "features": [{
            "id": "multi",
            "type": "Feature",
            "geometry": {
                "type": "MultiPolygon",
                "coordinates": [
                    [[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]]],
                    [[[2.0, 2.0], [3.0, 2.0], [3.0, 3.0], [2.0, 3.0], [2.0, 2.0]]]
                ]
            },
            "properties": {}
        }]
    }"#;

    let (locations, errors) = locations_geojson::parse(json).unwrap();
    assert_eq!(locations.len(), 1);
    assert!(errors.is_empty());
    match &locations[0].geometry {
        GeoJsonGeometry::MultiPolygon { coordinates } => {
            assert_eq!(coordinates.len(), 2);
        }
        other => panic!("expected MultiPolygon, got {other:?}"),
    }
}

#[test]
fn parses_polygon_with_hole() {
    let json = br#"{
        "type": "FeatureCollection",
        "features": [{
            "id": "donut",
            "type": "Feature",
            "geometry": {
                "type": "Polygon",
                "coordinates": [
                    [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0], [0.0, 0.0]],
                    [[2.0, 2.0], [8.0, 2.0], [8.0, 8.0], [2.0, 8.0], [2.0, 2.0]]
                ]
            },
            "properties": {}
        }]
    }"#;

    let (locations, errors) = locations_geojson::parse(json).unwrap();
    assert_eq!(locations.len(), 1);
    assert!(errors.is_empty());
    match &locations[0].geometry {
        GeoJsonGeometry::Polygon { coordinates } => {
            assert_eq!(coordinates.len(), 2, "outer ring + 1 hole");
            assert_eq!(coordinates[0].len(), 5);
            assert_eq!(coordinates[1].len(), 5);
        }
        other => panic!("expected Polygon, got {other:?}"),
    }
}

#[test]
fn invalid_json_returns_clear_error() {
    let json = b"{ this is not json";
    let err = locations_geojson::parse(json).unwrap_err();
    match err {
        ParserError::GeoJson(msg) => assert!(msg.contains("invalid JSON")),
        _ => panic!("expected ParserError::GeoJson"),
    }
}

#[test]
fn root_must_be_feature_collection() {
    let json = br#"{
        "type": "Feature",
        "geometry": { "type": "Polygon", "coordinates": [[[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,0.0]]] },
        "properties": {}
    }"#;
    let err = locations_geojson::parse(json).unwrap_err();
    match err {
        ParserError::GeoJson(msg) => {
            assert!(msg.contains("FeatureCollection"));
            assert!(msg.contains("Feature"));
        }
        _ => panic!("expected ParserError::GeoJson"),
    }
}

#[test]
fn feature_without_id_gets_autogen() {
    let json = br#"{
        "type": "FeatureCollection",
        "features": [{
            "type": "Feature",
            "geometry": { "type": "Polygon", "coordinates": [[[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,0.0]]] },
            "properties": {}
        }]
    }"#;

    let (locations, errors) = locations_geojson::parse(json).unwrap();
    assert!(errors.is_empty());
    assert_eq!(locations.len(), 1);
    assert_eq!(locations[0].id, "__autogen_0");
    assert!(locations[0].id_was_generated);
}

#[test]
fn extracts_gtfs_properties() {
    let json = br#"{
        "type": "FeatureCollection",
        "features": [{
            "id": "zone-1",
            "type": "Feature",
            "geometry": { "type": "Polygon", "coordinates": [[[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,0.0]]] },
            "properties": {
                "stop_name": "Downtown Zone",
                "stop_desc": "Main service area",
                "zone_id": "Z1"
            }
        }]
    }"#;

    let (locations, _) = locations_geojson::parse(json).unwrap();
    assert_eq!(locations[0].stop_name(), Some("Downtown Zone"));
    assert_eq!(locations[0].stop_desc(), Some("Main service area"));
    assert_eq!(locations[0].zone_id(), Some("Z1"));
}

#[test]
fn coordinates_with_altitude_are_accepted() {
    let json = br#"{
        "type": "FeatureCollection",
        "features": [{
            "id": "elevated",
            "type": "Feature",
            "geometry": {
                "type": "Polygon",
                "coordinates": [[[0.0, 0.0, 100.0], [1.0, 0.0, 100.0], [1.0, 1.0, 100.0], [0.0, 0.0, 100.0]]]
            },
            "properties": {}
        }]
    }"#;

    let (locations, errors) = locations_geojson::parse(json).unwrap();
    assert!(errors.is_empty());
    assert_eq!(locations.len(), 1);
    match &locations[0].geometry {
        GeoJsonGeometry::Polygon { coordinates } => {
            let p = coordinates[0][0];
            assert!((p.lon - 0.0).abs() < f64::EPSILON);
            assert!((p.lat - 0.0).abs() < f64::EPSILON);
        }
        other => panic!("expected Polygon, got {other:?}"),
    }
}

#[test]
fn unsupported_geometry_kept_as_unsupported_variant() {
    // Point / LineString / etc. are kept in the model as `Unsupported` so
    // section 11 can report them with feature context. The parser itself
    // does not emit a `GeoJsonParseError` for these.
    let json = br#"{
        "type": "FeatureCollection",
        "features": [
            {
                "id": "ok",
                "type": "Feature",
                "geometry": { "type": "Polygon", "coordinates": [[[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,0.0]]] },
                "properties": {}
            },
            {
                "id": "bad",
                "type": "Feature",
                "geometry": { "type": "Point", "coordinates": [0.0, 0.0] },
                "properties": {}
            }
        ]
    }"#;

    let (locations, errors) = locations_geojson::parse(json).unwrap();
    assert_eq!(locations.len(), 2);
    assert!(errors.is_empty());
    match &locations[1].geometry {
        GeoJsonGeometry::Unsupported { type_ } => assert_eq!(type_, "Point"),
        other => panic!("expected Unsupported, got {other:?}"),
    }
}

#[test]
fn numeric_id_is_stringified() {
    let json = br#"{
        "type": "FeatureCollection",
        "features": [{
            "id": 42,
            "type": "Feature",
            "geometry": { "type": "Polygon", "coordinates": [[[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,0.0]]] },
            "properties": {}
        }]
    }"#;

    let (locations, _) = locations_geojson::parse(json).unwrap();
    assert_eq!(locations[0].id, "42");
    assert!(!locations[0].id_was_generated);
}

#[test]
fn empty_feature_collection() {
    let json = br#"{ "type": "FeatureCollection", "features": [] }"#;
    let (locations, errors) = locations_geojson::parse(json).unwrap();
    assert!(locations.is_empty());
    assert!(errors.is_empty());
}
