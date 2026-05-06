use std::collections::HashMap;

use crate::models::rt::{
    TripUpdate, trip_descriptor::ScheduleRelationship as TripScheduleRelationship,
    trip_update::StopTimeUpdate,
};
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
        if !v.is_empty() && v != "1.0" && v != "2.0" {
            vec![err(
                self.rule_id(),
                Severity::Warning,
                format!("gtfs_realtime_version `{v}` is not 1.0 or 2.0"),
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
                    tu.trip.schedule_relationship,
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
                    trip.schedule_relationship,
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
    schedule_relationship: Option<i32>,
    idx: &ScheduleIndex,
    source: &str,
    errors: &mut Vec<ValidationError>,
) {
    // ADDED trips are not expected to exist in the static Schedule.
    if schedule_relationship == Some(TripScheduleRelationship::Added as i32) {
        return;
    }
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

pub struct StopTimeSequenceUnsortedRule;
impl RtValidationRule for StopTimeSequenceUnsortedRule {
    fn rule_id(&self) -> &'static str {
        "stop_time_sequence_unsorted"
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
            let Some(tu) = &entity.trip_update else {
                continue;
            };
            for pair in tu.stop_time_update.windows(2) {
                if let (Some(a), Some(b)) = (pair[0].stop_sequence, pair[1].stop_sequence)
                    && b <= a
                {
                    errors.push(err(
                        "stop_time_sequence_unsorted",
                        Severity::Error,
                        format!(
                            "TripUpdate {} stop_sequence not strictly increasing ({a} → {b}, trip_id={})",
                            entity_label(&entity.id),
                            trip_id_str(tu.trip.trip_id.as_ref()),
                        ),
                    ));
                }
            }
        }
        errors
    }
}

pub struct MissingStopSequenceForRepeatedStopRule;
impl RtValidationRule for MissingStopSequenceForRepeatedStopRule {
    fn rule_id(&self) -> &'static str {
        "missing_stop_sequence_for_repeated_stop"
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
            let Some(tu) = &entity.trip_update else {
                continue;
            };
            let Some(trip_id) = tu.trip.trip_id.as_ref() else {
                continue;
            };
            if !idx.trip_repeated_stops.contains(trip_id) {
                continue;
            }
            for stu in &tu.stop_time_update {
                if stu.stop_sequence.is_none() {
                    errors.push(err(
                        "missing_stop_sequence_for_repeated_stop",
                        Severity::Error,
                        format!(
                            "TripUpdate {} omits stop_sequence on a stop_time_update for trip_id `{trip_id}` whose Schedule has repeated stop_ids",
                            entity_label(&entity.id),
                        ),
                    ));
                }
            }
        }
        errors
    }
}

pub struct RtStopWrongLocationTypeRule;
impl RtValidationRule for RtStopWrongLocationTypeRule {
    fn rule_id(&self) -> &'static str {
        "rt_stop_wrong_location_type"
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
                for stu in &tu.stop_time_update {
                    if let Some(sid) = &stu.stop_id
                        && let Some(&lt) = idx.stop_location_types.get(sid)
                        && lt != 0
                    {
                        errors.push(err(
                            "rt_stop_wrong_location_type",
                            Severity::Error,
                            format!(
                                "TripUpdate {} references stop_id `{sid}` with location_type={lt} (must be 0)",
                                entity_label(&entity.id),
                            ),
                        ));
                    }
                }
            }
            if let Some(vp) = &entity.vehicle
                && let Some(sid) = &vp.stop_id
                && let Some(&lt) = idx.stop_location_types.get(sid)
                && lt != 0
            {
                errors.push(err(
                    "rt_stop_wrong_location_type",
                    Severity::Error,
                    format!(
                        "VehiclePosition {} references stop_id `{sid}` with location_type={lt} (must be 0)",
                        entity_label(&entity.id),
                    ),
                ));
            }
        }
        errors
    }
}

pub struct StopTimeUpdateTimesNotIncreasingRule;
impl RtValidationRule for StopTimeUpdateTimesNotIncreasingRule {
    fn rule_id(&self) -> &'static str {
        "stop_time_update_times_not_increasing"
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
            let Some(tu) = &entity.trip_update else {
                continue;
            };
            for pair in tu.stop_time_update.windows(2) {
                let prev_arr = stop_time_event_time(&pair[0], TimeKind::Arrival);
                let next_arr = stop_time_event_time(&pair[1], TimeKind::Arrival);
                if let (Some(a), Some(b)) = (prev_arr, next_arr)
                    && b <= a
                {
                    errors.push(err(
                        "stop_time_update_times_not_increasing",
                        Severity::Error,
                        format!(
                            "TripUpdate {} arrival times not strictly increasing ({a} → {b}, trip_id={})",
                            entity_label(&entity.id),
                            trip_id_str(tu.trip.trip_id.as_ref()),
                        ),
                    ));
                }
                let prev_dep = stop_time_event_time(&pair[0], TimeKind::Departure);
                let next_dep = stop_time_event_time(&pair[1], TimeKind::Departure);
                if let (Some(a), Some(b)) = (prev_dep, next_dep)
                    && b <= a
                {
                    errors.push(err(
                        "stop_time_update_times_not_increasing",
                        Severity::Error,
                        format!(
                            "TripUpdate {} departure times not strictly increasing ({a} → {b}, trip_id={})",
                            entity_label(&entity.id),
                            trip_id_str(tu.trip.trip_id.as_ref()),
                        ),
                    ));
                }
            }
        }
        errors
    }
}

pub struct StartTimeMismatchFirstArrivalRule;
impl RtValidationRule for StartTimeMismatchFirstArrivalRule {
    fn rule_id(&self) -> &'static str {
        "start_time_mismatch_first_arrival"
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
            let Some(tu) = &entity.trip_update else {
                continue;
            };
            let Some(start_time) = tu.trip.start_time.as_ref() else {
                continue;
            };
            let Some(trip_id) = tu.trip.trip_id.as_ref() else {
                continue;
            };
            if idx.trips_in_frequencies.contains(trip_id) {
                continue;
            }
            if let Some(expected) = idx.trip_first_arrivals.get(trip_id)
                && expected != start_time
            {
                errors.push(err(
                    "start_time_mismatch_first_arrival",
                    Severity::Error,
                    format!(
                        "TripUpdate {} start_time `{start_time}` does not match first GTFS arrival_time `{expected}` (trip_id=`{trip_id}`)",
                        entity_label(&entity.id),
                    ),
                ));
            }
        }
        errors
    }
}

pub struct ConsecutiveSameStopIdRule;
impl RtValidationRule for ConsecutiveSameStopIdRule {
    fn rule_id(&self) -> &'static str {
        "consecutive_same_stop_id"
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
            let Some(tu) = &entity.trip_update else {
                continue;
            };
            for pair in tu.stop_time_update.windows(2) {
                if let (Some(a), Some(b)) = (&pair[0].stop_id, &pair[1].stop_id)
                    && a == b
                {
                    errors.push(err(
                        "consecutive_same_stop_id",
                        Severity::Error,
                        format!(
                            "TripUpdate {} consecutive stop_time_updates share stop_id `{a}` (trip_id={})",
                            entity_label(&entity.id),
                            trip_id_str(tu.trip.trip_id.as_ref()),
                        ),
                    ));
                }
            }
        }
        errors
    }
}

pub struct MissingVehicleIdRule;
impl RtValidationRule for MissingVehicleIdRule {
    fn rule_id(&self) -> &'static str {
        "missing_vehicle_id"
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
            if let Some(tu) = &entity.trip_update {
                let id = tu.vehicle.as_ref().and_then(|v| v.id.as_deref());
                if id.is_none_or(str::is_empty) {
                    errors.push(err(
                        "missing_vehicle_id",
                        Severity::Warning,
                        format!(
                            "TripUpdate {} has no vehicle.id (trip_id={})",
                            entity_label(&entity.id),
                            trip_id_str(tu.trip.trip_id.as_ref()),
                        ),
                    ));
                }
            }
            if let Some(vp) = &entity.vehicle {
                let id = vp.vehicle.as_ref().and_then(|v| v.id.as_deref());
                if id.is_none_or(str::is_empty) {
                    errors.push(err(
                        "missing_vehicle_id",
                        Severity::Warning,
                        format!(
                            "VehiclePosition {} has no vehicle.id",
                            entity_label(&entity.id),
                        ),
                    ));
                }
            }
        }
        errors
    }
}

pub struct FeedNotFreshRule;
impl RtValidationRule for FeedNotFreshRule {
    fn rule_id(&self) -> &'static str {
        "feed_not_fresh"
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
        if ts == 0 {
            return Vec::new();
        }
        let age = ctx.now_unix.saturating_sub(ts);
        if age > 60 {
            vec![err(
                "feed_not_fresh",
                Severity::Warning,
                format!(
                    "FeedHeader.timestamp is {age}s old (must be < 60s; ts={ts}, now={})",
                    ctx.now_unix
                ),
            )]
        } else {
            Vec::new()
        }
    }
}

pub struct MissingScheduleRelationshipRule;
impl RtValidationRule for MissingScheduleRelationshipRule {
    fn rule_id(&self) -> &'static str {
        "missing_schedule_relationship"
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
            let Some(tu) = &entity.trip_update else {
                continue;
            };
            if tu.trip.schedule_relationship.is_none() {
                errors.push(err(
                    "missing_schedule_relationship",
                    Severity::Warning,
                    format!(
                        "TripUpdate {} trip.schedule_relationship is unset (trip_id={})",
                        entity_label(&entity.id),
                        trip_id_str(tu.trip.trip_id.as_ref()),
                    ),
                ));
            }
            for stu in &tu.stop_time_update {
                if stu.schedule_relationship.is_none() {
                    errors.push(err(
                        "missing_schedule_relationship",
                        Severity::Warning,
                        format!(
                            "TripUpdate {} stop_time_update.schedule_relationship is unset (trip_id={})",
                            entity_label(&entity.id),
                            trip_id_str(tu.trip.trip_id.as_ref()),
                        ),
                    ));
                }
            }
        }
        errors
    }
}
