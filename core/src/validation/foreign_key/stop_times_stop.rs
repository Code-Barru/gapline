//! FK rule: `stop_times.stop_id` → `stops.stop_id`.

impl_fk_rule! {
    StopTimesStopFkRule,
    child_file: "stop_times.txt",
    child: feed.stop_times as st,
    child_fk: stop_id (required),
    parent_file: "stops.txt",
    parent: feed.stops,
    parent_pk: stop_id (required),
    parent_entity: "stop",
}
