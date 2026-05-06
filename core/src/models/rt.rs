//! GTFS-Realtime feed wrapper.

use std::path::Path;

use prost::Message;
use thiserror::Error;

#[allow(clippy::all, clippy::pedantic, clippy::nursery, rustdoc::all)]
pub mod transit_realtime {
    include!(concat!(env!("OUT_DIR"), "/transit_realtime.rs"));
}

pub use transit_realtime::{
    Alert, EntitySelector, FeedEntity, FeedHeader, FeedMessage, Position, ReplacementStop, Shape,
    Stop, StopSelector, TimeRange, TranslatedImage, TranslatedString, TripDescriptor,
    TripModifications, TripUpdate, VehicleDescriptor, VehiclePosition, alert, feed_header, stop,
    translated_image, translated_string, trip_descriptor, trip_modifications, trip_update,
    vehicle_descriptor, vehicle_position,
};

#[derive(Debug, Error)]
pub enum RtError {
    #[error("invalid protobuf message: {0}")]
    Decode(#[from] prost::DecodeError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct GtfsRtFeed {
    inner: FeedMessage,
    trip_updates: Vec<TripUpdate>,
    vehicle_positions: Vec<VehiclePosition>,
    alerts: Vec<Alert>,
}

impl GtfsRtFeed {
    /// Decode a GTFS-Realtime `FeedMessage` from raw protobuf bytes.
    ///
    /// # Errors
    /// Returns [`RtError::Decode`] if the bytes are not a valid protobuf
    /// `FeedMessage` (truncated, wrong schema, or non-protobuf data).
    pub fn from_bytes(data: &[u8]) -> Result<Self, RtError> {
        let inner = FeedMessage::decode(data)?;
        let mut trip_updates = Vec::new();
        let mut vehicle_positions = Vec::new();
        let mut alerts = Vec::new();
        for entity in &inner.entity {
            if let Some(tu) = &entity.trip_update {
                trip_updates.push(tu.clone());
            }
            if let Some(vp) = &entity.vehicle {
                vehicle_positions.push(vp.clone());
            }
            if let Some(a) = &entity.alert {
                alerts.push(a.clone());
            }
        }
        Ok(Self {
            inner,
            trip_updates,
            vehicle_positions,
            alerts,
        })
    }

    /// Read and decode a GTFS-Realtime `.pb` file.
    ///
    /// # Errors
    /// Returns [`RtError::Io`] if the file cannot be read,
    /// [`RtError::Decode`] if its contents are not a valid `FeedMessage`.
    pub fn from_file(path: &Path) -> Result<Self, RtError> {
        let data = std::fs::read(path)?;
        Self::from_bytes(&data)
    }

    #[must_use]
    pub fn message(&self) -> &FeedMessage {
        &self.inner
    }

    #[must_use]
    pub fn header(&self) -> &FeedHeader {
        &self.inner.header
    }

    #[must_use]
    pub fn gtfs_realtime_version(&self) -> &str {
        &self.inner.header.gtfs_realtime_version
    }

    #[must_use]
    pub fn incrementality(&self) -> feed_header::Incrementality {
        self.inner.header.incrementality()
    }

    #[must_use]
    pub fn timestamp(&self) -> Option<u64> {
        self.inner.header.timestamp
    }

    #[must_use]
    pub fn trip_updates(&self) -> &[TripUpdate] {
        &self.trip_updates
    }

    #[must_use]
    pub fn vehicle_positions(&self) -> &[VehiclePosition] {
        &self.vehicle_positions
    }

    #[must_use]
    pub fn alerts(&self) -> &[Alert] {
        &self.alerts
    }
}
