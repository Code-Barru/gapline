use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::fares_v2::{
    Area, FareLegJoinRule, FareLegRule, FareMedia, FareProduct, FareTransferRule, Network,
    RiderCategory, RouteNetwork, StopArea, Timeframe,
};
use super::flex::{BookingRule, LocationGroup, LocationGroupStop};
use super::geojson::GeoJsonLocation;
use super::records::{
    Agency, Attribution, Calendar, CalendarDate, FareAttribute, FareRule, FeedInfo, Frequency,
    Level, Pathway, Route, Shape, Stop, StopTime, Transfer, Translation, Trip,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GtfsFeed {
    /// Names of GTFS files that were present in the feed source.
    /// Used by conditional validation rules (e.g. `shape_id` required only when `shapes.txt` exists).
    pub loaded_files: HashSet<String>,
    pub agencies: Vec<Agency>,
    pub stops: Vec<Stop>,
    pub routes: Vec<Route>,
    pub trips: Vec<Trip>,
    pub stop_times: Vec<StopTime>,
    pub calendars: Vec<Calendar>,
    pub calendar_dates: Vec<CalendarDate>,
    pub shapes: Vec<Shape>,
    pub frequencies: Vec<Frequency>,
    pub transfers: Vec<Transfer>,
    pub pathways: Vec<Pathway>,
    pub levels: Vec<Level>,
    pub feed_info: Option<FeedInfo>,
    /// Number of data rows found in `feed_info.txt` (0 if absent).
    /// The spec allows at most one row; duplicates are detected by section 6.
    #[serde(default)]
    pub feed_info_line_count: usize,
    pub fare_attributes: Vec<FareAttribute>,
    pub fare_rules: Vec<FareRule>,
    pub translations: Vec<Translation>,
    pub attributions: Vec<Attribution>,
    pub booking_rules: Vec<BookingRule>,
    pub location_groups: Vec<LocationGroup>,
    pub location_group_stops: Vec<LocationGroupStop>,
    #[serde(default)]
    pub fare_media: Vec<FareMedia>,
    #[serde(default)]
    pub fare_products: Vec<FareProduct>,
    #[serde(default)]
    pub fare_leg_rules: Vec<FareLegRule>,
    #[serde(default)]
    pub fare_transfer_rules: Vec<FareTransferRule>,
    #[serde(default)]
    pub rider_categories: Vec<RiderCategory>,
    #[serde(default)]
    pub timeframes: Vec<Timeframe>,
    #[serde(default)]
    pub areas: Vec<Area>,
    #[serde(default)]
    pub stop_areas: Vec<StopArea>,
    #[serde(default)]
    pub networks: Vec<Network>,
    #[serde(default)]
    pub route_networks: Vec<RouteNetwork>,
    #[serde(default)]
    pub fare_leg_join_rules: Vec<FareLegJoinRule>,
    #[serde(default)]
    pub geojson_locations: Vec<GeoJsonLocation>,
}

impl GtfsFeed {
    /// Returns `true` if the given file name was present in the feed source.
    #[must_use]
    pub fn has_file(&self, name: &str) -> bool {
        self.loaded_files.contains(name)
    }

    #[must_use]
    pub fn has_fares_v2(&self) -> bool {
        [
            "fare_media.txt",
            "fare_products.txt",
            "fare_leg_rules.txt",
            "fare_transfer_rules.txt",
            "rider_categories.txt",
            "timeframes.txt",
            "areas.txt",
            "stop_areas.txt",
            "networks.txt",
            "route_networks.txt",
            "fare_leg_join_rules.txt",
        ]
        .iter()
        .any(|f| self.has_file(f))
    }

    #[must_use]
    pub fn has_flex(&self) -> bool {
        self.has_file("booking_rules.txt")
            || self.has_file("location_groups.txt")
            || self.has_file("location_group_stops.txt")
            || self.has_geojson()
            || self.stop_times.iter().any(|st| {
                st.pickup_booking_rule_id.is_some()
                    || st.drop_off_booking_rule_id.is_some()
                    || st.start_pickup_drop_off_window.is_some()
                    || st.end_pickup_drop_off_window.is_some()
            })
    }

    #[must_use]
    pub fn has_geojson(&self) -> bool {
        self.has_file("locations.geojson") || !self.geojson_locations.is_empty()
    }
}
