//! FK rule: `frequencies.trip_id` → `trips.trip_id`.

impl_fk_rule! {
    FrequenciesTripFkRule,
    child_file: "frequencies.txt",
    child: feed.frequencies as f,
    child_fk: trip_id (required),
    parent_file: "trips.txt",
    parent: feed.trips,
    parent_pk: trip_id (required),
    parent_entity: "trip",
}
