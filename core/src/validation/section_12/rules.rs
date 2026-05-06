use std::collections::HashMap;

use crate::models::rt::{TripUpdate, trip_update::StopTimeUpdate};
use crate::validation::rt_rules::{RtValidationContext, RtValidationRule, ScheduleIndex};
use crate::validation::{Severity, ValidationError};

const SECTION: &str = "12";

fn err(rule_id: &'static str, sev: Severity, msg: impl Into<String>) -> ValidationError {
    ValidationError::new(rule_id, SECTION, sev).message(msg.into())
}

fn entity_label(id: &str) -> String {
    format!("entity_id={id}")
}

fn trip_id_str(tu_trip: Option<&String>) -> &str {
    tu_trip.map_or("<missing>", String::as_str)
}

pub struct MissingHeaderRule;
impl RtValidationRule for MissingHeaderRule {
    fn rule_id(&self) -> &'static str {
        "missing_header"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, ctx: &RtValidationContext<'_>) -> Vec<ValidationError> {
        if ctx.rt.gtfs_realtime_version().is_empty() {
            vec![err(
                self.rule_id(),
                Severity::Error,
                "FeedMessage header missing or invalid (gtfs_realtime_version empty)",
            )]
        } else {
            Vec::new()
        }
    }
}

pub struct UnsupportedVersionRule;
impl RtValidationRule for UnsupportedVersionRule {
    fn rule_id(&self) -> &'static str {
        "unsupported_version"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, ctx: &RtValidationContext<'_>) -> Vec<ValidationError> {
        let v = ctx.rt.gtfs_realtime_version();
        if !v.is_empty() && v != "2.0" {
            vec![err(
                self.rule_id(),
                Severity::Warning,
                format!("gtfs_realtime_version `{v}` is not 2.0"),
            )]
        } else {
            Vec::new()
        }
    }
}

pub struct MissingTimestampRule;
impl RtValidationRule for MissingTimestampRule {
    fn rule_id(&self) -> &'static str {
        "missing_or_zero_timestamp"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, ctx: &RtValidationContext<'_>) -> Vec<ValidationError> {
        match ctx.rt.timestamp() {
            None | Some(0) => vec![err(
                self.rule_id(),
                Severity::Error,
                "FeedHeader.timestamp is missing or zero",
            )],
            _ => Vec::new(),
        }
    }
}

pub struct FutureTimestampRule;
impl RtValidationRule for FutureTimestampRule {
    fn rule_id(&self) -> &'static str {
        "future_timestamp"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, ctx: &RtValidationContext<'_>) -> Vec<ValidationError> {
        let Some(ts) = ctx.rt.timestamp() else {
            return Vec::new();
        };
        if ts > ctx.now_unix.saturating_add(3600) {
            vec![err(
                self.rule_id(),
                Severity::Warning,
                format!(
                    "FeedHeader.timestamp {ts} is more than 1h ahead of now ({})",
                    ctx.now_unix
                ),
            )]
        } else {
            Vec::new()
        }
    }
}

fn schedule_or_skip<'a>(ctx: &'a RtValidationContext<'_>) -> Option<&'a ScheduleIndex> {
    ctx.schedule_index
}

pub struct RtTripNotInScheduleRule;
impl RtValidationRule for RtTripNotInScheduleRule {
    fn rule_id(&self) -> &'static str {
        "rt_trip_not_in_schedule"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, ctx: &RtValidationContext<'_>) -> Vec<ValidationError> {
        let Some(idx) = schedule_or_skip(ctx) else {
            return Vec::new();
        };
        let mut errors = Vec::new();
        for entity in &ctx.rt.message().entity {
            if let Some(tu) = &entity.trip_update {
                check_trip(
                    &entity.id,
                    tu.trip.trip_id.as_ref(),
                    idx,
                    "TripUpdate",
                    &mut errors,
                );
            }
            if let Some(vp) = &entity.vehicle
                && let Some(trip) = &vp.trip
            {
                check_trip(
                    &entity.id,
                    trip.trip_id.as_ref(),
                    idx,
                    "VehiclePosition",
                    &mut errors,
                );
            }
        }
        errors
    }
}

fn check_trip(
    entity_id: &str,
    trip_id: Option<&String>,
    idx: &ScheduleIndex,
    source: &str,
    errors: &mut Vec<ValidationError>,
) {
    match trip_id {
        None => errors.push(err(
            "rt_trip_not_in_schedule",
            Severity::Error,
            format!(
                "{source} {} has no trip_id (cannot match Schedule)",
                entity_label(entity_id)
            ),
        )),
        Some(id) => {
            if !idx.trip_ids.contains(id) {
                errors.push(err(
                    "rt_trip_not_in_schedule",
                    Severity::Error,
                    format!(
                        "{source} {} references trip_id `{id}` not present in Schedule",
                        entity_label(entity_id)
                    ),
                ));
            }
        }
    }
}

pub struct RtRouteNotInScheduleRule;
impl RtValidationRule for RtRouteNotInScheduleRule {
    fn rule_id(&self) -> &'static str {
        "rt_route_not_in_schedule"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, ctx: &RtValidationContext<'_>) -> Vec<ValidationError> {
        let Some(idx) = schedule_or_skip(ctx) else {
            return Vec::new();
        };
        let mut errors = Vec::new();
        for entity in &ctx.rt.message().entity {
            if let Some(tu) = &entity.trip_update
                && let Some(route_id) = &tu.trip.route_id
                && !idx.route_ids.contains(route_id)
            {
                errors.push(err(
                    self.rule_id(),
                    Severity::Error,
                    format!(
                        "TripUpdate {} references route_id `{route_id}` not present in Schedule \
                         (trip_id={})",
                        entity_label(&entity.id),
                        trip_id_str(tu.trip.trip_id.as_ref()),
                    ),
                ));
            }
        }
        errors
    }
}

pub struct RtStopNotInScheduleRule;
impl RtValidationRule for RtStopNotInScheduleRule {
    fn rule_id(&self) -> &'static str {
        "rt_stop_not_in_schedule"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, ctx: &RtValidationContext<'_>) -> Vec<ValidationError> {
        let Some(idx) = schedule_or_skip(ctx) else {
            return Vec::new();
        };
        let mut errors = Vec::new();
        for entity in &ctx.rt.message().entity {
            if let Some(tu) = &entity.trip_update {
                check_trip_update_stops(&entity.id, tu, idx, &mut errors);
            }
            if let Some(vp) = &entity.vehicle
                && let Some(stop_id) = &vp.stop_id
                && !idx.stop_ids.contains(stop_id)
            {
                errors.push(err(
                    "rt_stop_not_in_schedule",
                    Severity::Error,
                    format!(
                        "VehiclePosition {} references stop_id `{stop_id}` not present in \
                         Schedule",
                        entity_label(&entity.id)
                    ),
                ));
            }
        }
        errors
    }
}

fn check_trip_update_stops(
    entity_id: &str,
    tu: &TripUpdate,
    idx: &ScheduleIndex,
    errors: &mut Vec<ValidationError>,
) {
    for stu in &tu.stop_time_update {
        if let Some(stop_id) = &stu.stop_id
            && !idx.stop_ids.contains(stop_id)
        {
            errors.push(err(
                "rt_stop_not_in_schedule",
                Severity::Error,
                format!(
                    "TripUpdate.stop_time_update {} references stop_id `{stop_id}` not \
                     present in Schedule (trip_id={})",
                    entity_label(entity_id),
                    trip_id_str(tu.trip.trip_id.as_ref()),
                ),
            ));
        }
    }
}

pub struct PositionOutsideFeedBoundsRule;
impl RtValidationRule for PositionOutsideFeedBoundsRule {
    fn rule_id(&self) -> &'static str {
        "position_outside_feed_bounds"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, ctx: &RtValidationContext<'_>) -> Vec<ValidationError> {
        let Some(idx) = schedule_or_skip(ctx) else {
            return Vec::new();
        };
        let Some(bbox) = idx.bbox else {
            return Vec::new();
        };
        let mut errors = Vec::new();
        for entity in &ctx.rt.message().entity {
            if let Some(vp) = &entity.vehicle
                && let Some(pos) = &vp.position
            {
                let lat = f64::from(pos.latitude);
                let lon = f64::from(pos.longitude);
                if lat < bbox.min_lat
                    || lat > bbox.max_lat
                    || lon < bbox.min_lon
                    || lon > bbox.max_lon
                {
                    errors.push(err(
                        self.rule_id(),
                        Severity::Warning,
                        format!(
                            "VehiclePosition {} at ({lat}, {lon}) is outside Schedule bbox \
                             [({}, {}), ({}, {})]",
                            entity_label(&entity.id),
                            bbox.min_lat,
                            bbox.min_lon,
                            bbox.max_lat,
                            bbox.max_lon,
                        ),
                    ));
                }
            }
        }
        errors
    }
}

pub struct UnorderedStopTimesRule;
impl RtValidationRule for UnorderedStopTimesRule {
    fn rule_id(&self) -> &'static str {
        "unordered_stop_times"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn validate(&self, ctx: &RtValidationContext<'_>) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for entity in &ctx.rt.message().entity {
            if let Some(tu) = &entity.trip_update {
                check_stop_time_order(&entity.id, tu, &mut errors);
            }
        }
        errors
    }
}

fn stop_time_event_time(stu: &StopTimeUpdate, kind: TimeKind) -> Option<i64> {
    let ev = match kind {
        TimeKind::Arrival => stu.arrival.as_ref(),
        TimeKind::Departure => stu.departure.as_ref(),
    };
    ev.and_then(|e| e.time)
}

#[derive(Clone, Copy)]
enum TimeKind {
    Arrival,
    Departure,
}

fn check_stop_time_order(entity_id: &str, tu: &TripUpdate, errors: &mut Vec<ValidationError>) {
    for pair in tu.stop_time_update.windows(2) {
        let prev_dep = stop_time_event_time(&pair[0], TimeKind::Departure);
        let next_arr = stop_time_event_time(&pair[1], TimeKind::Arrival);
        if let (Some(d), Some(a)) = (prev_dep, next_arr)
            && a < d
        {
            errors.push(err(
                "unordered_stop_times",
                Severity::Error,
                format!(
                    "TripUpdate {} stop_time_update arrival {a} precedes previous departure \
                     {d} (trip_id={})",
                    entity_label(entity_id),
                    trip_id_str(tu.trip.trip_id.as_ref()),
                ),
            ));
        }
    }
}

pub struct ExcessiveDelayRule;
impl RtValidationRule for ExcessiveDelayRule {
    fn rule_id(&self) -> &'static str {
        "excessive_delay"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, ctx: &RtValidationContext<'_>) -> Vec<ValidationError> {
        let max = i64::from(ctx.max_delay_seconds);
        let mut errors = Vec::new();
        for entity in &ctx.rt.message().entity {
            if let Some(tu) = &entity.trip_update {
                check_delays(&entity.id, tu, max, &mut errors);
            }
        }
        errors
    }
}

fn check_delays(entity_id: &str, tu: &TripUpdate, max: i64, errors: &mut Vec<ValidationError>) {
    for stu in &tu.stop_time_update {
        for (kind, ev) in [
            ("arrival", stu.arrival.as_ref()),
            ("departure", stu.departure.as_ref()),
        ] {
            if let Some(ev) = ev
                && let Some(delay) = ev.delay
                && i64::from(delay).abs() > max
            {
                errors.push(err(
                    "excessive_delay",
                    Severity::Warning,
                    format!(
                        "TripUpdate {} {kind} delay {delay}s exceeds max {max}s \
                         (trip_id={})",
                        entity_label(entity_id),
                        trip_id_str(tu.trip.trip_id.as_ref()),
                    ),
                ));
            }
        }
    }
}

pub struct AlertWithoutTargetRule;
impl RtValidationRule for AlertWithoutTargetRule {
    fn rule_id(&self) -> &'static str {
        "alert_without_target"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, ctx: &RtValidationContext<'_>) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for entity in &ctx.rt.message().entity {
            if let Some(alert) = &entity.alert
                && alert.informed_entity.is_empty()
            {
                errors.push(err(
                    self.rule_id(),
                    Severity::Warning,
                    format!("Alert {} has no informed_entity", entity_label(&entity.id)),
                ));
            }
        }
        errors
    }
}

pub struct AlertTargetNotInScheduleRule;
impl RtValidationRule for AlertTargetNotInScheduleRule {
    fn rule_id(&self) -> &'static str {
        "alert_target_not_in_schedule"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, ctx: &RtValidationContext<'_>) -> Vec<ValidationError> {
        let Some(idx) = schedule_or_skip(ctx) else {
            return Vec::new();
        };
        let mut errors = Vec::new();
        for entity in &ctx.rt.message().entity {
            if let Some(alert) = &entity.alert {
                for sel in &alert.informed_entity {
                    if let Some(rid) = &sel.route_id
                        && !idx.route_ids.contains(rid)
                    {
                        errors.push(err(
                            self.rule_id(),
                            Severity::Warning,
                            format!(
                                "Alert {} references route_id `{rid}` not in Schedule",
                                entity_label(&entity.id)
                            ),
                        ));
                    }
                    if let Some(sid) = &sel.stop_id
                        && !idx.stop_ids.contains(sid)
                    {
                        errors.push(err(
                            self.rule_id(),
                            Severity::Warning,
                            format!(
                                "Alert {} references stop_id `{sid}` not in Schedule",
                                entity_label(&entity.id)
                            ),
                        ));
                    }
                    if let Some(trip) = &sel.trip
                        && let Some(tid) = &trip.trip_id
                        && !idx.trip_ids.contains(tid)
                    {
                        errors.push(err(
                            self.rule_id(),
                            Severity::Warning,
                            format!(
                                "Alert {} references trip_id `{tid}` not in Schedule",
                                entity_label(&entity.id)
                            ),
                        ));
                    }
                }
            }
        }
        errors
    }
}

pub struct DuplicateEntityIdRule;
impl RtValidationRule for DuplicateEntityIdRule {
    fn rule_id(&self) -> &'static str {
        "duplicate_entity_id"
    }
    fn section(&self) -> &'static str {
        SECTION
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn validate(&self, ctx: &RtValidationContext<'_>) -> Vec<ValidationError> {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for entity in &ctx.rt.message().entity {
            *counts.entry(entity.id.as_str()).or_insert(0) += 1;
        }
        counts
            .into_iter()
            .filter(|&(_, c)| c > 1)
            .map(|(id, c)| {
                err(
                    "duplicate_entity_id",
                    Severity::Warning,
                    format!("FeedEntity id `{id}` appears {c} times"),
                )
            })
            .collect()
    }
}
