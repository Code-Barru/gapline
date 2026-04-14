//! FK rule: `stops.parent_station` → `stops.stop_id` (self-reference).
//!
//! Parent type correctness is validated separately in section 7.

impl_fk_rule! {
    StopsParentStationFkRule,
    child_file: "stops.txt",
    child: feed.stops as s,
    child_fk: parent_station (optional),
    parent_file: "stops.txt",
    parent: feed.stops,
    parent_pk: stop_id (required),
    parent_entity: "stop",
}
