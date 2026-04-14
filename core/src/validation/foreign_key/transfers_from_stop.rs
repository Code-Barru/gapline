//! FK rule: `transfers.from_stop_id` → `stops.stop_id`.

impl_fk_rule! {
    TransfersFromStopFkRule,
    child_file: "transfers.txt",
    child: feed.transfers as t,
    child_fk: from_stop_id (optional),
    parent_file: "stops.txt",
    parent: feed.stops,
    parent_pk: stop_id (required),
    parent_entity: "stop",
}
