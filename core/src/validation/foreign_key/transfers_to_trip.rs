//! FK rule: `transfers.to_trip_id` → `trips.trip_id`.

impl_fk_rule! {
    TransfersToTripFkRule,
    child_file: "transfers.txt",
    child: feed.transfers as t,
    child_fk: to_trip_id (optional),
    parent_file: "trips.txt",
    parent: feed.trips,
    parent_pk: trip_id (required),
    parent_entity: "trip",
}
