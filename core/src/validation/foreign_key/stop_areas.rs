//! FK rules: `stop_areas.area_id` → `areas.area_id` and
//! `stop_areas.stop_id` → `stops.stop_id`.

impl_fk_rule! {
    StopAreasAreaFkRule,
    child_file: "stop_areas.txt",
    child: feed.stop_areas as sa,
    child_fk: area_id (required),
    parent_file: "areas.txt",
    parent: feed.areas,
    parent_pk: area_id (required),
    parent_entity: "area",
}

impl_fk_rule! {
    StopAreasStopFkRule,
    child_file: "stop_areas.txt",
    child: feed.stop_areas as sa,
    child_fk: stop_id (required),
    parent_file: "stops.txt",
    parent: feed.stops,
    parent_pk: stop_id (required),
    parent_entity: "stop",
}
