//! FK rule: `stop_times.trip_id` → `trips.trip_id`.

impl_fk_rule! {
    StopTimesTripFkRule,
    child_file: "stop_times.txt",
    child: feed.stop_times as st,
    child_fk: trip_id (required),
    parent_file: "trips.txt",
    parent: feed.trips,
    parent_pk: trip_id (required),
    parent_entity: "trip",
}
