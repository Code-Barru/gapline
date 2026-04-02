use std::collections::HashSet;

use serde::{Deserialize, Serialize};

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
    pub fare_attributes: Vec<FareAttribute>,
    pub fare_rules: Vec<FareRule>,
    pub translations: Vec<Translation>,
    pub attributions: Vec<Attribution>,
}

impl GtfsFeed {
    /// Returns `true` if the given file name was present in the feed source.
    #[must_use]
    pub fn has_file(&self, name: &str) -> bool {
        self.loaded_files.contains(name)
    }
}
