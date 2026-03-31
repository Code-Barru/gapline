use serde::{Deserialize, Serialize};

use super::records::{
    Agency, Attribution, Calendar, CalendarDate, FareAttribute, FareRule, FeedInfo, Frequency,
    Level, Pathway, Route, Shape, Stop, StopTime, Transfer, Translation, Trip,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GtfsFeed {
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
