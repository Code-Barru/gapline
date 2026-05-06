use std::collections::HashMap;

use crate::models::{GeoJsonGeometry, GeoJsonLocation};
use crate::parser::error::{GeoJsonParseError, ParserError};

#[derive(serde::Deserialize)]
struct RawFeatureCollection {
    #[serde(rename = "type")]
    type_: String,
    #[serde(default)]
    features: Vec<RawFeature>,
}

#[derive(serde::Deserialize)]
struct RawFeature {
    #[serde(default)]
    id: Option<serde_json::Value>,
    geometry: serde_json::Value,
    #[serde(default)]
    properties: HashMap<String, serde_json::Value>,
}

/// # Errors
///
/// Returns [`ParserError::GeoJson`] for invalid JSON or a non-`FeatureCollection` root.
pub fn parse(bytes: &[u8]) -> Result<(Vec<GeoJsonLocation>, Vec<GeoJsonParseError>), ParserError> {
    let raw: RawFeatureCollection = serde_json::from_slice(bytes)
        .map_err(|e| ParserError::GeoJson(format!("invalid JSON: {e}")))?;

    if raw.type_ != "FeatureCollection" {
        return Err(ParserError::GeoJson(format!(
            "expected 'FeatureCollection' at root, got '{}'",
            raw.type_
        )));
    }

    let mut out = Vec::with_capacity(raw.features.len());
    let mut errors = Vec::new();

    for (idx, feature) in raw.features.into_iter().enumerate() {
        let (id, id_was_generated) = match feature.id {
            Some(serde_json::Value::String(s)) => (s, false),
            Some(serde_json::Value::Number(n)) => (n.to_string(), false),
            _ => (format!("__autogen_{idx}"), true),
        };

        let geometry = match feature
            .geometry
            .get("type")
            .and_then(serde_json::Value::as_str)
        {
            Some(
                t @ ("Point" | "MultiPoint" | "LineString" | "MultiLineString"
                | "GeometryCollection"),
            ) => GeoJsonGeometry::Unsupported {
                type_: t.to_string(),
            },
            _ => match serde_json::from_value::<GeoJsonGeometry>(feature.geometry) {
                Ok(g) => g,
                Err(e) => {
                    errors.push(GeoJsonParseError {
                        feature_index: Some(idx),
                        message: format!("invalid geometry: {e}"),
                    });
                    continue;
                }
            },
        };

        out.push(GeoJsonLocation {
            id,
            id_was_generated,
            geometry,
            properties: feature.properties,
        });
    }

    Ok((out, errors))
}
