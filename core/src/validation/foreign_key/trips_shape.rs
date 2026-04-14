//! FK rule: `trips.shape_id` → `shapes.shape_id`.

impl_fk_rule! {
    TripsShapeFkRule,
    child_file: "trips.txt",
    child: feed.trips as t,
    child_fk: shape_id (optional),
    parent_file: "shapes.txt",
    parent: feed.shapes,
    parent_pk: shape_id (required),
    parent_entity: "shape",
}
