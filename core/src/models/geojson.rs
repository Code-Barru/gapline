use std::collections::HashMap;

use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub lon: f64,
    pub lat: f64,
}

impl Serialize for Position {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = s.serialize_seq(Some(2))?;
        seq.serialize_element(&self.lon)?;
        seq.serialize_element(&self.lat)?;
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Position {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct PosVisitor;
        impl<'de> Visitor<'de> for PosVisitor {
            type Value = Position;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("array of 2 or 3 numbers [lon, lat] or [lon, lat, alt]")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Position, A::Error> {
                let lon: f64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &"at least 2 elements"))?;
                let lat: f64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &"at least 2 elements"))?;
                while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                Ok(Position { lon, lat })
            }
        }
        d.deserialize_seq(PosVisitor)
    }
}

pub type LinearRing = Vec<Position>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum GeoJsonGeometry {
    Polygon {
        coordinates: Vec<LinearRing>,
    },
    MultiPolygon {
        coordinates: Vec<Vec<LinearRing>>,
    },
    /// Syntactically valid `GeoJSON` types disallowed by GTFS-Locations
    /// (`Point`, `LineString`, …). Constructed by the parser; the
    /// validation layer then reports them with feature context.
    #[serde(skip)]
    Unsupported {
        type_: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoJsonLocation {
    pub id: String,
    #[serde(default)]
    pub id_was_generated: bool,
    pub geometry: GeoJsonGeometry,
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

impl GeoJsonLocation {
    #[must_use]
    pub fn stop_name(&self) -> Option<&str> {
        self.properties.get("stop_name")?.as_str()
    }

    #[must_use]
    pub fn stop_desc(&self) -> Option<&str> {
        self.properties.get("stop_desc")?.as_str()
    }

    #[must_use]
    pub fn zone_id(&self) -> Option<&str> {
        self.properties.get("zone_id")?.as_str()
    }
}
